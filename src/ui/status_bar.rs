use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{AppState, StatusLevel};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let workspace = state.config.workspace_root.display().to_string();
    let workspace_str = workspace.replace(
        &dirs::home_dir().unwrap_or_default().display().to_string(),
        "~",
    );

    let fetch_str = match &state.last_fetch_time {
        Some(t) => format!("Last fetch: {}", t.format("%Y-%m-%d %H:%M")),
        None => "No fetch yet".to_string(),
    };

    let batch_str = if state.batch_fetching {
        " [fetching…]".to_string()
    } else {
        String::new()
    };

    let status_part = if let Some((msg, level, _)) = &state.status_message {
        let color = match level {
            StatusLevel::Info => Color::White,
            StatusLevel::Success => Color::Green,
            StatusLevel::Error => Color::Red,
        };
        Span::styled(format!("  {msg}"), Style::default().fg(color))
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {workspace_str}"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{fetch_str}{batch_str}"),
            Style::default().fg(Color::DarkGray),
        ),
        status_part,
    ]);

    let para = Paragraph::new(line).style(Style::default().bg(Color::Reset));
    frame.render_widget(para, area);
}

pub fn render_title(frame: &mut Frame, area: Rect, _state: &AppState) {
    let line = Line::from(vec![
        Span::styled(
            " github-repo-manager ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " [Tab] switch  [/] search  [Enter] clone  [f] fetch  [F] fetch all  [m] upmain  [s] sort field  [S] sort order  [a] archived  [r] refresh  [q] quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
