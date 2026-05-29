use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_info(frame: &mut Frame, title: &str, body: &str) {
    let area = popup_rect(70, 16, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {title} "))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [msg_area, footer_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);

    let para = Paragraph::new(format!(" {}", body.replace('\n', "\n ")))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, msg_area);

    let footer = Line::from(vec![
        Span::styled(" [any key] ", Style::default().fg(Color::DarkGray)),
        Span::raw("close"),
    ]);
    frame.render_widget(Paragraph::new(footer), footer_area);
}

pub fn render(frame: &mut Frame, message: &str) {
    let area = popup_rect(70, 16, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Error ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [msg_area, footer_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);

    let para = Paragraph::new(format!(" {}", message.replace('\n', "\n ")))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, msg_area);

    let footer = Line::from(vec![
        Span::styled(" [any key] ", Style::default().fg(Color::DarkGray)),
        Span::raw("close"),
    ]);
    frame.render_widget(Paragraph::new(footer), footer_area);
}

fn popup_rect(percent_x: u16, max_height: u16, area: Rect) -> Rect {
    let height = max_height.min(area.height.saturating_sub(4));

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
