mod app;
mod engine;
mod model;
mod parser;
mod serializer;
mod tui;
mod watcher;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::App;
use tui::input::{self, Action};

#[derive(ClapParser)]
#[command(name = "gws", about = "GWS - Getting Work Sorted: A GTD task manager TUI")]
struct Cli {
    /// Path to the todo markdown file
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn default_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".gws")
        .join("todo.md")
}

fn ensure_file(path: &PathBuf) -> Result<String> {
    if path.exists() {
        fs::read_to_string(path).context("Failed to read todo file")
    } else {
        // Create parent directories and write template
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create directory")?;
        }
        let doc = model::Document::template();
        let content = serializer::serialize(&doc);
        fs::write(path, &content).context("Failed to write template file")?;
        Ok(content)
    }
}

fn save_atomic(path: &PathBuf, content: &str) -> Result<()> {
    let tmp_path = path.with_extension("md.tmp");
    fs::write(&tmp_path, content).context("Failed to write temp file")?;
    fs::rename(&tmp_path, path).context("Failed to rename temp file")?;
    Ok(())
}

fn state_file_path(file_path: &PathBuf) -> PathBuf {
    file_path.with_extension("state")
}

fn load_collapse_state(file_path: &PathBuf) -> model::CollapseState {
    let state_path = state_file_path(file_path);
    if let Ok(content) = fs::read_to_string(&state_path) {
        model::CollapseState::deserialize(&content)
    } else {
        model::CollapseState::new()
    }
}

