use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::App;
use crate::model::TaskState;

fn section_label(state: TaskState) -> &'static str {
    match state {
        TaskState::Todo => "Not Started",
        TaskState::InProgress => "In Progress",
        TaskState::OnDeck => "On Deck",
        TaskState::Done => "Done",
    }
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let mut items: Vec<ListItem> = Vec::new();

    let scroll = app.agenda_scroll;
    let is_moving = app.is_moving();

    if app.agenda_items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No active tasks. Press Tab to go to Backlog.",
            Style::default().fg(theme.text_dim),
        ))));
    } else {
        let mut rows_used = 0;
        let mut item_idx = scroll;
        let mut prev_state: Option<TaskState> = None;

        while item_idx < app.agenda_items.len() && rows_used < visible_height {
            let agenda_item = &app.agenda_items[item_idx];
            let current_state = agenda_item.task.state;

            // Render section header if state changed (or first visible item)
            if prev_state.map_or(true, |prev| prev != current_state) {
                let label = section_label(current_state);
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("  ── {} ──", label),
                    Style::default()
                        .fg(theme.text_dim)
                        .add_modifier(Modifier::BOLD),
                ))));
                rows_used += 1;
                prev_state = Some(current_state);

                if rows_used >= visible_height {
                    break;
                }
            }

            // Render the task item
            let is_selected = item_idx == app.agenda_cursor;
            let dot_color = match agenda_item.task.state {
                TaskState::Todo => theme.state_todo,
                TaskState::OnDeck => theme.state_ondeck,
                TaskState::InProgress => theme.state_inprogress,
                TaskState::Done => theme.state_done,
            };

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

            let prefix_style = if is_selected {
                if is_moving {
                    Style::default().fg(theme.moving)
                } else {
                    Style::default().fg(theme.cursor)
                }
            } else {
                Style::default()
            };

            let project_label = format!(" ({})", agenda_item.project_name);

            items.push(ListItem::new(Line::from(vec![
                Span::styled(prefix.to_string(), prefix_style),
                Span::styled(
                    format!("{} ", agenda_item.task.state.dot()),
                    Style::default().fg(dot_color),
                ),
                Span::styled(agenda_item.task.text.clone(), style),
                Span::styled(project_label, Style::default().fg(theme.text_dim)),
            ])));
            rows_used += 1;
            item_idx += 1;
        }
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Agenda ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    let mut state = ListState::default();
    frame.render_stateful_widget(list, area, &mut state);
}
