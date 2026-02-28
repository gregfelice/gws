# GWS — Terminal GTD Task Manager

Keyboard-driven TUI task manager with force-ranked agenda, three-level hierarchy (Categories > Projects > Tasks), and plain markdown storage.

## Quick Reference

```bash
# Build and run
cargo build --release
cargo run -- --file ~/todo.md

# Development
cargo run -- --file sample_todo.md
cargo test
cargo clippy
```

## Architecture

Single-binary Rust TUI with three views: Agenda (force-ranked active tasks), Backlog (category/project tree), Settings (category management).

### Source Layout

```
src/
├── main.rs          # CLI entry point (clap)
├── app.rs           # Core application logic (1,125 lines)
├── engine.rs        # Task processing engine
├── model.rs         # Category, Project, Task data structures
├── parser.rs        # Markdown file parsing
├── serializer.rs    # Markdown file writing
├── theme.rs         # Color themes (Gruvbox, Nord, Tokyo Night, Rose Pine)
├── watcher.rs       # File change detection
└── tui/
    ├── ui.rs        # Main rendering
    ├── input.rs     # Keyboard handling
    ├── widgets.rs   # Custom ratatui widgets
    └── views/       # Agenda, Backlog, Settings views
```

### Data Format
Plain markdown file with emoji state markers. See `sample_todo.md` for format.

## Conventions

- Rust 2021 edition
- Ratatui for TUI, Crossterm for terminal I/O, Clap for CLI args
- No TODO/FIXME comments (clean codebase)
- Atomic file saves (write to temp, rename)

## Key Documentation

- `README.md` — User-facing docs (features, keybindings, markdown format)
- `docs/BACKLOG.md` — Prioritized backlog (P0-P3)
- `sample_todo.md` — Example data file showing markdown format
- `*.gs` files — Legacy Google Apps Script predecessor (reference only)

## Status

MVP feature-complete (v0.1.0). Development paused.
