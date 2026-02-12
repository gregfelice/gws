use crate::model::*;

/// Auto-promote: For each active project, scan tasks top-down.
/// Skip ✅. If first 🔴 found, promote to 🔵, stop. If 🔵 or 🔶 already exists, stop.
pub fn auto_promote(doc: &mut Document) {
    for category in &mut doc.categories {
        for project in &mut category.projects {
            if !project.is_active() {
                continue;
            }
            for task in &mut project.tasks {
                match task.state {
                    TaskState::Done => continue,
                    TaskState::OnDeck => break,
                    TaskState::InProgress => break,
                    TaskState::Todo => {
                        task.state = TaskState::OnDeck;
                        break;
                    }
                }
            }
        }
    }
}

/// Archive: Collect all ✅ tasks from all projects, prepend to `## Done` section.
pub fn archive_done(doc: &mut Document) {
    let mut archived: Vec<String> = Vec::new();

    for category in &mut doc.categories {
        for project in &mut category.projects {
            project.tasks.retain(|task| {
                if task.state == TaskState::Done {
                    archived.push(format!("- ✅ {}", task.text));
                    return false;
                }
                true
            });
        }
    }

    archived.append(&mut doc.archive);
    doc.archive = archived;
}

/// Promote a specific task by 3-index address.
pub fn promote_task(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize) -> bool {
    if let Some(task) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
        .and_then(|p| p.tasks.get_mut(task_idx))
    {
        let new_state = task.state.promote();
        if new_state != task.state {
            task.state = new_state;
            return true;
        }
    }
    false
}

/// Demote a specific task by 3-index address.
pub fn demote_task(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize) -> bool {
    if let Some(task) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
        .and_then(|p| p.tasks.get_mut(task_idx))
    {
        let new_state = task.state.demote();
        if new_state != task.state {
            task.state = new_state;
            return true;
        }
    }
    false
}

/// Build flat agenda: all tasks from active projects, sorted by section.
pub fn build_agenda(doc: &Document) -> Vec<AgendaItem> {
    let mut items: Vec<AgendaItem> = Vec::new();

    for (cat_idx, category) in doc.categories.iter().enumerate() {
        for (proj_idx, project) in category.projects.iter().enumerate() {
            if !project.is_active() {
                continue;
            }
            for (task_idx, task) in project.tasks.iter().enumerate() {
                items.push(AgendaItem {
                    project_name: project.name.clone(),
                    task: task.clone(),
                    category_idx: cat_idx,
                    project_idx: proj_idx,
                    task_idx,
                });
            }
        }
    }

    // Stable sort by section order: Todo=0, InProgress=1, OnDeck=2, Done=3
    items.sort_by_key(|item| section_order(item.task.state));

    items
}

/// Section display order for agenda grouping.
pub fn section_order(state: TaskState) -> u8 {
    match state {
        TaskState::InProgress => 0,
        TaskState::OnDeck => 1,
        TaskState::Done => 2,
        TaskState::Todo => 3,
    }
}

/// Add a new Todo task to a project.
pub fn add_task(doc: &mut Document, cat_idx: usize, proj_idx: usize, text: String) -> bool {
    if let Some(project) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
    {
        project.tasks.push(Task::new(TaskState::Todo, text));
        true
    } else {
        false
    }
}

/// Toggle project active/inactive.
pub fn toggle_project_active(doc: &mut Document, cat_idx: usize, proj_idx: usize) -> bool {
    if let Some(project) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
    {
        project.active = !project.active;
        true
    } else {
        false
    }
}

/// Add a new project to a category.
pub fn add_project(doc: &mut Document, cat_idx: usize, name: String, active: bool) -> bool {
    if let Some(category) = doc.categories.get_mut(cat_idx) {
        category.projects.push(Project::new(name, active));
        true
    } else {
        false
    }
}

