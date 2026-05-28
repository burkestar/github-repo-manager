use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::state::{AppState, PanelFocus};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let focused = state.focus == PanelFocus::OrgPanel;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = state
        .orgs
        .iter()
        .enumerate()
        .map(|(i, org)| {
            let is_selected = i == state.selected_org_idx;
            let is_loading = state.repos_loading.contains(org.as_str());

            let prefix = if is_selected { "▶ " } else { "  " };
            let suffix = if is_loading { " …" } else { "" };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![Span::styled(
                format!("{prefix}{org}{suffix}"),
                style,
            )]))
        })
        .collect();

    let block = Block::default()
        .title(" Organizations ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);
    frame.render_stateful_widget(list, area, &mut state.org_list_state);
}
