use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Dialog, View};
use crate::model::TreeNodeKind;

/// Action returned by input handling to tell the event loop what to do.
pub enum Action {
    None,
    Save,
    Reload,
    Quit,
}

/// Handle a key event, mutating app state and returning an action for the event loop.
pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Dialog handling takes priority
    if app.dialog != Dialog::None {
        return handle_dialog_input(app, key);
    }

    // Move mode takes priority over normal view keys
    if app.is_moving() {
        return handle_move_input(app, key);
    }

    match app.view {
        View::Agenda => handle_agenda_key(app, key),
        View::Backlog => handle_backlog_key(app, key),
        View::Settings => handle_settings_key(app, key),
    }
}

// --- Move mode ---

fn handle_move_input(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.move_step(1),
        KeyCode::Char('k') | KeyCode::Up => app.move_step(-1),
        KeyCode::Enter => app.accept_move(),
        KeyCode::Esc => app.cancel_move(),
        _ => {}
    }
    Action::None
}

// --- Global keys (shared across views) ---

fn handle_global_key(app: &mut App, key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::Quit)
        }
        KeyCode::Tab => {
            app.cycle_view();
            Some(Action::None)
        }
        KeyCode::Char('s') => Some(Action::Save),
        KeyCode::Char('R') => Some(Action::Reload),
        _ => None,
    }
}

// --- Agenda view ---

fn handle_agenda_key(app: &mut App, key: KeyEvent) -> Action {
    if let Some(action) = handle_global_key(app, &key) {
        return action;
    }

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('g') => app.move_top(),
        KeyCode::Char('G') => app.move_bottom(),
        KeyCode::Char('l') => app.center_cursor(app.visible_height),

        // Move mode
        KeyCode::Char('m') => app.start_move(),

        // Mutations
        KeyCode::Char('p') => app.promote_selected_agenda(),
        KeyCode::Char('x') => app.demote_selected_agenda(),
        KeyCode::Char('r') => app.run_auto_promote(),
        KeyCode::Char('A') => app.open_dialog(Dialog::ConfirmArchive),

        _ => {}
    }

    Action::None
}

// --- Backlog view ---

fn handle_backlog_key(app: &mut App, key: KeyEvent) -> Action {
    if let Some(action) = handle_global_key(app, &key) {
        return action;
    }

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('g') => app.move_top(),
        KeyCode::Char('G') => app.move_bottom(),
        KeyCode::Char('l') => app.center_cursor(app.visible_height),

        // Collapse/expand
        KeyCode::Char(' ') => app.toggle_collapse(),

        // Promote/demote
        KeyCode::Char('p') => app.promote_selected_backlog(),
        KeyCode::Char('x') => app.demote_selected_backlog(),

        // Move mode
        KeyCode::Char('m') => app.start_move(),

        // Add
        KeyCode::Char('a') => {
            if let Some(node) = app.current_tree_node() {
                match &node.kind {
                    TreeNodeKind::Category { .. } => {
                        app.open_dialog(Dialog::AddProject);
                    }
                    _ => {
                        app.open_dialog(Dialog::AddTask);
                    }
                }
            }
        }

        // Edit
        KeyCode::Char('e') => {
            if let Some(node) = app.current_tree_node() {
                match &node.kind {
                    TreeNodeKind::Task { .. } => {
                        let text = app.focused_edit_text();
                        app.open_dialog_with_text(Dialog::EditTask, &text);
                    }
                    TreeNodeKind::Project { .. } => {
                        let text = app.focused_edit_text();
                        app.open_dialog_with_text(Dialog::EditProject, &text);
                    }
                    TreeNodeKind::Category { .. } => {
                        let text = app.focused_edit_text();
                        app.open_dialog_with_text(Dialog::EditCategory, &text);
                    }
                    TreeNodeKind::Note { .. } => {
                        let text = app.focused_edit_text();
                        app.open_dialog_with_text(Dialog::EditExistingNote, &text);
                    }
                }
            }
        }

        // Delete
        KeyCode::Char('d') => {
            if let Some(node) = app.current_tree_node() {
                match &node.kind {
                    TreeNodeKind::Task { .. }
                    | TreeNodeKind::Project { .. }
                    | TreeNodeKind::Note { .. } => {
                        app.open_dialog(Dialog::ConfirmDelete);
                    }
                    _ => {}
                }
            }
        }

        // Add note
        KeyCode::Char('n') => {
            if let Some(node) = app.current_tree_node() {
                if matches!(&node.kind, TreeNodeKind::Task { .. }) {
                    app.open_dialog(Dialog::EditNote);
                }
            }
        }

        // Auto-promote & archive
        KeyCode::Char('r') => app.run_auto_promote(),
        KeyCode::Char('A') => app.open_dialog(Dialog::ConfirmArchive),

        _ => {}
    }

    Action::None
}

