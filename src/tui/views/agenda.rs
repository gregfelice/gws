use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::App;
use crate::model::TaskState;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let mut items: Vec<ListItem> = Vec::new();

    let scroll = app.agenda_scroll;
    let end = (scroll + visible_height).min(app.agenda_items.len());
    let is_moving = app.is_moving();

    for idx in scroll..end {
        let agenda_item = &app.agenda_items[idx];
        let is_selected = idx == app.agenda_cursor;
        let symbol_color = match agenda_item.task.state {
            TaskState::InProgress => Color::Rgb(255, 165, 0),
            TaskState::OnDeck => Color::Rgb(100, 149, 237),
            _ => Color::White,
        };

        let style = if is_selected && is_moving {
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let prefix = if is_selected && is_moving {
            " ↕ "
        } else if is_selected {
            " ▸ "
        } else {
            "   "
        };

        let prefix_style = if is_selected {
            if is_moving {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::Yellow)
            }
        } else {
            Style::default()
        };

        let project_label = format!(" ({})", agenda_item.project_name);

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix.to_string(), prefix_style),
            Span::styled(
                format!("{} ", agenda_item.task.state.symbol()),
                Style::default().fg(symbol_color),
            ),
            Span::styled(agenda_item.task.text.clone(), style),
            Span::styled(
                project_label,
                Style::default().fg(Color::DarkGray),
            ),
        ])));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No active tasks. Press Tab to go to Backlog.",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Agenda ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    let mut state = ListState::default();
    frame.render_stateful_widget(list, area, &mut state);
}
