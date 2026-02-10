use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Events from the file watcher.
#[derive(Debug)]
pub enum FileEvent {
    Modified,
}

/// Start watching a file for modifications.
/// Returns a receiver that emits FileEvent when the file changes.
pub fn watch_file(
    path: PathBuf,
) -> anyhow::Result<(mpsc::Receiver<FileEvent>, RecommendedWatcher)> {
    let (tx, rx) = mpsc::channel();
    let watch_path = path.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    // Only send if the event is for our file
                    if event.paths.iter().any(|p| p == &watch_path) {
                        let _ = tx.send(FileEvent::Modified);
                    }
                }
                _ => {}
            }
        }
    })?;

    // Watch the parent directory to catch renames (atomic writes)
    let parent = path.parent().unwrap_or(&path);
    watcher.watch(parent.as_ref(), RecursiveMode::NonRecursive)?;

    Ok((rx, watcher))
}

/// Check for file events without blocking.
pub fn poll_file_events(rx: &mpsc::Receiver<FileEvent>) -> Option<FileEvent> {
    rx.recv_timeout(Duration::from_millis(0)).ok()
}