/// Delete a task.
pub fn delete_task(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize) -> bool {
    if let Some(project) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
    {
        if task_idx < project.tasks.len() {
            project.tasks.remove(task_idx);
            return true;
        }
    }
    false
}

/// Delete a project.
pub fn delete_project(doc: &mut Document, cat_idx: usize, proj_idx: usize) -> bool {
    if let Some(category) = doc.categories.get_mut(cat_idx) {
        if proj_idx < category.projects.len() {
            category.projects.remove(proj_idx);
            return true;
        }
    }
    false
}

/// Rerank a task within its project (direction: -1 = up, 1 = down).
pub fn rerank_task(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize, direction: i32) -> Option<usize> {
    let project = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))?;

    let new_idx = task_idx as i32 + direction;
    if new_idx < 0 || new_idx >= project.tasks.len() as i32 {
        return None;
    }
    let new_idx = new_idx as usize;
    project.tasks.swap(task_idx, new_idx);
    Some(new_idx)
}

/// Rerank a project within its category (direction: -1 = up, 1 = down).
pub fn rerank_project(doc: &mut Document, cat_idx: usize, proj_idx: usize, direction: i32) -> Option<usize> {
    let category = doc.categories.get_mut(cat_idx)?;
    let new_idx = proj_idx as i32 + direction;
    if new_idx < 0 || new_idx >= category.projects.len() as i32 {
        return None;
    }
    let new_idx = new_idx as usize;
    category.projects.swap(proj_idx, new_idx);
    Some(new_idx)
}

/// Move a project from one category to another.
/// Returns (new_cat_idx, new_proj_idx) on success.
pub fn move_project_to_category(doc: &mut Document, from_cat: usize, proj_idx: usize, to_cat: usize, insert_idx: usize) -> Option<(usize, usize)> {
    if from_cat >= doc.categories.len() || to_cat >= doc.categories.len() {
        return None;
    }
    if proj_idx >= doc.categories[from_cat].projects.len() {
        return None;
    }
    let project = doc.categories[from_cat].projects.remove(proj_idx);
    let idx = insert_idx.min(doc.categories[to_cat].projects.len());
    doc.categories[to_cat].projects.insert(idx, project);
    Some((to_cat, idx))
}

/// Rename a task.
pub fn rename_task(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize, new_text: String) -> bool {
    if let Some(task) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
        .and_then(|p| p.tasks.get_mut(task_idx))
    {
        task.text = new_text;
        true
    } else {
        false
    }
}

/// Rename a project.
pub fn rename_project(doc: &mut Document, cat_idx: usize, proj_idx: usize, new_name: String) -> bool {
    if let Some(project) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
    {
        project.name = new_name;
        true
    } else {
        false
    }
}

/// Add a new category.
pub fn add_category(doc: &mut Document, name: String) {
    doc.categories.push(Category::new(name));
}

/// Remove a category by index.
pub fn remove_category(doc: &mut Document, cat_idx: usize) -> bool {
    if cat_idx < doc.categories.len() {
        doc.categories.remove(cat_idx);
        true
    } else {
        false
    }
}

/// Rename a category.
pub fn rename_category(doc: &mut Document, cat_idx: usize, new_name: String) -> bool {
    if let Some(category) = doc.categories.get_mut(cat_idx) {
        category.name = new_name;
        true
    } else {
        false
    }
}

/// Rerank a category (direction: -1 = up, 1 = down).
pub fn rerank_category(doc: &mut Document, cat_idx: usize, direction: i32) -> Option<usize> {
    let new_idx = cat_idx as i32 + direction;
    if new_idx < 0 || new_idx >= doc.categories.len() as i32 {
        return None;
    }
    let new_idx = new_idx as usize;
    doc.categories.swap(cat_idx, new_idx);
    Some(new_idx)
}

