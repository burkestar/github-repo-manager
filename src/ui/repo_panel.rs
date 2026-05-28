use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::state::{AppState, PanelFocus, SortField, SortOrder};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let focused = state.focus == PanelFocus::RepoPanel;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let org = state.current_org().to_string();
    let is_loading = state.repos_loading.contains(&org);
    let total = state.current_repos().len();
    let filtered = state.filtered_repos.len();

    let archived_note = if !state.show_archived { " [archived hidden]" } else { "" };
    let sort_indicator = match (&state.sort_field, &state.sort_order) {
        (SortField::Name, SortOrder::Asc) => " ↑name",
        (SortField::Name, SortOrder::Desc) => " ↓name",
        (SortField::UpdatedAt, SortOrder::Asc) => " ↑updated",
        (SortField::UpdatedAt, SortOrder::Desc) => " ↓updated",
    };
    let title = if is_loading {
        format!(" {org} — loading… ")
    } else if state.search_active && !state.search_query.is_empty() {
        format!(" {org} — {filtered}/{total} repos{archived_note} ")
    } else {
        format!(" {org} — {filtered} repos{sort_indicator}{archived_note} ")
    };

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let [search_area, list_area] = Layout::vertical([
        Constraint::Length(if state.search_active { 1 } else { 0 }),
        Constraint::Fill(1),
    ])
    .areas(inner_area);

    if state.search_active {
        let search_text = format!(" Search: {}_", state.search_query);
        let search = Paragraph::new(search_text).style(Style::default().fg(Color::Yellow));
        frame.render_widget(search, search_area);
    }

    let repos = state.current_repos().to_vec();
    let checked_out = &state.checked_out;
    let filtered_indices = state.filtered_repos.clone();

    let items: Vec<ListItem> = filtered_indices
        .iter()
        .filter_map(|&idx| repos.get(idx))
        .map(|repo| {
            if let Some(info) = checked_out.get(&repo.full_name.to_lowercase()) {
                let branch = info.current_branch.as_deref().unwrap_or("?");
                let ahead_behind = if info.ahead > 0 || info.behind > 0 {
                    format!(" ↑{} ↓{}", info.ahead, info.behind)
                } else {
                    String::new()
                };
                let archived = if repo.archived { " [archived]" } else { "" };
                let line = Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("{:<36}", &repo.name),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("({branch}){ahead_behind}{archived}"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                ListItem::new(line)
            } else if repo.archived {
                let line = Line::from(vec![
                    Span::styled("⊙ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} [archived]", repo.name),
                        Style::default().fg(Color::Yellow),
                    ),
                ]);
                ListItem::new(line)
            } else {
                let line = Line::from(vec![Span::raw("○ "), Span::raw(&repo.name)]);
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, list_area, &mut state.repo_list_state);
}
