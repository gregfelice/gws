use crate::model::*;

const TASK_SYMBOLS: [&str; 4] = ["🔴", "🔵", "🔶", "✅"];

/// Parse a markdown line that starts with `- ` and contains a task symbol.
fn parse_task_line(line: &str) -> Option<Task> {
    let trimmed = line.trim();
    let content = trimmed.strip_prefix("- ")?;

    for sym in &TASK_SYMBOLS {
        if let Some(rest) = content.strip_prefix(sym) {
            let state = TaskState::from_symbol(sym)?;
            let text = rest.trim_start().to_string();
            return Some(Task::new(state, text));
        }
    }
    None
}

/// Parse `## Name` category heading (not `## Done`).
fn parse_category_heading(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let content = trimmed.strip_prefix("## ")?;
    let name = content.trim().to_string();
    if name.eq_ignore_ascii_case("Done") {
        return None;
    }
    Some(name)
}

/// Parse `### 🔶 Name` (active) or `### Name` (inactive) project heading.
fn parse_project_heading(line: &str) -> Option<(bool, String)> {
    let trimmed = line.trim();
    let content = trimmed.strip_prefix("### ")?;

    // Check for 🔶 prefix → active project
    if let Some(rest) = content.strip_prefix("🔶") {
        let name = rest.trim_start().to_string();
        return Some((true, name));
    }

    // Backward compat: old format used any symbol for project state
    // Check for other task symbols at start of project name
    for sym in &TASK_SYMBOLS {
        if let Some(rest) = content.strip_prefix(sym) {
            let name = rest.trim_start().to_string();
            // 🔶 is active, everything else is inactive
            let active = *sym == "🔶";
            return Some((active, name));
        }
    }

    // No symbol → inactive project
    Some((false, content.trim().to_string()))
}

/// Check if a line is the `## Done` archive header.
fn is_done_header(line: &str) -> bool {
    line.trim() == "## Done"
}

/// Check if a line is a note (indented by 2+ spaces, not a task line).
fn is_note_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    // Line starts with 2+ spaces (or a tab) and is not a task line
    let has_indent = line.starts_with("  ") || line.starts_with('\t');
    has_indent && parse_task_line(line).is_none()
}

