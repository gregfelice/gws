use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Draw a centered popup area.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

pub fn draw_input_dialog(frame: &mut Frame, app: &App, title: &str) {
    let theme = app.theme();
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    // Show cursor in input
    let display_text = if app.input_buffer.is_empty() {
        String::from("Type task name...")
    } else {
        app.input_buffer.clone()
    };

    let style = if app.input_buffer.is_empty() {
        Style::default().fg(theme.dialog_placeholder)
    } else {
        Style::default().fg(theme.dialog_text)
    };

    let input = Paragraph::new(Line::from(Span::styled(&display_text, style))).block(
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.dialog_border)),
    );

    frame.render_widget(input, area);

    // Position cursor
    let cursor_x = area.x + 1 + app.input_cursor as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

pub fn draw_confirm_dialog(frame: &mut Frame, app: &App, message: &str) {
    let theme = app.theme();
    let area = centered_rect(40, 5, frame.area());
    frame.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            message,
            Style::default()
                .fg(theme.dialog_text)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  y: Yes   n: No",
            Style::default().fg(theme.text),
        )),
    ];

    let dialog = Paragraph::new(text).block(
        Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.dialog_border)),
    );

    frame.render_widget(dialog, area);
}