/// Add a note to a task.
pub fn add_task_note(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize, note: String) -> bool {
    if let Some(task) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
        .and_then(|p| p.tasks.get_mut(task_idx))
    {
        task.notes.push(format!("  {}", note));
        true
    } else {
        false
    }
}

/// Delete a note from a task.
pub fn delete_task_note(doc: &mut Document, cat_idx: usize, proj_idx: usize, task_idx: usize, note_idx: usize) -> bool {
    if let Some(task) = doc
        .categories
        .get_mut(cat_idx)
        .and_then(|c| c.projects.get_mut(proj_idx))
        .and_then(|p| p.tasks.get_mut(task_idx))
    {
        if note_idx < task.notes.len() {
            task.notes.remove(note_idx);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn sample_doc() -> Document {
        parse(
            "\
## Work

### 🔶 Project Alpha
- 🔴 First todo
- 🔴 Second todo

### 🔶 Project Beta
- 🔵 Already on deck
- 🔴 A todo

### Inactive Project
- 🔴 Should not be touched
",
        )
    }

    #[test]
    fn test_auto_promote_basic() {
        let mut doc = sample_doc();
        auto_promote(&mut doc);

        let alpha = &doc.categories[0].projects[0];
        assert_eq!(alpha.tasks[0].state, TaskState::OnDeck); // 🔴 → 🔵
        assert_eq!(alpha.tasks[1].state, TaskState::Todo); // unchanged

        let beta = &doc.categories[0].projects[1];
        assert_eq!(beta.tasks[0].state, TaskState::OnDeck); // already 🔵, unchanged
        assert_eq!(beta.tasks[1].state, TaskState::Todo); // unchanged

        let inactive = &doc.categories[0].projects[2];
        assert_eq!(inactive.tasks[0].state, TaskState::Todo); // not touched
    }

    #[test]
    fn test_auto_promote_idempotent() {
        let mut doc = sample_doc();
        auto_promote(&mut doc);
        let after_first = doc.clone();
        auto_promote(&mut doc);
        assert_eq!(doc, after_first);
    }

    #[test]
    fn test_archive_done() {
        let mut doc = parse(
            "\
## Work

### 🔶 Project
- ✅ Already done
- 🔴 Not done

## Done
- ✅ Old archive
",
        );

        archive_done(&mut doc);
        assert_eq!(doc.categories[0].projects[0].tasks.len(), 1);
        assert!(doc.archive.iter().any(|l| l.contains("Already done")));
        assert!(doc.archive.iter().any(|l| l.contains("Old archive")));
    }

    #[test]
    fn test_promote_task() {
        let mut doc = sample_doc();
        assert!(promote_task(&mut doc, 0, 0, 0)); // 🔴 → 🔵
        assert_eq!(doc.categories[0].projects[0].tasks[0].state, TaskState::OnDeck);
    }

    #[test]
    fn test_demote_task() {
        let mut doc = sample_doc();
        assert!(demote_task(&mut doc, 0, 1, 0)); // 🔵 → 🔴
        assert_eq!(doc.categories[0].projects[1].tasks[0].state, TaskState::Todo);
    }

    #[test]
    fn test_build_agenda() {
        let mut doc = sample_doc();
        auto_promote(&mut doc);
        let agenda = build_agenda(&doc);

        // Alpha: [OnDeck, Todo], Beta: [OnDeck, Todo] — inactive project excluded
        // Sorted by section: OnDeck(1), Todo(3)
        assert_eq!(agenda.len(), 4);
        assert_eq!(agenda[0].task.state, TaskState::OnDeck);
        assert_eq!(agenda[1].task.state, TaskState::OnDeck);
        assert_eq!(agenda[2].task.state, TaskState::Todo);
        assert_eq!(agenda[3].task.state, TaskState::Todo);
    }

    #[test]
    fn test_add_task() {
        let mut doc = sample_doc();
        assert!(add_task(&mut doc, 0, 0, "New task".to_string()));
        let last = doc.categories[0].projects[0].tasks.last().unwrap();
        assert_eq!(last.state, TaskState::Todo);
        assert_eq!(last.text, "New task");
    }

    #[test]
    fn test_add_task_invalid_index() {
        let mut doc = sample_doc();
        assert!(!add_task(&mut doc, 99, 0, "Nope".to_string()));
    }

    #[test]
    fn test_toggle_project_active() {
        let mut doc = sample_doc();
        let proj = &doc.categories[0].projects[2];
        assert!(!proj.active);

        toggle_project_active(&mut doc, 0, 2);
        assert!(doc.categories[0].projects[2].active);

        toggle_project_active(&mut doc, 0, 2);
        assert!(!doc.categories[0].projects[2].active);
    }

    #[test]
    fn test_add_delete_project() {
        let mut doc = sample_doc();
        let count_before = doc.categories[0].projects.len();
        add_project(&mut doc, 0, "New Project".to_string(), true);
        assert_eq!(doc.categories[0].projects.len(), count_before + 1);

        delete_project(&mut doc, 0, count_before);
        assert_eq!(doc.categories[0].projects.len(), count_before);
    }

    #[test]
    fn test_rerank_task() {
        let mut doc = sample_doc();
        let new_idx = rerank_task(&mut doc, 0, 0, 0, 1); // move first task down
        assert_eq!(new_idx, Some(1));
        assert_eq!(doc.categories[0].projects[0].tasks[0].text, "Second todo");
        assert_eq!(doc.categories[0].projects[0].tasks[1].text, "First todo");
    }

    #[test]
    fn test_rerank_project() {
        let mut doc = sample_doc();
        let new_idx = rerank_project(&mut doc, 0, 0, 1);
        assert_eq!(new_idx, Some(1));
        assert_eq!(doc.categories[0].projects[0].name, "Project Beta");
        assert_eq!(doc.categories[0].projects[1].name, "Project Alpha");
    }

    #[test]
    fn test_rename_task() {
        let mut doc = sample_doc();
        assert!(rename_task(&mut doc, 0, 0, 0, "Renamed".to_string()));
        assert_eq!(doc.categories[0].projects[0].tasks[0].text, "Renamed");
    }

    #[test]
    fn test_rename_project() {
        let mut doc = sample_doc();
        assert!(rename_project(&mut doc, 0, 0, "Renamed Proj".to_string()));
        assert_eq!(doc.categories[0].projects[0].name, "Renamed Proj");
    }

    #[test]
    fn test_category_crud() {
        let mut doc = sample_doc();
        let count = doc.categories.len();

        add_category(&mut doc, "New Cat".to_string());
        assert_eq!(doc.categories.len(), count + 1);

        rename_category(&mut doc, count, "Renamed Cat".to_string());
        assert_eq!(doc.categories[count].name, "Renamed Cat");

        let new_idx = rerank_category(&mut doc, count, -1);
        assert_eq!(new_idx, Some(count - 1));

        remove_category(&mut doc, count - 1);
        assert_eq!(doc.categories.len(), count);
    }

    #[test]
    fn test_task_notes() {
        let mut doc = sample_doc();
        assert!(add_task_note(&mut doc, 0, 0, 0, "A note".to_string()));
        assert_eq!(doc.categories[0].projects[0].tasks[0].notes.len(), 1);
        assert_eq!(doc.categories[0].projects[0].tasks[0].notes[0], "  A note");

        assert!(delete_task_note(&mut doc, 0, 0, 0, 0));
        assert_eq!(doc.categories[0].projects[0].tasks[0].notes.len(), 0);
    }

    #[test]
    fn test_delete_task() {
        let mut doc = sample_doc();
        let count = doc.categories[0].projects[0].tasks.len();
        assert!(delete_task(&mut doc, 0, 0, 0));
        assert_eq!(doc.categories[0].projects[0].tasks.len(), count - 1);
    }
}
