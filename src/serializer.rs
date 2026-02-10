use crate::model::*;

/// Serialize a Document back to markdown text.
pub fn serialize(doc: &Document) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Preamble
    for line in &doc.preamble {
        lines.push(line.clone());
    }

    // Categories → Projects → Tasks
    for (_cat_idx, category) in doc.categories.iter().enumerate() {
        // Blank line before category (unless first thing after preamble)
        if !lines.is_empty() && !lines.last().is_some_and(|l| l.is_empty()) {
            lines.push(String::new());
        }

        lines.push(format!("## {}", category.name));

        for (_proj_idx, project) in category.projects.iter().enumerate() {
            lines.push(String::new()); // blank line before project

            if project.active {
                lines.push(format!("### 🔶 {}", project.name));
            } else {
                lines.push(format!("### {}", project.name));
            }

            // Project notes
            for note in &project.notes {
                lines.push(note.clone());
            }

            // Tasks
            for task in &project.tasks {
                lines.push(format!("- {} {}", task.state.symbol(), task.text));
                for note in &task.notes {
                    lines.push(note.clone());
                }
            }
        }
    }

    // Archive section
    if !doc.archive.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("## Done".to_string());
        for line in &doc.archive {
            lines.push(line.clone());
        }
    }

    // Trailing
    for line in &doc.trailing {
        lines.push(line.clone());
    }

    let mut result = lines.join("\n");
    // Ensure file ends with newline
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_roundtrip_new_format() {
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
        let output = serialize(&doc);
        let doc2 = parse(&output);
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_serialize_template() {
        let doc = Document::template();
        let output = serialize(&doc);
        assert!(output.contains("## Inbox"));
        assert!(output.contains("### 🔶 Tasks"));
        assert!(output.contains("- 🔴 Your first task"));
    }

    #[test]
    fn test_roundtrip_with_preamble() {
        let input = "\
# My GTD

Some notes here.

## Work

### 🔶 Project A
- 🔴 Task 1

## Done
- ✅ Old task
";
        let doc = parse(input);
        let output = serialize(&doc);
        let doc2 = parse(&output);
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_roundtrip_with_notes() {
        let input = "\
## Work

### 🔶 Project
- 🔴 Task with notes
  Note line 1
  Note line 2
- 🔴 Task without notes
";
        let doc = parse(input);
        let output = serialize(&doc);
        let doc2 = parse(&output);
        assert_eq!(doc.categories[0].projects[0].tasks[0].notes, doc2.categories[0].projects[0].tasks[0].notes);
        assert_eq!(doc, doc2);
    }
}
