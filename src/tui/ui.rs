use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, Dialog, View};
use crate::tui::views::{agenda, backlog, settings};
use crate::tui::widgets;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // header + tabs
        Constraint::Min(1),   // main content
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    // Update scroll before rendering so viewport tracks cursor
    let visible_height = chunks[1].height.saturating_sub(2) as usize; // borders
    app.update_scroll(visible_height);

    draw_header(frame, app, chunks[0]);

    match app.view {
        View::Agenda => agenda::draw(frame, app, chunks[1]),
        View::Backlog => backlog::draw(frame, app, chunks[1]),
        View::Settings => settings::draw(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Draw dialogs on top
    match app.dialog {
        Dialog::AddTask => widgets::draw_input_dialog(frame, app, "Add Task"),
        Dialog::AddProject => widgets::draw_input_dialog(frame, app, "Add Project"),
        Dialog::EditTask | Dialog::EditProject | Dialog::EditCategory | Dialog::EditExistingNote => {
            widgets::draw_input_dialog(frame, app, "Edit")
        }
        Dialog::EditNote => widgets::draw_input_dialog(frame, app, "Add Note"),
        Dialog::AddCategory => widgets::draw_input_dialog(frame, app, "Add Category"),
        Dialog::ConfirmArchive => widgets::draw_confirm_dialog(frame, "Archive all done tasks?"),
        Dialog::ConfirmDelete => widgets::draw_confirm_dialog(frame, "Delete this item?"),
        Dialog::ConfirmDeleteCategory => widgets::draw_confirm_dialog(frame, "Delete this category and all its projects?"),
        Dialog::None => {}
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec![" Agenda ", " Backlog ", " Settings "];
    let selected = match app.view {
        View::Agenda => 0,
        View::Backlog => 1,
        View::Settings => 2,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" GWS - Getting Work Sorted ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .select(selected)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let dirty_indicator = if app.dirty { " [modified]" } else { "" };
    let status = if app.status_msg.is_empty() {
        String::new()
    } else {
        format!(" {} ", app.status_msg)
    };

    let help = if app.is_moving() {
        "j/k:Move  Enter:Accept  Esc:Cancel"
    } else {
        match app.dialog {
            Dialog::None => match app.view {
                View::Agenda => "q:Quit  Tab:View  j/k:Nav  l:Center  m:Move  p:Promote  x:Demote  r:Auto  A:Archive  s:Save",
                View::Backlog => "q:Quit  Tab:View  j/k:Nav  l:Center  Space:Fold  p/x:Cycle  a:Add  e:Edit  d:Del  m:Move  n:Note  s:Save",
                View::Settings => "q:Quit  Tab:View  j/k:Nav  l:Center  a:Add  e:Rename  d:Del  m:Move  s:Save",
            },
            Dialog::ConfirmArchive | Dialog::ConfirmDelete | Dialog::ConfirmDeleteCategory => {
                "y:Yes  n/Esc:No"
            }
            _ => "Enter:Confirm  Esc:Cancel",
        }
    };

    let line = Line::from(vec![
        Span::styled(
            &status,
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            dirty_indicator,
            Style::default().fg(Color::Red),
        ),
        Span::raw("  "),
        Span::styled(
            help,
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);
}
