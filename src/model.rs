use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Todo,
    OnDeck,
    InProgress,
    Done,
}

impl TaskState {
    pub fn symbol(self) -> &'static str {
        match self {
            TaskState::Todo => "🔴",
            TaskState::OnDeck => "🔵",
            TaskState::InProgress => "🔶",
            TaskState::Done => "✅",
        }
    }

    pub fn dot(self) -> &'static str {
        "●"
    }

    pub fn label(self) -> &'static str {
        match self {
            TaskState::Todo => "Todo",
            TaskState::OnDeck => "On Deck",
            TaskState::InProgress => "In Progress",
            TaskState::Done => "Done",
        }
    }

    pub fn promote(self) -> Self {
        match self {
            TaskState::Todo => TaskState::OnDeck,
            TaskState::OnDeck => TaskState::InProgress,
            TaskState::InProgress => TaskState::Done,
            TaskState::Done => TaskState::Todo,
        }
    }

    pub fn demote(self) -> Self {
        match self {
            TaskState::Todo => TaskState::Done,
            TaskState::OnDeck => TaskState::Todo,
            TaskState::InProgress => TaskState::OnDeck,
            TaskState::Done => TaskState::InProgress,
        }
    }

    pub fn from_symbol(s: &str) -> Option<Self> {
        match s {
            "🔴" => Some(TaskState::Todo),
            "🔵" => Some(TaskState::OnDeck),
            "🔶" => Some(TaskState::InProgress),
            "✅" => Some(TaskState::Done),
            _ => None,
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.symbol(), self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub state: TaskState,
    pub text: String,
    pub notes: Vec<String>,
}

impl Task {
    pub fn new(state: TaskState, text: String) -> Self {
        Self {
            state,
            text,
            notes: Vec::new(),
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub active: bool,
    pub notes: Vec<String>,
    pub tasks: Vec<Task>,
}

impl Project {
    pub fn new(name: String, active: bool) -> Self {
        Self {
            name,
            active,
            notes: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub name: String,
    pub projects: Vec<Project>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            name,
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub preamble: Vec<String>,
    pub categories: Vec<Category>,
    pub archive: Vec<String>,
    pub trailing: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            preamble: Vec::new(),
            categories: Vec::new(),
            archive: Vec::new(),
            trailing: Vec::new(),
        }
    }

    pub fn template() -> Self {
        Self {
            preamble: Vec::new(),
            categories: vec![Category {
                name: "Inbox".to_string(),
                projects: vec![Project {
                    name: "Tasks".to_string(),
                    active: true,
                    notes: Vec::new(),
                    tasks: vec![Task::new(TaskState::Todo, "Your first task".to_string())],
                }],
            }],
            archive: Vec::new(),
            trailing: Vec::new(),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

// --- Agenda ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgendaItem {
    pub project_name: String,
    pub task: Task,
    pub category_idx: usize,
    pub project_idx: usize,
    pub task_idx: usize,
}

// --- Tree navigation ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeNodeKind {
    Category { cat_idx: usize },
    Project { cat_idx: usize, proj_idx: usize },
    Task { cat_idx: usize, proj_idx: usize, task_idx: usize },
    Note { cat_idx: usize, proj_idx: usize, task_idx: usize, note_idx: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub kind: TreeNodeKind,
    pub depth: u8,
    pub display: String,
}

#[derive(Debug, Clone)]
pub struct CollapseState {
    pub collapsed_categories: HashSet<usize>,
    pub collapsed_projects: HashSet<(usize, usize)>,
    pub collapsed_tasks: HashSet<(usize, usize, usize)>,
    pub theme_name: String,
}

impl CollapseState {
    pub fn new() -> Self {
        Self {
            collapsed_categories: HashSet::new(),
            collapsed_projects: HashSet::new(),
            collapsed_tasks: HashSet::new(),
            theme_name: String::new(),
        }
    }

    pub fn serialize(&self) -> String {
        let mut lines = Vec::new();
        if !self.theme_name.is_empty() {
            lines.push(format!("theme:{}", self.theme_name));
        }
        for idx in &self.collapsed_categories {
            lines.push(format!("cat:{}", idx));
        }
        for (ci, pi) in &self.collapsed_projects {
            lines.push(format!("proj:{},{}", ci, pi));
        }
        for (ci, pi, ti) in &self.collapsed_tasks {
            lines.push(format!("task:{},{},{}", ci, pi, ti));
        }
        lines.join("\n")
    }

    pub fn deserialize(content: &str) -> Self {
        let mut state = Self::new();
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("theme:") {
                state.theme_name = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("cat:") {
                if let Ok(idx) = rest.parse() {
                    state.collapsed_categories.insert(idx);
                }
            } else if let Some(rest) = line.strip_prefix("proj:") {
                let parts: Vec<&str> = rest.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(ci), Ok(pi)) = (parts[0].parse(), parts[1].parse()) {
                        state.collapsed_projects.insert((ci, pi));
                    }
                }
            } else if let Some(rest) = line.strip_prefix("task:") {
                let parts: Vec<&str> = rest.split(',').collect();
                if parts.len() == 3 {
                    if let (Ok(ci), Ok(pi), Ok(ti)) = (parts[0].parse(), parts[1].parse(), parts[2].parse()) {
                        state.collapsed_tasks.insert((ci, pi, ti));
                    }
                }
            }
        }
        state
    }
}

impl Default for CollapseState {
    fn default() -> Self {
        Self::new()
    }
}
