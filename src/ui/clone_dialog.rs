use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};
use ratatui::Frame;

use crate::state::{CloneDialogState, CloneStage};

pub fn render(frame: &mut Frame, state: &CloneDialogState) {
    let area = centered_rect(60, 10, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Clone Repository ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &state.stage {
        CloneStage::Confirm => render_confirm(frame, inner, state),
        CloneStage::Cloning { progress } => render_progress(frame, inner, state, *progress),
        CloneStage::Done(path) => render_done(frame, inner, path.display().to_string()),
        CloneStage::Failed(err) => render_error(frame, inner, err),
    }
}

fn render_confirm(frame: &mut Frame, area: Rect, state: &CloneDialogState) {
    let repo = &state.repo;
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Clone "),
            Span::styled(
                &repo.full_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] ", Style::default().fg(Color::Green)),
            Span::raw("confirm   "),
            Span::styled("  [Esc] ", Style::default().fg(Color::Red)),
            Span::raw("cancel"),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_progress(frame: &mut Frame, area: Rect, state: &CloneDialogState, progress: f64) {
    let pct = (progress * 100.0) as u16;
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Cloning "),
            Span::styled(&state.repo.full_name, Style::default().fg(Color::Cyan)),
            Span::raw("…"),
        ]),
        Line::from(""),
    ];
    frame.render_widget(Paragraph::new(lines), area);

    let [_, gauge_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    let gauge_area = Rect {
        x: gauge_area.x + 2,
        width: gauge_area.width.saturating_sub(4),
        ..gauge_area
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(pct);
    frame.render_widget(gauge, gauge_area);
}

fn render_done(frame: &mut Frame, area: Rect, path: String) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✓ Cloned to ", Style::default().fg(Color::Green)),
            Span::styled(path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(Color::DarkGray)),
            Span::raw("close"),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_error(frame: &mut Frame, area: Rect, err: &str) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✗ Error: ", Style::default().fg(Color::Red)),
            Span::raw(err),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(Color::DarkGray)),
            Span::raw("close"),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let [_, vertical, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center)
    .areas(area);

    let [_, horizontal, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(percent_x),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center)
    .areas(vertical);

    horizontal
}
