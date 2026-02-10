use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::App;
use crate::model::{TaskState, TreeNodeKind};

fn dot_color(state: TaskState) -> Color {
    match state {
        TaskState::Todo => Color::Red,
        TaskState::OnDeck => Color::Rgb(100, 149, 237),
        TaskState::InProgress => Color::Yellow,
        TaskState::Done => Color::Green,
    }
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let mut items: Vec<ListItem> = Vec::new();

    let scroll = app.backlog_scroll;
    let end = (scroll + visible_height).min(app.tree_nodes.len());

    let is_moving = app.is_moving();

    for idx in scroll..end {
        let node = &app.tree_nodes[idx];
        let is_selected = idx == app.backlog_cursor;

        let indent = "    ".repeat(node.depth as usize);

        // Determine task state dot for task nodes
        let task_state = match &node.kind {
            TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                app.doc.categories.get(*cat_idx)
                    .and_then(|c| c.projects.get(*proj_idx))
                    .and_then(|p| p.tasks.get(*task_idx))
                    .map(|t| t.state)
            }
            _ => None,
        };

        let (line, style) = if is_selected && is_moving {
            let style = Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD);
            (node.display.clone(), style)
        } else {
            match &node.kind {
                TreeNodeKind::Category { .. } => {
                    let style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    (node.display.clone(), style)
                }
                TreeNodeKind::Project { .. } => {
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    (node.display.clone(), style)
                }
                TreeNodeKind::Task { .. } => {
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    (node.display.clone(), style)
                }
                TreeNodeKind::Note { .. } => {
                    let style = Style::default().fg(Color::DarkGray);
                    (node.display.clone(), style)
                }
            }
        };

        let prefix = if is_selected && is_moving {
            "↕"
        } else if is_selected {
            "▸"
        } else {
            " "
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

        let mut spans = vec![
            Span::styled(prefix.to_string(), prefix_style),
            Span::styled(indent, Style::default()),
        ];

        // Add colored dot for task nodes
        if let Some(state) = task_state {
            spans.push(Span::styled(
                format!("{} ", state.dot()),
                Style::default().fg(dot_color(state)),
            ));
        }

        spans.push(Span::styled(line, style));

        items.push(ListItem::new(Line::from(spans)));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No categories. Press 'a' to add one.",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Backlog ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    let mut state = ListState::default();
    frame.render_stateful_widget(list, area, &mut state);
}
