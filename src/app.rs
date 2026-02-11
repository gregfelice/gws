use std::path::PathBuf;

use crate::engine;
use crate::model::*;
use crate::parser;
use crate::serializer;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Agenda,
    Backlog,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    None,
    AddTask,
    AddProject,
    EditTask,
    EditProject,
    EditNote,
    EditExistingNote,
    ConfirmDelete,
    ConfirmArchive,
    AddCategory,
    EditCategory,
    ConfirmDeleteCategory,
}

/// Tracks what kind of item is being moved and where it started.
#[derive(Debug, Clone)]
pub enum MoveKind {
    Task { cat_idx: usize, proj_idx: usize, original_task_idx: usize },
    Project { original_cat_idx: usize, original_proj_idx: usize },
    Category { original_cat_idx: usize },
    AgendaItem { original_idx: usize },
}

pub struct App {
    pub doc: Document,
    pub file_path: PathBuf,
    pub view: View,
    pub dialog: Dialog,
    pub dirty: bool,
    pub running: bool,
    pub status_msg: String,

    // Agenda view state
    pub agenda_items: Vec<AgendaItem>,
    pub agenda_cursor: usize,
    pub agenda_scroll: usize,

    // Backlog tree state
    pub tree_nodes: Vec<TreeNode>,
    pub backlog_cursor: usize,
    pub backlog_scroll: usize,
    pub collapse: CollapseState,

    // Settings state
    pub settings_cursor: usize,
    pub settings_scroll: usize,

    // Theme
    pub theme_index: usize,

    // Move mode
    pub moving: Option<MoveKind>,

    // Last known visible height (updated each frame)
    pub visible_height: usize,

    // Dialog state
    pub input_buffer: String,
    pub input_cursor: usize,
}

impl App {
    pub fn new(mut doc: Document, file_path: PathBuf) -> Self {
        engine::auto_promote(&mut doc);
        let agenda_items = engine::build_agenda(&doc);
        let mut app = Self {
            doc,
            file_path,
            view: View::Agenda,
            dialog: Dialog::None,
            dirty: false,
            running: true,
            status_msg: String::new(),
            agenda_items,
            agenda_cursor: 0,
            agenda_scroll: 0,
            tree_nodes: Vec::new(),
            backlog_cursor: 0,
            backlog_scroll: 0,
            collapse: CollapseState::new(),
            settings_cursor: 0,
            settings_scroll: 0,
            theme_index: 0,
            moving: None,
            visible_height: 0,
            input_buffer: String::new(),
            input_cursor: 0,
        };
        app.rebuild_tree();
        app
    }

    // --- Tree building ---

    pub fn rebuild_tree(&mut self) {
        let mut nodes = Vec::new();

        for (cat_idx, category) in self.doc.categories.iter().enumerate() {
            let cat_collapsed = self.collapse.collapsed_categories.contains(&cat_idx);
            let indicator = if cat_collapsed { "►" } else { "▼" };
            nodes.push(TreeNode {
                kind: TreeNodeKind::Category { cat_idx },
                depth: 0,
                display: format!("{} {}", indicator, category.name),
            });

            if cat_collapsed {
                continue;
            }

            for (proj_idx, project) in category.projects.iter().enumerate() {
                let proj_collapsed = self.collapse.collapsed_projects.contains(&(cat_idx, proj_idx));
                let indicator = if proj_collapsed { "►" } else { "▼" };
                let active_marker = if project.active { "🔶 " } else { "" };
                nodes.push(TreeNode {
                    kind: TreeNodeKind::Project { cat_idx, proj_idx },
                    depth: 1,
                    display: format!("{} {}{}", indicator, active_marker, project.name),
                });

                if proj_collapsed {
                    continue;
                }

                for (task_idx, task) in project.tasks.iter().enumerate() {
                    let has_notes = !task.notes.is_empty();
                    let task_collapsed = self.collapse.collapsed_tasks.contains(&(cat_idx, proj_idx, task_idx));

                    nodes.push(TreeNode {
                        kind: TreeNodeKind::Task { cat_idx, proj_idx, task_idx },
                        depth: 2,
                        display: task.text.clone(),
                    });

                    if has_notes && !task_collapsed {
                        for (note_idx, note) in task.notes.iter().enumerate() {
                            nodes.push(TreeNode {
                                kind: TreeNodeKind::Note { cat_idx, proj_idx, task_idx, note_idx },
                                depth: 3,
                                display: note.trim().to_string(),
                            });
                        }
                    }
                }
            }
        }

        self.tree_nodes = nodes;

        // Clamp cursor
        if !self.tree_nodes.is_empty() {
            if self.backlog_cursor >= self.tree_nodes.len() {
                self.backlog_cursor = self.tree_nodes.len() - 1;
            }
        } else {
            self.backlog_cursor = 0;
        }
    }

