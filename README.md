# GWS - Getting Work Sorted

A terminal-based GTD (Getting Things Done) task manager built in Rust. GWS uses a simple markdown file as its data store, giving you a fast keyboard-driven TUI while keeping your data portable and human-readable.

## Features

- **Three-level hierarchy**: Categories > Projects > Tasks
- **Three views**: Agenda (force-ranked work queue), Backlog (collapsible tree), Settings (category management)
- **Markdown-native**: Your todo list is a plain `.md` file you can edit anywhere
- **Auto-promote**: Active projects automatically surface their next task
- **Move mode**: Reorder anything with `m`, `j/k`, `Enter/Esc`
- **Task notes**: Attach freeform notes to any task
- **File watcher**: External edits are detected and reloaded
- **Atomic saves**: Data is never partially written
- **Persistent state**: Collapse/expand state preserved across sessions

## Installation

```
cargo build --release
cp target/release/gws ~/.local/bin/
```

## Usage

```
gws                          # Uses ~/.gws/todo.md (created if missing)
gws --file ~/my-tasks.md     # Use a specific file
```

## Markdown Format

```markdown
## Business

### 🔶 Active Project
- 🔵 Task on deck
- 🔴 Todo task
  A note attached to this task

### Inactive Project
- 🔴 Won't appear on agenda

## Finance

### 🔶 Tax Filing
- 🔶 In progress task
- 🔴 Next up

## Done
- ✅ Completed task
```

- `## Name` — Category
- `### 🔶 Name` — Active project (feeds the agenda) | `### Name` — Inactive
- `- 🔴 Text` — Task: 🔴 Todo, 🔵 OnDeck, 🔶 InProgress, ✅ Done
- Indented lines after a task — Notes

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit (auto-saves) |
| `Tab` | Cycle view |
| `s` | Save |
| `R` | Reload from disk |

### Agenda

| Key | Action |
|-----|--------|
| `j/k` | Navigate (wraps) |
| `g/G` | Top / Bottom |
| `l` | Center cursor |
| `m` | Move mode (reorder) |
| `p` | Promote task |
| `x` | Demote task |
| `r` | Force refresh |
| `A` | Archive done tasks |

### Backlog

| Key | Action |
|-----|--------|
| `j/k` | Navigate (wraps) |
| `g/G` | Top / Bottom |
| `l` | Center cursor |
| `Space` | Collapse / Expand |
| `p` | Promote (task: cycle state, project: toggle active) |
| `x` | Demote (reverse cycle) |
| `m` | Move mode (reorder, cross-category for projects) |
| `a` | Add (on category: new project, on project/task: new task) |
| `e` | Edit / Rename |
| `d` | Delete |
| `n` | Add note to task |
| `r` | Force refresh |
| `A` | Archive done tasks |

### Settings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `l` | Center cursor |
| `a` | Add category |
| `e` | Rename category |
| `d` | Delete category |
| `m` | Move mode (reorder) |

## Task State Cycle

```
p (promote): 🔴 Todo → 🔵 OnDeck → 🔶 InProgress → ✅ Done → 🔴 Todo
x (demote):  🔴 Todo → ✅ Done → 🔶 InProgress → 🔵 OnDeck → 🔴 Todo
```

The agenda shows OnDeck, InProgress, and Done tasks from active projects. Done tasks remain on the agenda until archived with `A`.

## Auto-Promote

When a project is active (🔶), its first non-done task is automatically promoted to OnDeck (🔵) if no task is already OnDeck or InProgress. This happens automatically whenever the agenda refreshes.

## License

MIT
