use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let mut items: Vec<ListItem> = Vec::new();

    let scroll = app.settings_scroll;
    let end = (scroll + visible_height).min(app.doc.categories.len());

    for idx in scroll..end {
        let category = &app.doc.categories[idx];
        let is_selected = idx == app.settings_cursor;
        let project_count = category.projects.len();

        let style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let prefix = if is_selected { " ▸ " } else { "   " };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(category.name.clone(), style),
            Span::styled(
                format!("  ({} projects)", project_count),
                Style::default().fg(Color::DarkGray),
            ),
        ])));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No categories. Press 'a' to add one.",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Settings — Categories ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    let mut state = ListState::default();
    frame.render_stateful_widget(list, area, &mut state);
}
