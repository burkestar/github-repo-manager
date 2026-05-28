use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::Config;
use crate::error::Result;
use crate::events::AppEvent;
use crate::git::workspace::{load_workspace_cache, repo_target_path, save_workspace_cache, scan_workspace};
use crate::github::cache::RepoCache;
use crate::github::client::fetch_org_repos;
use crate::scheduler::start_scheduler;
use crate::state::{AppState, CloneDialogState, CloneStage, PanelFocus, StatusLevel};
use crate::ui;

pub struct App {
    state: AppState,
    cache: RepoCache,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let state = AppState::new(config);
        let cache = RepoCache::new();
        Self {
            state,
            cache,
            event_tx,
            event_rx,
        }
    }

    pub async fn run(mut self, mut terminal: ratatui::DefaultTerminal) -> Result<()> {
        self.state.checked_out = load_workspace_cache();
        info!("Loaded {} repos from workspace cache", self.state.checked_out.len());

        let fresh = scan_workspace(&self.state.config.workspace_root, &self.state.config.layout);
        info!("Found {} checked-out repos after scan", fresh.len());
        self.state.checked_out = fresh;
        save_workspace_cache(&self.state.checked_out);

        self.load_repos_for_current_org();

        let _scheduler = start_scheduler(self.state.config.clone(), self.event_tx.clone()).await?;

        let tick_rate = Duration::from_millis(50);
        let mut tick_interval = tokio::time::interval(tick_rate);
        tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut crossterm_events = EventStream::new();

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    self.state.check_status_expiry();
                    terminal.draw(|frame| ui::draw(frame, &mut self.state))?;
                }
                Some(Ok(event)) = crossterm_events.next() => {
                    self.handle_crossterm_event(event);
                }
                Some(app_event) = self.event_rx.recv() => {
                    self.handle_app_event(app_event);
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn load_repos_for_current_org(&mut self) {
        let org = self.state.current_org().to_string();

        if let Some(repos) = self.cache.load_org(&org) {
            info!("Loaded {} repos for {} from cache", repos.len(), org);
            self.state.repos.insert(org.clone(), repos.clone());
            self.update_filtered_repos();
            return;
        }

        self.state.repos_loading.insert(org.clone());
        let token = self.state.config.github_token.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            info!("Fetching repos for org: {}", org);
            match fetch_org_repos(&org, &token).await {
                Ok(repos) => {
                    let _ = tx.send(AppEvent::ReposLoaded { org, repos });
                }
                Err(e) => {
                    error!("Failed to fetch repos for {}: {}", org, e);
                    let _ = tx.send(AppEvent::ReposFailed {
                        org,
                        error: e.to_string(),
                    });
                }
            }
        });
    }

    fn update_filtered_repos(&mut self) {
        let org = self.state.current_org().to_string();
        let repos = self.state.repos.get(&org).cloned().unwrap_or_default();
        let show_archived = self.state.show_archived;

        let visible: Vec<usize> = (0..repos.len())
            .filter(|&i| show_archived || !repos[i].archived)
            .collect();

        if self.state.search_query.is_empty() {
            self.state.filtered_repos = visible;
        } else {
            let matcher = SkimMatcherV2::default();
            let query = self.state.search_query.to_lowercase();
            let mut scored: Vec<(usize, i64)> = visible
                .into_iter()
                .filter_map(|i| {
                    matcher
                        .fuzzy_match(&repos[i].name.to_lowercase(), &query)
                        .map(|score| (i, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.state.filtered_repos = scored.into_iter().map(|(i, _)| i).collect();
        }

        let max = self.state.filtered_repos.len().saturating_sub(1);
        let current = self.state.repo_list_state.selected().unwrap_or(0);
        self.state.repo_list_state.select(Some(current.min(max)));
    }

    fn handle_crossterm_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            // Quit
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.state.should_quit = true;
                return;
            }

            if self.state.clone_dialog.is_some() {
                self.handle_clone_dialog_key(key);
            } else if self.state.search_active {
                self.handle_search_key(key);
            } else {
                self.handle_normal_key(key);
            }
        }
    }

    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.state.should_quit = true,
            KeyCode::Tab => {
                self.state.focus = match self.state.focus {
                    PanelFocus::OrgPanel => PanelFocus::RepoPanel,
                    PanelFocus::RepoPanel => PanelFocus::OrgPanel,
                };
            }
            KeyCode::Char('h') => self.state.focus = PanelFocus::OrgPanel,
            KeyCode::Char('l') => self.state.focus = PanelFocus::RepoPanel,

            KeyCode::Down | KeyCode::Char('j') => self.nav_down(),
            KeyCode::Up | KeyCode::Char('k') => self.nav_up(),

            KeyCode::Char('/') => {
                self.state.focus = PanelFocus::RepoPanel;
                self.state.search_active = true;
            }

            KeyCode::Enter => self.handle_enter(),

            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.trigger_batch_fetch();
            }
            KeyCode::Char('F') => self.trigger_batch_fetch(),
            KeyCode::Char('f') => self.trigger_single_fetch(),

            KeyCode::Char('a') => {
                self.state.show_archived = !self.state.show_archived;
                self.update_filtered_repos();
                let msg = if self.state.show_archived {
                    "Showing archived repos"
                } else {
                    "Hiding archived repos"
                };
                self.state.set_status(msg, StatusLevel::Info);
            }

            KeyCode::Char('r') => {
                let org = self.state.current_org().to_string();
                self.cache.invalidate_org(&org);
                self.load_repos_for_current_org();
                self.state
                    .set_status("Refreshing repos from GitHub…", StatusLevel::Info);
            }

            _ => {}
        }
    }

    fn nav_down(&mut self) {
        match self.state.focus {
            PanelFocus::OrgPanel => {
                let next =
                    (self.state.selected_org_idx + 1).min(self.state.orgs.len().saturating_sub(1));
                if next != self.state.selected_org_idx {
                    self.state.selected_org_idx = next;
                    self.state.org_list_state.select(Some(next));
                    self.state.search_query.clear();
                    self.state.search_active = false;
                    self.load_repos_for_current_org();
                }
            }
            PanelFocus::RepoPanel => {
                let max = self.state.filtered_repos.len().saturating_sub(1);
                let next = self
                    .state
                    .repo_list_state
                    .selected()
                    .map(|i| (i + 1).min(max))
                    .unwrap_or(0);
                self.state.repo_list_state.select(Some(next));
            }
        }
    }

    fn nav_up(&mut self) {
        match self.state.focus {
            PanelFocus::OrgPanel => {
                let prev = self.state.selected_org_idx.saturating_sub(1);
                if prev != self.state.selected_org_idx {
                    self.state.selected_org_idx = prev;
                    self.state.org_list_state.select(Some(prev));
                    self.state.search_query.clear();
                    self.state.search_active = false;
                    self.load_repos_for_current_org();
                }
            }
            PanelFocus::RepoPanel => {
                let prev = self
                    .state
                    .repo_list_state
                    .selected()
                    .map(|i| i.saturating_sub(1))
                    .unwrap_or(0);
                self.state.repo_list_state.select(Some(prev));
            }
        }
    }

    fn handle_enter(&mut self) {
        match self.state.focus {
            PanelFocus::OrgPanel => {
                self.state.focus = PanelFocus::RepoPanel;
            }
            PanelFocus::RepoPanel => {
                if let Some(repo) = self.state.selected_repo().cloned() {
                    if self.state.checked_out.contains_key(&repo.full_name) {
                        let path = self.state.checked_out[&repo.full_name]
                            .local_path
                            .display()
                            .to_string();
                        self.state
                            .set_status(format!("Already at: {path}"), StatusLevel::Info);
                    } else {
                        self.state.clone_dialog = Some(CloneDialogState {
                            repo,
                            stage: CloneStage::Confirm,
                        });
                    }
                }
            }
        }
    }

    fn handle_clone_dialog_key(&mut self, key: crossterm::event::KeyEvent) {
        let stage = self.state.clone_dialog.as_ref().map(|d| d.stage.clone());

        match stage {
            Some(CloneStage::Confirm) => match key.code {
                KeyCode::Enter => self.start_clone(),
                KeyCode::Esc => self.state.clone_dialog = None,
                _ => {}
            },
            Some(CloneStage::Done(_)) | Some(CloneStage::Failed(_)) => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                    self.state.clone_dialog = None;
                }
            }
            _ => {
                if key.code == KeyCode::Esc {
                    self.state.clone_dialog = None;
                }
            }
        }
    }

    fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state.search_active = false;
                self.state.search_query.clear();
                self.update_filtered_repos();
            }
            KeyCode::Enter => {
                self.state.search_active = false;
                self.handle_enter();
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
                self.update_filtered_repos();
            }
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
                self.update_filtered_repos();
            }
            KeyCode::Down => self.nav_down(),
            KeyCode::Up => self.nav_up(),
            _ => {}
        }
    }

    fn start_clone(&mut self) {
        let dialog = match &self.state.clone_dialog {
            Some(d) => d.clone(),
            None => return,
        };

        let repo = dialog.repo.clone();
        let target = repo_target_path(
            &self.state.config.workspace_root,
            &self.state.config.layout,
            &repo.full_name,
        );

        if target.join(".git").exists() {
            if let Some(dialog) = &mut self.state.clone_dialog {
                dialog.stage = CloneStage::Done(target.clone());
            }
            self.state.set_status(
                format!("{} already checked out at {}", repo.full_name, target.display()),
                StatusLevel::Info,
            );
            self.state.checked_out =
                scan_workspace(&self.state.config.workspace_root, &self.state.config.layout);
            save_workspace_cache(&self.state.checked_out);
            return;
        }

        if let Some(parent) = target.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.state.clone_dialog = Some(CloneDialogState {
                    repo,
                    stage: CloneStage::Failed(e.to_string()),
                });
                return;
            }
        }

        let (progress_tx, mut progress_rx) = tokio::sync::watch::channel(0.0f64);
        let tx = self.event_tx.clone();
        let clone_url = repo.clone_url.clone();
        let token = self.state.config.github_token.clone();
        let target_clone = target.clone();
        let repo_name = repo.full_name.clone();

        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                crate::git::clone::clone_repo(&clone_url, &target_clone, &token, progress_tx)
            });

            let progress_tx2 = tx.clone();
            let repo_name2 = repo_name.clone();
            tokio::spawn(async move {
                while progress_rx.changed().await.is_ok() {
                    let pct = *progress_rx.borrow();
                    let _ = progress_tx2.send(AppEvent::CloneProgress {
                        repo: repo_name2.clone(),
                        progress: pct,
                    });
                }
            });

            match result.await {
                Ok(Ok(())) => {
                    let _ = tx.send(AppEvent::CloneCompleted {
                        repo: repo_name,
                        path: target,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(AppEvent::CloneFailed {
                        repo: repo_name,
                        error: e.to_string(),
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::CloneFailed {
                        repo: repo_name,
                        error: e.to_string(),
                    });
                }
            }
        });

        if let Some(dialog) = &mut self.state.clone_dialog {
            dialog.stage = CloneStage::Cloning { progress: 0.0 };
        }
    }

    fn trigger_single_fetch(&mut self) {
        let repo = match self.state.selected_repo().cloned() {
            Some(r) => r,
            None => return,
        };
        let checkout = match self.state.checked_out.get(&repo.full_name).cloned() {
            Some(c) => c,
            None => {
                self.state
                    .set_status("Repo is not checked out locally", StatusLevel::Info);
                return;
            }
        };
        let path = checkout.local_path.clone();
        let token = self.state.config.github_token.clone();
        let tx = self.event_tx.clone();
        let repo_name = repo.full_name.clone();

        tokio::spawn(async move {
            let result =
                tokio::task::spawn_blocking(move || crate::git::fetch::fetch_repo(&path, &token))
                    .await;
            match result {
                Ok(Ok(())) => {
                    let _ = tx.send(AppEvent::FetchCompleted { repo: repo_name });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(AppEvent::FetchFailed {
                        repo: repo_name,
                        error: e.to_string(),
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::FetchFailed {
                        repo: repo_name,
                        error: e.to_string(),
                    });
                }
            }
        });

        self.state
            .set_status(format!("Fetching {}…", repo.name), StatusLevel::Info);
    }

    fn trigger_batch_fetch(&mut self) {
        let checked_out: Vec<_> = self.state.checked_out.values().cloned().collect();
        if checked_out.is_empty() {
            self.state
                .set_status("No local repos to fetch", StatusLevel::Info);
            return;
        }
        let token = self.state.config.github_token.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let _ = tx.send(AppEvent::BatchFetchStarted);
            let (mut fetched, mut failed) = (0usize, 0usize);
            for info in &checked_out {
                let path = info.local_path.clone();
                let t = token.clone();
                let result =
                    tokio::task::spawn_blocking(move || crate::git::fetch::fetch_repo(&path, &t))
                        .await;
                if result.is_ok_and(|r| r.is_ok()) {
                    fetched += 1;
                } else {
                    failed += 1;
                }
            }
            let _ = tx.send(AppEvent::BatchFetchCompleted { fetched, failed });
        });

        self.state
            .set_status("Batch fetch started…", StatusLevel::Info);
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::ReposLoaded { org, repos } => {
                info!("Repos loaded for {}: {} repos", org, repos.len());
                self.state.repos_loading.remove(&org);
                self.cache.store_org(&org, &repos);
                self.state.repos.insert(org.clone(), repos);
                if org == self.state.current_org() {
                    self.update_filtered_repos();
                }
            }

            AppEvent::ReposFailed { org, error } => {
                self.state.repos_loading.remove(&org);
                let display = if error.contains("SAML")
                    || error.contains("enforcement")
                    || error.contains("API error: GitHub")
                {
                    format!("SSO auth required for '{org}': authorize token at github.com/settings/tokens → Configure SSO")
                } else {
                    format!("GitHub error ({org}): {error}")
                };
                self.state.set_status(display, StatusLevel::Error);
                error!("Failed to load repos for {}: {}", org, error);
            }

            AppEvent::CloneProgress { repo, progress } => {
                if let Some(dialog) = &mut self.state.clone_dialog {
                    if dialog.repo.full_name == repo {
                        dialog.stage = CloneStage::Cloning { progress };
                    }
                }
            }

            AppEvent::CloneCompleted { repo, path } => {
                if let Some(dialog) = &mut self.state.clone_dialog {
                    if dialog.repo.full_name == repo {
                        dialog.stage = CloneStage::Done(path.clone());
                    }
                }
                self.state
                    .set_status(format!("Cloned {repo}"), StatusLevel::Success);
                self.state.checked_out =
                    scan_workspace(&self.state.config.workspace_root, &self.state.config.layout);
                save_workspace_cache(&self.state.checked_out);
                info!("Clone complete: {} → {}", repo, path.display());
            }

            AppEvent::CloneFailed { repo, error } => {
                if let Some(dialog) = &mut self.state.clone_dialog {
                    if dialog.repo.full_name == repo {
                        dialog.stage = CloneStage::Failed(error.clone());
                    }
                }
                self.state
                    .set_status(format!("Clone failed: {error}"), StatusLevel::Error);
                error!("Clone failed for {}: {}", repo, error);
            }

            AppEvent::FetchCompleted { repo } => {
                self.state
                    .set_status(format!("Fetched {repo}"), StatusLevel::Success);
                self.state.checked_out =
                    scan_workspace(&self.state.config.workspace_root, &self.state.config.layout);
                save_workspace_cache(&self.state.checked_out);
            }

            AppEvent::FetchFailed { repo, error } => {
                self.state.set_status(
                    format!("Fetch failed for {repo}: {error}"),
                    StatusLevel::Error,
                );
                error!("Fetch failed for {}: {}", repo, error);
            }

            AppEvent::BatchFetchStarted => {
                self.state.batch_fetching = true;
                self.state
                    .set_status("Batch fetch running…", StatusLevel::Info);
            }

            AppEvent::BatchFetchCompleted { fetched, failed } => {
                self.state.batch_fetching = false;
                self.state.last_fetch_time = Some(chrono::Local::now());
                self.state.checked_out =
                    scan_workspace(&self.state.config.workspace_root, &self.state.config.layout);
                save_workspace_cache(&self.state.checked_out);
                let msg = if failed == 0 {
                    format!("Fetched {fetched} repos")
                } else {
                    format!("Fetched {fetched} repos, {failed} failed")
                };
                let level = if failed == 0 {
                    StatusLevel::Success
                } else {
                    StatusLevel::Error
                };
                self.state.set_status(msg, level);
                info!("Batch fetch done: {} ok, {} failed", fetched, failed);
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Ensure terminal is restored even on unexpected drops
        ratatui::restore();
    }
}