fn save_collapse_state(file_path: &PathBuf, state: &model::CollapseState) {
    let state_path = state_file_path(file_path);
    let _ = fs::write(&state_path, state.serialize());
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file_path = cli.file.unwrap_or_else(default_file_path);
    let content = ensure_file(&file_path)?;
    let doc = parser::parse(&content);

    let mut app = App::new(doc, file_path.clone());

    // Restore collapse state
    app.collapse = load_collapse_state(&file_path);
    app.rebuild_tree();

    // Set up file watcher
    let (watcher_rx, _watcher_handle) = match watcher::watch_file(file_path.clone()) {
        Ok((rx, w)) => (Some(rx), Some(w)),
        Err(_) => (None, None),
    };

    // Terminal setup
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Main event loop
    let result = run_loop(&mut terminal, &mut app, watcher_rx.as_ref());

    // Cleanup
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    // Auto-save on quit if dirty
    if app.dirty {
        let content = app.serialize();
        save_atomic(&app.file_path, &content)?;
    }

    // Save collapse state
    save_collapse_state(&app.file_path, &app.collapse);

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    watcher_rx: Option<&std::sync::mpsc::Receiver<watcher::FileEvent>>,
) -> Result<()> {
    loop {
        terminal.draw(|frame| tui::ui::draw(frame, &mut *app))?;

        // Check for file changes
        if let Some(rx) = watcher_rx {
            if watcher::poll_file_events(rx).is_some() {
                if !app.dirty {
                    let content = fs::read_to_string(&app.file_path)?;
                    app.reload(&content);
                } else {
                    app.status_msg = "External change detected (unsaved changes)".to_string();
                }
            }
        }

        // Poll for keyboard events with a timeout to allow watcher checks
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match input::handle_key(app, key) {
                    Action::Quit => {
                        app.running = false;
                        break;
                    }
                    Action::Save => {
                        let content = app.serialize();
                        save_atomic(&app.file_path, &content)?;
                        app.dirty = false;
                        app.status_msg = "Saved".to_string();
                    }
                    Action::Reload => {
                        let content = fs::read_to_string(&app.file_path)?;
                        app.reload(&content);
                    }
                    Action::None => {}
                }
            }
        }

        if !app.running {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use crate::app::App;
    use crate::engine;
    use crate::parser;
    use crate::serializer;
    use std::path::PathBuf;

    fn sample_content() -> &'static str {
        "\
## Business

### 🔶 Website Redesign
- 🔶 Finalize color palette with design team
- 🔵 Create wireframes for landing page
- 🔴 Set up staging environment
- 🔴 Write copy for About page

### 🔶 Essential
- 🔵 NVIDIA Conference expense
- 🔶 MFC agreement back to AA
- 🔴 NVIDIA plan

## Finance

### 🔶 Q1 Tax Filing
- 🔴 Collect all 1099 forms
- 🔴 Review expense categories
- 🔴 Schedule appointment with accountant

### 🔶 Kitchen Renovation
- 🔵 Get quotes from three contractors
- 🔴 Choose cabinet hardware
- 🔴 Order countertop samples

### Learn Rust
- 🔴 Read chapters 1-5 of The Book
- 🔴 Complete rustlings exercises

### 🔶 Inbox
- 🔴 Reply to Sarah's email
- 🔴 Book dentist appointment

## Done
- ✅ Set up new laptop
- ✅ File Q4 taxes
- ✅ Cancel old gym membership
"
    }

    #[test]
    fn test_full_workflow() {
        let content = sample_content();
        let doc = parser::parse(content);
        let mut app = App::new(doc, PathBuf::from("/tmp/test_gws.md"));

        // 1. Verify parse: 2 categories
        assert_eq!(app.doc.categories.len(), 2);
        assert_eq!(app.doc.categories[0].name, "Business");
        assert_eq!(app.doc.categories[1].name, "Finance");
        assert_eq!(app.doc.archive.len(), 3);

        // 2. Build agenda (before auto-promote)
        let agenda = engine::build_agenda(&app.doc);
        // Active projects: Website Redesign (🔶+🔵), Essential (🔵+🔶), Q1 Tax (no 🔵/🔶),
        // Kitchen (🔵), Inbox (no 🔵/🔶)
        let total: usize = agenda.len();
        assert!(total > 0);

        // 3. Run auto-promote
        app.run_auto_promote();
        let agenda = engine::build_agenda(&app.doc);
        assert!(agenda.len() >= total); // should have more or equal

        // 4. Promote a specific task (cat 0, proj 0, task 2 = "Set up staging")
        assert!(engine::promote_task(&mut app.doc, 0, 0, 2));
        assert_eq!(
            app.doc.categories[0].projects[0].tasks[2].state,
            crate::model::TaskState::OnDeck
        );

        // 5. Demote it back
        assert!(engine::demote_task(&mut app.doc, 0, 0, 2));
        assert_eq!(
            app.doc.categories[0].projects[0].tasks[2].state,
            crate::model::TaskState::Todo
        );

        // 6. Add a task
        engine::add_task(&mut app.doc, 0, 0, "Review PR #42".to_string());
        let last = app.doc.categories[0].projects[0].tasks.last().unwrap();
        assert_eq!(last.text, "Review PR #42");

        // 7. Archive done tasks
        let done_before: usize = app.doc.categories.iter()
            .flat_map(|c| c.projects.iter())
            .flat_map(|p| p.tasks.iter())
            .filter(|t| t.state == crate::model::TaskState::Done)
            .count();
        // We may have 0 done tasks in projects (they were in Done section originally)
        // Let's promote a task to Done first
        engine::promote_task(&mut app.doc, 0, 0, 0); // 🔶 → ✅
        let done_after_promote: usize = app.doc.categories.iter()
            .flat_map(|c| c.projects.iter())
            .flat_map(|p| p.tasks.iter())
            .filter(|t| t.state == crate::model::TaskState::Done)
            .count();
        assert!(done_after_promote > done_before);

        app.archive_done();
        let done_final: usize = app.doc.categories.iter()
            .flat_map(|c| c.projects.iter())
            .flat_map(|p| p.tasks.iter())
            .filter(|t| t.state == crate::model::TaskState::Done)
            .count();
        assert_eq!(done_final, 0);

        // 8. Serialize and round-trip
        let output = app.serialize();
        let doc2 = parser::parse(&output);
        assert_eq!(doc2.categories.len(), app.doc.categories.len());
    }

    #[test]
    fn test_parse_sample_file() {
        let content = std::fs::read_to_string("sample_todo.md").unwrap();
        let doc = parser::parse(&content);
        let serialized = serializer::serialize(&doc);
        let doc2 = parser::parse(&serialized);

        // Round-trip fidelity
        assert_eq!(doc.categories.len(), doc2.categories.len());
        assert_eq!(doc.archive.len(), doc2.archive.len());

        // Verify categories exist
        assert!(doc.categories.len() >= 2);

        let agenda = engine::build_agenda(&doc);
        assert!(!agenda.is_empty());
    }

    #[test]
    fn test_backward_compat_old_format() {
        let old_format = "\
### 🔶 Project Alpha
- 🔵 On deck task
- 🔴 Todo task

### 🔴 Inactive Project
- 🔴 Should not be touched

## Done
- ✅ Archived task 1
";
        let doc = parser::parse(old_format);

        // Should create synthetic "Uncategorized" category
        assert_eq!(doc.categories.len(), 1);
        assert_eq!(doc.categories[0].name, "Uncategorized");
        assert_eq!(doc.categories[0].projects.len(), 2);
        assert!(doc.categories[0].projects[0].active);
        assert!(!doc.categories[0].projects[1].active);

        // Agenda should work
        let agenda = engine::build_agenda(&doc);
        assert_eq!(agenda.len(), 1); // one 🔵 task from Alpha
    }

    #[test]
    fn test_tree_rebuild() {
        let content = sample_content();
        let doc = parser::parse(content);
        let app = App::new(doc, PathBuf::from("/tmp/test.md"));

        // Tree should have nodes
        assert!(!app.tree_nodes.is_empty());

        // First node should be a category
        assert!(matches!(app.tree_nodes[0].kind, crate::model::TreeNodeKind::Category { .. }));
    }

    #[test]
    fn test_task_notes_roundtrip() {
        let input = "\
## Work

### 🔶 Project
- 🔴 Task with notes
  Note line 1
  Note line 2
- 🔴 Task without notes
";
        let doc = parser::parse(input);
        let serialized = serializer::serialize(&doc);
        let doc2 = parser::parse(&serialized);
        assert_eq!(doc, doc2);
        assert_eq!(doc2.categories[0].projects[0].tasks[0].notes.len(), 2);
    }

    #[test]
    fn test_smoke_sample_file() {
        let content = std::fs::read_to_string("sample_todo.md").unwrap();
        let doc = parser::parse(&content);

        // --- Categories ---
        println!("\n=== CATEGORIES ===");
        for (ci, cat) in doc.categories.iter().enumerate() {
            println!("  [{}] ## {}", ci, cat.name);
            for (pi, proj) in cat.projects.iter().enumerate() {
                let marker = if proj.active { "🔶" } else { "  " };
                let task_count = proj.tasks.len();
                println!("      [{}] {} {} ({} tasks)", pi, marker, proj.name, task_count);
                for task in &proj.tasks {
                    println!("          {} {}", task.state.symbol(), task.text);
                    for note in &task.notes {
                        println!("              {}", note.trim());
                    }
                }
            }
        }

        // --- Agenda before auto-promote ---
        println!("\n=== AGENDA (before auto-promote) ===");
        let agenda = engine::build_agenda(&doc);
        for item in &agenda {
            println!("  ({}) {} {}", item.project_name, item.task.state.symbol(), item.task.text);
        }
        println!("  Total: {} items", agenda.len());

        // --- Auto-promote ---
        let mut doc = doc;
        engine::auto_promote(&mut doc);
        println!("\n=== AGENDA (after auto-promote) ===");
        let agenda = engine::build_agenda(&doc);
        for item in &agenda {
            println!("  ({}) {} {}", item.project_name, item.task.state.symbol(), item.task.text);
        }
        println!("  Total: {} items", agenda.len());

        // --- Tree view simulation ---
        println!("\n=== BACKLOG TREE ===");
        let app = App::new(doc, PathBuf::from("sample_todo.md"));
        for (i, node) in app.tree_nodes.iter().enumerate() {
            let indent = "    ".repeat(node.depth as usize);
            let cursor = if i == app.backlog_cursor { "▸" } else { " " };
            println!("{}{}{}", cursor, indent, node.display);
        }
        println!("  Total: {} tree nodes", app.tree_nodes.len());

        // --- Roundtrip ---
        let serialized = app.serialize();
        let doc2 = parser::parse(&serialized);
        assert_eq!(app.doc.categories.len(), doc2.categories.len());
        assert_eq!(app.doc.archive.len(), doc2.archive.len());
        println!("\n=== ROUNDTRIP OK ===");
        println!("  {} categories, {} archive lines", doc2.categories.len(), doc2.archive.len());

        // --- Archive ---
        println!("\n=== ARCHIVE ({} items) ===", app.doc.archive.len());
        for line in &app.doc.archive {
            println!("  {}", line);
        }
    }

    #[test]
    fn test_done_stays_on_agenda() {
        let content = "\
## Work

### 🔶 Project
- 🔶 Almost done task
- 🔵 On deck task
";
        let doc = parser::parse(content);
        let mut app = App::new(doc, PathBuf::from("/tmp/test.md"));

        // Both tasks should be on agenda (🔶 = InProgress, 🔵 = OnDeck)
        assert_eq!(app.agenda_items.len(), 2);

        // Promote the InProgress task to Done
        app.agenda_cursor = 0;
        app.promote_selected_agenda();

        // Task should now be Done
        assert_eq!(
            app.doc.categories[0].projects[0].tasks[0].state,
            crate::model::TaskState::Done,
        );

        // Done task should still be on the agenda
        assert_eq!(app.agenda_items.len(), 2, "Done task disappeared from agenda!");

        // Archive should remove it
        app.archive_done();
        app.refresh_agenda();
        assert_eq!(app.agenda_items.len(), 1, "Archived task should be gone from agenda");
    }
}