/// Parse a markdown string into a Document.
pub fn parse(input: &str) -> Document {
    let mut doc = Document::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut in_archive = false;
    let mut current_category: Option<Category> = None;
    let mut current_project: Option<Project> = None;
    let mut _has_categories = false;

    while i < lines.len() {
        let line = lines[i];

        // Check for ## Done
        if is_done_header(line) {
            // Flush current project into current category
            if let Some(proj) = current_project.take() {
                if let Some(ref mut cat) = current_category {
                    cat.projects.push(proj);
                }
            }
            // Flush current category
            if let Some(cat) = current_category.take() {
                doc.categories.push(cat);
            }
            in_archive = true;
            i += 1;
            continue;
        }

        if in_archive {
            doc.archive.push(line.to_string());
            i += 1;
            continue;
        }

        // Check for ## Category heading
        if let Some(name) = parse_category_heading(line) {
            // Flush current project into current category
            if let Some(proj) = current_project.take() {
                if let Some(ref mut cat) = current_category {
                    cat.projects.push(proj);
                }
            }
            // Flush current category
            if let Some(cat) = current_category.take() {
                doc.categories.push(cat);
            }
            _has_categories = true;
            current_category = Some(Category::new(name));
            i += 1;
            continue;
        }

        // Check for ### Project heading
        if let Some((active, name)) = parse_project_heading(line) {
            // Flush current project into current category
            if let Some(proj) = current_project.take() {
                if let Some(ref mut cat) = current_category {
                    cat.projects.push(proj);
                }
            }

            // Backward compat: if no ## category seen yet, create "Uncategorized"
            if current_category.is_none() {
                current_category = Some(Category::new("Uncategorized".to_string()));
            }

            current_project = Some(Project::new(name, active));
            i += 1;
            continue;
        }

        // Inside a project
        if let Some(ref mut proj) = current_project {
            if let Some(task) = parse_task_line(line) {
                proj.tasks.push(task);
            } else if !proj.tasks.is_empty() && is_note_line(line) {
                // Note on the last task
                let last = proj.tasks.last_mut().unwrap();
                last.notes.push(line.to_string());
            } else if proj.tasks.is_empty() && !line.trim().is_empty() {
                // Non-task line before first task → project note
                proj.notes.push(line.to_string());
            } else if !proj.tasks.is_empty() && !is_note_line(line) && !line.trim().is_empty() {
                // Non-indented, non-task line after tasks started → also project note
                // (e.g. raw lines in old format)
                let last = proj.tasks.last_mut().unwrap();
                last.notes.push(line.to_string());
            }
            // else: blank line inside project, skip
        } else if current_category.is_some() {
            // Line inside a category but not in a project — skip blank lines
            // Non-blank lines are unusual here but we'll ignore them
        } else {
            doc.preamble.push(line.to_string());
        }

        i += 1;
    }

    // Flush remaining
    if let Some(proj) = current_project.take() {
        if let Some(ref mut cat) = current_category {
            cat.projects.push(proj);
        }
    }
    if let Some(cat) = current_category.take() {
        doc.categories.push(cat);
    }

    // Trim trailing empty lines from archive into trailing
    while doc
        .archive
        .last()
        .is_some_and(|l| l.trim().is_empty())
    {
        let line = doc.archive.pop().unwrap();
        doc.trailing.push(line);
    }
    doc.trailing.reverse();

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_new_format() {
        let input = "\
## Business

### 🔶 Essential
- 🔵 NVIDIA Conference expense
- 🔴 NVIDIA plan

### Medical
- 🔶 Calcium score appointment +1 ..1/18
  PC MD: (516) 671-9800

## Finance

### 🔶 Expense Reduction
- 🔴 Verify $1000 credit

## Done
- ✅ Set up new laptop
";
        let doc = parse(input);
        assert_eq!(doc.categories.len(), 2);
        assert_eq!(doc.categories[0].name, "Business");
        assert_eq!(doc.categories[0].projects.len(), 2);

        let essential = &doc.categories[0].projects[0];
        assert_eq!(essential.name, "Essential");
        assert!(essential.active);
        assert_eq!(essential.tasks.len(), 2);

        let medical = &doc.categories[0].projects[1];
        assert_eq!(medical.name, "Medical");
        assert!(!medical.active);
        assert_eq!(medical.tasks.len(), 1);
        assert_eq!(medical.tasks[0].notes.len(), 1);
        assert_eq!(medical.tasks[0].notes[0], "  PC MD: (516) 671-9800");

        assert_eq!(doc.categories[1].name, "Finance");
        assert_eq!(doc.categories[1].projects[0].name, "Expense Reduction");
        assert!(doc.categories[1].projects[0].active);

        assert_eq!(doc.archive.len(), 1);
    }

    #[test]
    fn test_parse_backward_compat_no_categories() {
        let input = "\
### 🔶 My Project
- 🔵 On deck task
- 🔴 Todo task

### 🔴 Inactive Project
- 🔴 Should not be touched

## Done
- ✅ Archived task 1
";
        let doc = parse(input);
        // Should create synthetic "Uncategorized" category
        assert_eq!(doc.categories.len(), 1);
        assert_eq!(doc.categories[0].name, "Uncategorized");
        assert_eq!(doc.categories[0].projects.len(), 2);

        let proj1 = &doc.categories[0].projects[0];
        assert_eq!(proj1.name, "My Project");
        assert!(proj1.active);
        assert_eq!(proj1.tasks.len(), 2);

        let proj2 = &doc.categories[0].projects[1];
        assert_eq!(proj2.name, "Inactive Project");
        assert!(!proj2.active); // 🔴 maps to inactive

        assert_eq!(doc.archive.len(), 1);
    }

    #[test]
    fn test_parse_task_line() {
        let task = parse_task_line("- 🔴 Buy milk").unwrap();
        assert_eq!(task.state, TaskState::Todo);
        assert_eq!(task.text, "Buy milk");

        let task = parse_task_line("- 🔵 Next up").unwrap();
        assert_eq!(task.state, TaskState::OnDeck);
        assert_eq!(task.text, "Next up");

        assert!(parse_task_line("Regular line").is_none());
        assert!(parse_task_line("- Regular list item").is_none());
    }

    #[test]
    fn test_parse_project_heading() {
        let (active, name) = parse_project_heading("### 🔶 My Project").unwrap();
        assert!(active);
        assert_eq!(name, "My Project");

        let (active, name) = parse_project_heading("### Inactive One").unwrap();
        assert!(!active);
        assert_eq!(name, "Inactive One");

        assert!(parse_project_heading("## Not a project").is_none());
    }

    #[test]
    fn test_parse_empty() {
        let doc = parse("");
        assert_eq!(doc.categories.len(), 0);
        assert_eq!(doc.archive.len(), 0);
    }

    #[test]
    fn test_parse_task_notes() {
        let input = "\
## Work

### 🔶 Project
- 🔴 Task with notes
  Note line 1
  Note line 2
- 🔴 Task without notes
";
        let doc = parse(input);
        let task = &doc.categories[0].projects[0].tasks[0];
        assert_eq!(task.text, "Task with notes");
        assert_eq!(task.notes.len(), 2);
        assert_eq!(task.notes[0], "  Note line 1");
        assert_eq!(task.notes[1], "  Note line 2");

        let task2 = &doc.categories[0].projects[0].tasks[1];
        assert_eq!(task2.notes.len(), 0);
    }

    #[test]
    fn test_parse_project_notes() {
        let input = "\
## Work

### 🔶 Project
Some project note
- 🔴 A task
";
        let doc = parse(input);
        let proj = &doc.categories[0].projects[0];
        assert_eq!(proj.notes.len(), 1);
        assert_eq!(proj.notes[0], "Some project note");
    }
}
