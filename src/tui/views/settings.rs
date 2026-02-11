use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let mut items: Vec<ListItem> = Vec::new();

    let total = app.settings_total();
    let scroll = app.settings_scroll;
    let end = (scroll + visible_height).min(total);

    for idx in scroll..end {
        let is_selected = idx == app.settings_cursor;

        if idx == 0 {
            // Theme row
            let theme_name = Theme::all()[app.theme_index].name;
            let prefix = if is_selected { " ▸ " } else { "   " };
            let prefix_style = if is_selected {
                Style::default()
                    .fg(theme.cursor)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let label_style = if is_selected {
                Style::default()
                    .fg(theme.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled("Theme:  ", label_style),
                Span::styled("◀ ", Style::default().fg(theme.text_dim)),
                Span::styled(theme_name, Style::default().fg(theme.tab_active).add_modifier(Modifier::BOLD)),
                Span::styled(" ▶", Style::default().fg(theme.text_dim)),
            ])));
        } else {
            // Category row (idx - 1 is the category index)
            let cat_idx = idx - 1;
            let category = &app.doc.categories[cat_idx];
            let project_count = category.projects.len();
            let is_moving = app.is_moving();

            let style = if is_selected && is_moving {
                Style::default()
                    .fg(theme.moving)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(theme.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let prefix = if is_selected && is_moving {
                " ↕ "
            } else if is_selected {
                " ▸ "
            } else {
                "   "
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(category.name.clone(), style),
                Span::styled(
                    format!("  ({} projects)", project_count),
                    Style::default().fg(theme.text_dim),
                ),
            ])));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No categories. Press 'a' to add one.",
            Style::default().fg(theme.text_dim),
        ))));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    let mut state = ListState::default();
    frame.render_stateful_widget(list, area, &mut state);
}