// --- Settings view ---

fn handle_settings_key(app: &mut App, key: KeyEvent) -> Action {
    if let Some(action) = handle_global_key(app, &key) {
        return action;
    }

    // Theme row: cursor == 0
    let on_theme_row = app.settings_cursor == 0;

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),

        // Theme cycling (h/l/arrows) when on theme row; l centers otherwise
        KeyCode::Char('h') | KeyCode::Left if on_theme_row => app.prev_theme(),
        KeyCode::Char('l') | KeyCode::Right if on_theme_row => app.next_theme(),
        KeyCode::Char('l') => app.center_cursor(app.visible_height),

        // Add category
        KeyCode::Char('a') => app.open_dialog(Dialog::AddCategory),

        // Rename category (only when on a category row)
        KeyCode::Char('e') => {
            if let Some(cat_idx) = app.settings_category_idx() {
                if let Some(cat) = app.doc.categories.get(cat_idx) {
                    let name = cat.name.clone();
                    app.open_dialog_with_text(Dialog::EditCategory, &name);
                }
            }
        }

        // Delete category (only when on a category row)
        KeyCode::Char('d') => {
            if app.settings_category_idx().is_some() && !app.doc.categories.is_empty() {
                app.open_dialog(Dialog::ConfirmDeleteCategory);
            }
        }

        // Move mode (only when on a category row)
        KeyCode::Char('m') => app.start_move(),

        _ => {}
    }

    Action::None
}

// --- Dialog input handling ---

fn handle_dialog_input(app: &mut App, key: KeyEvent) -> Action {
    match app.dialog {
        Dialog::ConfirmArchive => handle_confirm_input(app, key, |app| app.archive_done()),
        Dialog::ConfirmDelete => handle_confirm_input(app, key, |app| app.delete_focused()),
        Dialog::ConfirmDeleteCategory => handle_confirm_input(app, key, |app| app.delete_selected_category()),
        Dialog::AddTask => handle_text_input(app, key, |app| app.add_task_to_focused()),
        Dialog::AddProject => handle_text_input(app, key, |app| app.add_project_to_focused()),
        Dialog::EditTask | Dialog::EditProject => handle_text_input(app, key, |app| app.apply_edit()),
        Dialog::EditNote => handle_text_input(app, key, |app| app.add_note_to_focused()),
        Dialog::EditExistingNote => handle_text_input(app, key, |app| app.apply_edit()),
        Dialog::AddCategory => handle_text_input(app, key, |app| app.add_category_from_input()),
        Dialog::EditCategory => {
            if app.view == View::Settings {
                handle_text_input(app, key, |app| app.rename_category_from_input())
            } else {
                // Editing category name from backlog
                handle_text_input(app, key, |app| app.apply_edit())
            }
        }
        Dialog::None => Action::None,
    }
}

fn handle_text_input(app: &mut App, key: KeyEvent, on_confirm: fn(&mut App)) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.close_dialog();
        }
        KeyCode::Enter => {
            on_confirm(app);
            app.close_dialog();
        }
        KeyCode::Backspace => {
            app.input_backspace();
        }
        KeyCode::Delete => {
            app.input_delete();
        }
        KeyCode::Left => {
            app.input_move_left();
        }
        KeyCode::Right => {
            app.input_move_right();
        }
        KeyCode::Char(c) => {
            app.input_char(c);
        }
        _ => {}
    }
    Action::None
}

fn handle_confirm_input(app: &mut App, key: KeyEvent, on_confirm: fn(&mut App)) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            on_confirm(app);
            app.close_dialog();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.close_dialog();
        }
        _ => {}
    }
    Action::None
}