    // --- Agenda ---

    pub fn refresh_agenda(&mut self) {
        engine::auto_promote(&mut self.doc);
        self.agenda_items = engine::build_agenda(&self.doc);
        if !self.agenda_items.is_empty() {
            if self.agenda_cursor >= self.agenda_items.len() {
                self.agenda_cursor = self.agenda_items.len() - 1;
            }
        } else {
            self.agenda_cursor = 0;
        }
    }


    /// Total number of rows in the Settings view (1 theme row + categories).
    pub fn settings_total(&self) -> usize {
        1 + self.doc.categories.len()
    }

    /// Index of the category in doc.categories for the current settings_cursor,
    /// or None if the cursor is on the theme row.
    pub fn settings_category_idx(&self) -> Option<usize> {
        if self.settings_cursor == 0 {
            None
        } else {
            Some(self.settings_cursor - 1)
        }
    }

    // --- Navigation ---

    pub fn move_down(&mut self) {
        match self.view {
            View::Agenda => {
                if !self.agenda_items.is_empty() {
                    if self.agenda_cursor < self.agenda_items.len() - 1 {
                        self.agenda_cursor += 1;
                    } else {
                        self.agenda_cursor = 0;
                    }
                }
            }
            View::Backlog => {
                if !self.tree_nodes.is_empty() {
                    if self.backlog_cursor < self.tree_nodes.len() - 1 {
                        self.backlog_cursor += 1;
                    } else {
                        self.backlog_cursor = 0;
                    }
                }
            }
            View::Settings => {
                let total = self.settings_total();
                if total > 0 {
                    if self.settings_cursor < total - 1 {
                        self.settings_cursor += 1;
                    } else {
                        self.settings_cursor = 0;
                    }
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        match self.view {
            View::Agenda => {
                if !self.agenda_items.is_empty() {
                    if self.agenda_cursor > 0 {
                        self.agenda_cursor -= 1;
                    } else {
                        self.agenda_cursor = self.agenda_items.len() - 1;
                    }
                }
            }
            View::Backlog => {
                if !self.tree_nodes.is_empty() {
                    if self.backlog_cursor > 0 {
                        self.backlog_cursor -= 1;
                    } else {
                        self.backlog_cursor = self.tree_nodes.len() - 1;
                    }
                }
            }
            View::Settings => {
                let total = self.settings_total();
                if total > 0 {
                    if self.settings_cursor > 0 {
                        self.settings_cursor -= 1;
                    } else {
                        self.settings_cursor = total - 1;
                    }
                }
            }
        }
    }

    pub fn move_top(&mut self) {
        match self.view {
            View::Agenda => self.agenda_cursor = 0,
            View::Backlog => self.backlog_cursor = 0,
            View::Settings => self.settings_cursor = 0,
        }
    }

    pub fn move_bottom(&mut self) {
        match self.view {
            View::Agenda => {
                if !self.agenda_items.is_empty() {
                    self.agenda_cursor = self.agenda_items.len() - 1;
                }
            }
            View::Backlog => {
                if !self.tree_nodes.is_empty() {
                    self.backlog_cursor = self.tree_nodes.len() - 1;
                }
            }
            View::Settings => {
                let total = self.settings_total();
                if total > 0 {
                    self.settings_cursor = total - 1;
                }
            }
        }
    }

    /// Update scroll offset to keep cursor visible for the given view height.
    pub fn update_scroll(&mut self, visible_height: usize) {
        self.visible_height = visible_height;
        let settings_total = self.settings_total();
        let (cursor, scroll, len) = match self.view {
            View::Agenda => (self.agenda_cursor, &mut self.agenda_scroll, self.agenda_items.len()),
            View::Backlog => (self.backlog_cursor, &mut self.backlog_scroll, self.tree_nodes.len()),
            View::Settings => (self.settings_cursor, &mut self.settings_scroll, settings_total),
        };
        if len == 0 || visible_height == 0 {
            *scroll = 0;
            return;
        }
        if cursor >= *scroll + visible_height {
            *scroll = cursor - visible_height + 1;
        } else if cursor < *scroll {
            *scroll = cursor;
        }
    }

    /// Center the cursor vertically in the viewport.
    pub fn center_cursor(&mut self, visible_height: usize) {
        let (cursor, scroll) = match self.view {
            View::Agenda => (self.agenda_cursor, &mut self.agenda_scroll),
            View::Backlog => (self.backlog_cursor, &mut self.backlog_scroll),
            View::Settings => (self.settings_cursor, &mut self.settings_scroll),
        };
        *scroll = cursor.saturating_sub(visible_height / 2);
    }

    pub fn cycle_view(&mut self) {
        self.view = match self.view {
            View::Agenda => View::Backlog,
            View::Backlog => View::Settings,
            View::Settings => View::Agenda,
        };
    }

    /// Switch from Agenda to Backlog view, focusing on the same task.
    pub fn jump_to_backlog_task(&mut self) {
        let Some(item) = self.agenda_items.get(self.agenda_cursor) else {
            return;
        };
        let cat_idx = item.category_idx;
        let proj_idx = item.project_idx;
        let task_idx = item.task_idx;

        // Ensure parent category and project are expanded so the task is visible
        self.collapse.collapsed_categories.remove(&cat_idx);
        self.collapse.collapsed_projects.remove(&(cat_idx, proj_idx));
        self.rebuild_tree();

        // Find the matching task node in the tree
        let target = TreeNodeKind::Task { cat_idx, proj_idx, task_idx };
        for (i, node) in self.tree_nodes.iter().enumerate() {
            if node.kind == target {
                self.backlog_cursor = i;
                break;
            }
        }

        self.view = View::Backlog;
        self.update_scroll(self.visible_height);
    }

    // --- Backlog: toggle collapse ---

    pub fn toggle_collapse(&mut self) {
        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            match &node.kind {
                TreeNodeKind::Category { cat_idx } => {
                    let cat_idx = *cat_idx;
                    if !self.collapse.collapsed_categories.remove(&cat_idx) {
                        self.collapse.collapsed_categories.insert(cat_idx);
                    }
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    let key = (*cat_idx, *proj_idx);
                    if !self.collapse.collapsed_projects.remove(&key) {
                        self.collapse.collapsed_projects.insert(key);
                    }
                }
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    let key = (*cat_idx, *proj_idx, *task_idx);
                    if !self.collapse.collapsed_tasks.remove(&key) {
                        self.collapse.collapsed_tasks.insert(key);
                    }
                }
                TreeNodeKind::Note { .. } => {} // notes can't collapse
            }
            // Save the kind before rebuild so we can restore cursor
            let saved_kind = node.kind.clone();
            self.rebuild_tree();
            self.restore_cursor(&saved_kind);
        }
    }

    /// Restore cursor to the node matching the given kind after a rebuild.
    fn restore_cursor(&mut self, kind: &TreeNodeKind) {
        for (i, node) in self.tree_nodes.iter().enumerate() {
            if node.kind == *kind {
                self.backlog_cursor = i;
                return;
            }
        }
        // Fallback: clamp
        if !self.tree_nodes.is_empty() && self.backlog_cursor >= self.tree_nodes.len() {
            self.backlog_cursor = self.tree_nodes.len() - 1;
        }
    }

    // --- Backlog: current node ---

    pub fn current_tree_node(&self) -> Option<&TreeNode> {
        self.tree_nodes.get(self.backlog_cursor)
    }

    // --- Mutations from Agenda view ---

    pub fn promote_selected_agenda(&mut self) {
        if let Some(item) = self.agenda_items.get(self.agenda_cursor) {
            let ci = item.category_idx;
            let pi = item.project_idx;
            let ti = item.task_idx;
            if engine::promote_task(&mut self.doc, ci, pi, ti) {
                self.dirty = true;
                // Update the agenda item in-place to reflect new state
                let new_task = self.doc.categories[ci].projects[pi].tasks[ti].clone();
                self.agenda_items[self.agenda_cursor].task = new_task;
                self.status_msg = "Task promoted".to_string();
                self.rebuild_tree();
            }
        }
    }

    pub fn demote_selected_agenda(&mut self) {
        if let Some(item) = self.agenda_items.get(self.agenda_cursor) {
            let ci = item.category_idx;
            let pi = item.project_idx;
            let ti = item.task_idx;
            if engine::demote_task(&mut self.doc, ci, pi, ti) {
                self.dirty = true;
                // Update the agenda item in-place to reflect new state
                let new_task = self.doc.categories[ci].projects[pi].tasks[ti].clone();
                self.agenda_items[self.agenda_cursor].task = new_task;
                self.status_msg = "Task demoted".to_string();
                self.rebuild_tree();
            }
        }
    }

    // --- Mutations from Backlog view ---

    pub fn promote_selected_backlog(&mut self) {
        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            let saved_kind = node.kind.clone();
            match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    if engine::promote_task(&mut self.doc, *cat_idx, *proj_idx, *task_idx) {
                        self.dirty = true;
                        self.status_msg = "Task promoted".to_string();
                    }
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    if engine::toggle_project_active(&mut self.doc, *cat_idx, *proj_idx) {
                        self.dirty = true;
                        let active = self.doc.categories[*cat_idx].projects[*proj_idx].active;
                        self.status_msg = if active { "Project activated".to_string() } else { "Project deactivated".to_string() };
                    }
                }
                _ => {}
            }
            self.refresh_agenda();
            self.rebuild_tree();
            self.restore_cursor(&saved_kind);
        }
    }

    pub fn demote_selected_backlog(&mut self) {
        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            let saved_kind = node.kind.clone();
            match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    if engine::demote_task(&mut self.doc, *cat_idx, *proj_idx, *task_idx) {
                        self.dirty = true;
                        self.status_msg = "Task demoted".to_string();
                    }
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    if engine::toggle_project_active(&mut self.doc, *cat_idx, *proj_idx) {
                        self.dirty = true;
                        let active = self.doc.categories[*cat_idx].projects[*proj_idx].active;
                        self.status_msg = if active { "Project activated".to_string() } else { "Project deactivated".to_string() };
                    }
                }
                _ => {}
            }
            self.refresh_agenda();
            self.rebuild_tree();
            self.restore_cursor(&saved_kind);
        }
    }

    // --- Global mutations ---

    pub fn run_auto_promote(&mut self) {
        engine::auto_promote(&mut self.doc);
        self.dirty = true;
        self.status_msg = "Auto-promote complete".to_string();
        self.refresh_agenda();
        self.rebuild_tree();
    }

    pub fn archive_done(&mut self) {
        engine::archive_done(&mut self.doc);
        self.dirty = true;
        self.status_msg = "Done tasks archived".to_string();
        self.refresh_agenda();
        self.rebuild_tree();
    }

    // --- Backlog: add task ---

    pub fn add_task_to_focused(&mut self) {
        let text = self.input_buffer.trim().to_string();
        if text.is_empty() {
            return;
        }

        // Figure out where to add based on current backlog focus
        let (cat_idx, proj_idx) = if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            match &node.kind {
                TreeNodeKind::Category { cat_idx: _ } => {
                    // Adding a task to category doesn't make sense; this should be add project
                    return;
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => (*cat_idx, *proj_idx),
                TreeNodeKind::Task { cat_idx, proj_idx, .. } => (*cat_idx, *proj_idx),
                TreeNodeKind::Note { cat_idx, proj_idx, .. } => (*cat_idx, *proj_idx),
            }
        } else {
            return;
        };

        if engine::add_task(&mut self.doc, cat_idx, proj_idx, text) {
            self.dirty = true;
            self.status_msg = "Task added".to_string();
            self.refresh_agenda();
            self.rebuild_tree();
        }
    }

    pub fn add_project_to_focused(&mut self) {
        let name = self.input_buffer.trim().to_string();
        if name.is_empty() {
            return;
        }

        let cat_idx = if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            match &node.kind {
                TreeNodeKind::Category { cat_idx } => *cat_idx,
                TreeNodeKind::Project { cat_idx, .. } => *cat_idx,
                TreeNodeKind::Task { cat_idx, .. } => *cat_idx,
                TreeNodeKind::Note { cat_idx, .. } => *cat_idx,
            }
        } else {
            return;
        };

        if engine::add_project(&mut self.doc, cat_idx, name, true) {
            self.dirty = true;
            self.status_msg = "Project added".to_string();
            self.refresh_agenda();
            self.rebuild_tree();
        }
    }

    // --- Backlog: edit ---

    pub fn apply_edit(&mut self) {
        let new_text = self.input_buffer.trim().to_string();
        if new_text.is_empty() {
            return;
        }

        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            let saved_kind = node.kind.clone();
            match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    if engine::rename_task(&mut self.doc, *cat_idx, *proj_idx, *task_idx, new_text) {
                        self.dirty = true;
                        self.status_msg = "Task renamed".to_string();
                    }
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    if engine::rename_project(&mut self.doc, *cat_idx, *proj_idx, new_text) {
                        self.dirty = true;
                        self.status_msg = "Project renamed".to_string();
                    }
                }
                TreeNodeKind::Category { cat_idx } => {
                    if engine::rename_category(&mut self.doc, *cat_idx, new_text) {
                        self.dirty = true;
                        self.status_msg = "Category renamed".to_string();
                    }
                }
                TreeNodeKind::Note { cat_idx, proj_idx, task_idx, note_idx } => {
                    if let Some(note) = self.doc.categories
                        .get_mut(*cat_idx)
                        .and_then(|c| c.projects.get_mut(*proj_idx))
                        .and_then(|p| p.tasks.get_mut(*task_idx))
                        .and_then(|t| t.notes.get_mut(*note_idx))
                    {
                        *note = format!("  {}", new_text);
                        self.dirty = true;
                        self.status_msg = "Note updated".to_string();
                    }
                }
            }
            self.refresh_agenda();
            self.rebuild_tree();
            self.restore_cursor(&saved_kind);
        }
    }

    // --- Backlog: delete ---

    pub fn delete_focused(&mut self) {
        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    engine::delete_task(&mut self.doc, *cat_idx, *proj_idx, *task_idx);
                    self.dirty = true;
                    self.status_msg = "Task deleted".to_string();
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    engine::delete_project(&mut self.doc, *cat_idx, *proj_idx);
                    self.dirty = true;
                    self.status_msg = "Project deleted".to_string();
                }
                TreeNodeKind::Note { cat_idx, proj_idx, task_idx, note_idx } => {
                    engine::delete_task_note(&mut self.doc, *cat_idx, *proj_idx, *task_idx, *note_idx);
                    self.dirty = true;
                    self.status_msg = "Note deleted".to_string();
                }
                _ => {}
            }
            self.refresh_agenda();
            self.rebuild_tree();
        }
    }

    // --- Backlog: rerank ---

    pub fn rerank_focused(&mut self, direction: i32) {
        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            let new_kind = match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    if let Some(new_idx) = engine::rerank_task(&mut self.doc, *cat_idx, *proj_idx, *task_idx, direction) {
                        self.dirty = true;
                        Some(TreeNodeKind::Task { cat_idx: *cat_idx, proj_idx: *proj_idx, task_idx: new_idx })
                    } else {
                        None
                    }
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    if let Some(new_idx) = engine::rerank_project(&mut self.doc, *cat_idx, *proj_idx, direction) {
                        self.dirty = true;
                        Some(TreeNodeKind::Project { cat_idx: *cat_idx, proj_idx: new_idx })
                    } else {
                        // Move across categories
                        let ci = *cat_idx;
                        let pi = *proj_idx;
                        if direction < 0 && ci > 0 {
                            // Move to end of previous category
                            let dest_len = self.doc.categories[ci - 1].projects.len();
                            if let Some((new_ci, new_pi)) = engine::move_project_to_category(&mut self.doc, ci, pi, ci - 1, dest_len) {
                                self.dirty = true;
                                Some(TreeNodeKind::Project { cat_idx: new_ci, proj_idx: new_pi })
                            } else {
                                None
                            }
                        } else if direction > 0 && ci + 1 < self.doc.categories.len() {
                            // Move to start of next category
                            if let Some((new_ci, new_pi)) = engine::move_project_to_category(&mut self.doc, ci, pi, ci + 1, 0) {
                                self.dirty = true;
                                Some(TreeNodeKind::Project { cat_idx: new_ci, proj_idx: new_pi })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                _ => None,
            };

            if let Some(kind) = new_kind {
                self.refresh_agenda();
                self.rebuild_tree();
                self.restore_cursor(&kind);
            }
        }
    }

    // --- Move mode ---

    /// Enter move mode for the focused item in backlog or settings.
    pub fn start_move(&mut self) {
        match self.view {
            View::Backlog => {
                if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
                    let kind = match &node.kind {
                        TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                            Some(MoveKind::Task {
                                cat_idx: *cat_idx,
                                proj_idx: *proj_idx,
                                original_task_idx: *task_idx,
                            })
                        }
                        TreeNodeKind::Project { cat_idx, proj_idx } => {
                            Some(MoveKind::Project {
                                original_cat_idx: *cat_idx,
                                original_proj_idx: *proj_idx,
                            })
                        }
                        _ => None,
                    };
                    if let Some(k) = kind {
                        self.moving = Some(k);
                        self.status_msg = "Moving... j/k to reorder, Enter to accept, Esc to cancel".to_string();
                    }
                }
            }
            View::Settings => {
                if let Some(cat_idx) = self.settings_category_idx() {
                    if cat_idx < self.doc.categories.len() {
                        self.moving = Some(MoveKind::Category {
                            original_cat_idx: cat_idx,
                        });
                        self.status_msg = "Moving... j/k to reorder, Enter to accept, Esc to cancel".to_string();
                    }
                }
            }
            View::Agenda => {
                if !self.agenda_items.is_empty() {
                    self.moving = Some(MoveKind::AgendaItem {
                        original_idx: self.agenda_cursor,
                    });
                    self.status_msg = "Moving... j/k to reorder, Enter to accept, Esc to cancel".to_string();
                }
            }
        }
    }

    /// Move the item one step in the given direction during move mode.
    pub fn move_step(&mut self, direction: i32) {
        match self.view {
            View::Agenda => self.rerank_agenda(direction),
            View::Backlog => self.rerank_focused(direction),
            View::Settings => self.rerank_category(direction),
        }
    }

    /// Rerank the selected agenda item.
    fn rerank_agenda(&mut self, direction: i32) {
        if self.agenda_items.is_empty() {
            return;
        }
        let cur = self.agenda_cursor;
        let new_idx = cur as i32 + direction;
        if new_idx < 0 || new_idx >= self.agenda_items.len() as i32 {
            return;
        }
        let new_idx = new_idx as usize;
        self.agenda_items.swap(cur, new_idx);
        self.agenda_cursor = new_idx;
    }

    /// Accept the current move (just exit move mode, changes already applied).
    pub fn accept_move(&mut self) {
        if let Some(ref kind) = self.moving {
            let is_agenda = matches!(kind, MoveKind::AgendaItem { .. });
            self.moving = None;
            self.dirty = true;
            self.status_msg = "Moved".to_string();
            if !is_agenda {
                self.refresh_agenda();
            }
        }
    }

    /// Cancel the move and revert to the original position.
    pub fn cancel_move(&mut self) {
        let Some(move_kind) = self.moving.take() else { return };

        match move_kind {
            MoveKind::Task { cat_idx, proj_idx, original_task_idx } => {
                // Find current position of the task from the tree cursor
                if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
                    if let TreeNodeKind::Task { task_idx: current_idx, .. } = &node.kind {
                        let current = *current_idx;
                        if current != original_task_idx {
                            if let Some(project) = self.doc.categories
                                .get_mut(cat_idx)
                                .and_then(|c| c.projects.get_mut(proj_idx))
                            {
                                let task = project.tasks.remove(current);
                                project.tasks.insert(original_task_idx, task);
                            }
                        }
                    }
                }
            }
            MoveKind::Project { original_cat_idx, original_proj_idx } => {
                if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
                    if let TreeNodeKind::Project { cat_idx: current_cat, proj_idx: current_proj, .. } = &node.kind {
                        let cur_cat = *current_cat;
                        let cur_proj = *current_proj;
                        if cur_cat != original_cat_idx || cur_proj != original_proj_idx {
                            // Remove from current position, insert at original
                            if let Some(category) = self.doc.categories.get_mut(cur_cat) {
                                let proj = category.projects.remove(cur_proj);
                                let dest = self.doc.categories.get_mut(original_cat_idx);
                                if let Some(dest_cat) = dest {
                                    let idx = original_proj_idx.min(dest_cat.projects.len());
                                    dest_cat.projects.insert(idx, proj);
                                }
                            }
                        }
                    }
                }
            }
            MoveKind::Category { original_cat_idx } => {
                if let Some(current) = self.settings_category_idx() {
                    if current != original_cat_idx {
                        let cat = self.doc.categories.remove(current);
                        self.doc.categories.insert(original_cat_idx, cat);
                    }
                }
                self.settings_cursor = original_cat_idx + 1; // +1 for theme row
            }
            MoveKind::AgendaItem { original_idx } => {
                // refresh_agenda() below rebuilds from doc, restoring original order
                self.agenda_cursor = original_idx;
            }
        }

        self.status_msg = "Move cancelled".to_string();
        self.refresh_agenda();
        self.rebuild_tree();
    }

    pub fn is_moving(&self) -> bool {
        self.moving.is_some()
    }

    // --- Backlog: add note ---

    pub fn add_note_to_focused(&mut self) {
        let note = self.input_buffer.trim().to_string();
        if note.is_empty() {
            return;
        }

        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    if engine::add_task_note(&mut self.doc, *cat_idx, *proj_idx, *task_idx, note) {
                        self.dirty = true;
                        self.status_msg = "Note added".to_string();
                        let saved = node.kind.clone();
                        self.rebuild_tree();
                        self.restore_cursor(&saved);
                    }
                }
                _ => {}
            }
        }
    }

    // --- Settings: category operations ---

    pub fn add_category_from_input(&mut self) {
        let name = self.input_buffer.trim().to_string();
        if name.is_empty() {
            return;
        }
        engine::add_category(&mut self.doc, name);
        self.dirty = true;
        self.status_msg = "Category added".to_string();
        self.rebuild_tree();
    }

    pub fn rename_category_from_input(&mut self) {
        let new_name = self.input_buffer.trim().to_string();
        if new_name.is_empty() {
            return;
        }
        if let Some(cat_idx) = self.settings_category_idx() {
            if engine::rename_category(&mut self.doc, cat_idx, new_name) {
                self.dirty = true;
                self.status_msg = "Category renamed".to_string();
                self.rebuild_tree();
            }
        }
    }

    pub fn delete_selected_category(&mut self) {
        if let Some(cat_idx) = self.settings_category_idx() {
            if engine::remove_category(&mut self.doc, cat_idx) {
                self.dirty = true;
                self.status_msg = "Category deleted".to_string();
                self.refresh_agenda();
                self.rebuild_tree();
                // Clamp cursor to valid range
                let total = self.settings_total();
                if total > 0 && self.settings_cursor >= total {
                    self.settings_cursor = total - 1;
                }
            }
        }
    }

    pub fn rerank_category(&mut self, direction: i32) {
        if let Some(cat_idx) = self.settings_category_idx() {
            if let Some(new_idx) = engine::rerank_category(&mut self.doc, cat_idx, direction) {
                self.settings_cursor = new_idx + 1; // +1 for theme row
                self.dirty = true;
                self.refresh_agenda();
                self.rebuild_tree();
            }
        }
    }

    // --- Serialization / Reload ---

    pub fn serialize(&self) -> String {
        serializer::serialize(&self.doc)
    }

    pub fn reload(&mut self, content: &str) {
        self.doc = parser::parse(content);
        self.dirty = false;
        self.status_msg = "Reloaded from disk".to_string();
        self.refresh_agenda();
        self.rebuild_tree();
    }

    // --- Dialog management ---

    pub fn open_dialog(&mut self, dialog: Dialog) {
        self.dialog = dialog;
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    pub fn open_dialog_with_text(&mut self, dialog: Dialog, text: &str) {
        self.dialog = dialog;
        self.input_buffer = text.to_string();
        self.input_cursor = text.len();
    }

    pub fn close_dialog(&mut self) {
        self.dialog = Dialog::None;
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    // --- Theme ---

    pub fn theme(&self) -> &Theme {
        &Theme::all()[self.theme_index]
    }

    pub fn next_theme(&mut self) {
        let count = Theme::all().len();
        self.theme_index = (self.theme_index + 1) % count;
    }

    pub fn prev_theme(&mut self) {
        let count = Theme::all().len();
        self.theme_index = (self.theme_index + count - 1) % count;
    }

    // Input buffer for dialogs
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.insert(self.input_cursor, c);
        self.input_cursor += 1;
    }

    pub fn input_backspace(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
            self.input_buffer.remove(self.input_cursor);
        }
    }

    pub fn input_delete(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_buffer.remove(self.input_cursor);
        }
    }

    pub fn input_move_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    pub fn input_move_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor += 1;
        }
    }

    /// Get the edit text for the currently focused backlog item.
    pub fn focused_edit_text(&self) -> String {
        if let Some(node) = self.tree_nodes.get(self.backlog_cursor) {
            match &node.kind {
                TreeNodeKind::Task { cat_idx, proj_idx, task_idx } => {
                    self.doc.categories.get(*cat_idx)
                        .and_then(|c| c.projects.get(*proj_idx))
                        .and_then(|p| p.tasks.get(*task_idx))
                        .map(|t| t.text.clone())
                        .unwrap_or_default()
                }
                TreeNodeKind::Project { cat_idx, proj_idx } => {
                    self.doc.categories.get(*cat_idx)
                        .and_then(|c| c.projects.get(*proj_idx))
                        .map(|p| p.name.clone())
                        .unwrap_or_default()
                }
                TreeNodeKind::Category { cat_idx } => {
                    self.doc.categories.get(*cat_idx)
                        .map(|c| c.name.clone())
                        .unwrap_or_default()
                }
                TreeNodeKind::Note { cat_idx, proj_idx, task_idx, note_idx } => {
                    self.doc.categories.get(*cat_idx)
                        .and_then(|c| c.projects.get(*proj_idx))
                        .and_then(|p| p.tasks.get(*task_idx))
                        .and_then(|t| t.notes.get(*note_idx))
                        .map(|n| n.trim().to_string())
                        .unwrap_or_default()
                }
            }
        } else {
            String::new()
        }
    }
}
