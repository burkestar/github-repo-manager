use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Local, Utc};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub clone_url: String,
    pub stars: u32,
    pub archived: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutInfo {
    pub local_path: PathBuf,
    pub current_branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PanelFocus {
    OrgPanel,
    RepoPanel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusLevel {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortField {
    Name,
    UpdatedAt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CloneStage {
    Cloning { progress: f64 },
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct CloneDialogState {
    pub repo: RepoInfo,
    pub stage: CloneStage,
}

pub struct AppState {
    pub config: Config,
    pub focus: PanelFocus,
    pub orgs: Vec<String>,
    pub selected_org_idx: usize,
    pub repos: HashMap<String, Vec<RepoInfo>>,
    pub repos_loading: HashSet<String>,
    pub repo_list_state: ListState,
    pub org_list_state: ListState,
    pub search_active: bool,
    pub search_query: String,
    pub filtered_repos: Vec<usize>,
    pub checked_out: HashMap<String, CheckoutInfo>,
    pub clone_dialog: Option<CloneDialogState>,
    pub status_message: Option<(String, StatusLevel, Instant)>,
    pub last_fetch_time: Option<DateTime<Local>>,
    pub batch_fetching: bool,
    pub show_archived: bool,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
    pub error_popup: Option<String>,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let orgs = config.organizations.clone();
        let mut org_list_state = ListState::default();
        org_list_state.select(Some(0));
        let mut repo_list_state = ListState::default();
        repo_list_state.select(Some(0));
        Self {
            config,
            focus: PanelFocus::RepoPanel,
            orgs,
            selected_org_idx: 0,
            repos: HashMap::new(),
            repos_loading: HashSet::new(),
            repo_list_state,
            org_list_state,
            search_active: false,
            search_query: String::new(),
            filtered_repos: Vec::new(),
            checked_out: HashMap::new(),
            clone_dialog: None,
            status_message: None,
            last_fetch_time: None,
            batch_fetching: false,
            show_archived: true,
            sort_field: SortField::Name,
            sort_order: SortOrder::Asc,
            error_popup: None,
            should_quit: false,
        }
    }

    pub fn current_org(&self) -> &str {
        &self.orgs[self.selected_org_idx]
    }

    pub fn current_repos(&self) -> &[RepoInfo] {
        self.repos
            .get(self.current_org())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn selected_repo(&self) -> Option<&RepoInfo> {
        let repos = self.current_repos();
        if repos.is_empty() {
            return None;
        }
        let idx = self.repo_list_state.selected()?;
        let repo_idx = *self.filtered_repos.get(idx)?;
        repos.get(repo_idx)
    }

    pub fn set_status(&mut self, msg: impl Into<String>, level: StatusLevel) {
        self.status_message = Some((msg.into(), level, Instant::now()));
    }

    pub fn check_status_expiry(&mut self) {
        if let Some((_, _, set_at)) = &self.status_message {
            if set_at.elapsed().as_secs() >= 5 {
                self.status_message = None;
            }
        }
    }
}
