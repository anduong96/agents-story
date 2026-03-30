use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration as StdDuration, SystemTime};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::stream::protocol::parse_line;
use crate::stream::reader::ReaderMessage;

/// How recently a file must have been modified to be considered active.
const ACTIVE_THRESHOLD: StdDuration = StdDuration::from_secs(60);

/// Recursively find `.jsonl` files under `base` that were modified recently.
fn find_active_jsonl(base: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let cutoff = SystemTime::now() - ACTIVE_THRESHOLD;
    walk_dir(base, &mut results, cutoff);
    results
}

fn walk_dir(dir: &Path, results: &mut Vec<PathBuf>, cutoff: SystemTime) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, results, cutoff);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            // Only include files modified after the cutoff.
            let is_active = path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .is_some_and(|mtime| mtime > cutoff);
            if is_active {
                results.push(path);
            }
        }
    }
}

/// Watch `~/.claude/` for session `.jsonl` files and tail each one.
/// Polls every 2 seconds for new files. Each file gets its own tailing task.
pub async fn watch_sessions(tx: mpsc::Sender<ReaderMessage>) {
    let base = match std::env::var_os("HOME").map(PathBuf::from) {
        Some(h) => h.join(".claude"),
        None => return,
    };

    let mut known: HashSet<PathBuf> = HashSet::new();

    loop {
        for path in find_active_jsonl(&base) {
            if known.insert(path.clone()) {
                let project = extract_project(&path);
                let tx = tx.clone();
                tokio::spawn(tail_file(path, project, tx));
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
}

/// Tail a file, sending parsed events through the channel.
/// On EOF, polls for new data every 200ms (the file may still be written to).
async fn tail_file(path: PathBuf, project: String, tx: mpsc::Sender<ReaderMessage>) {
    let session_id = session_id_from_path(&path);

    let file = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(e) => {
            let _ = tx
                .send(ReaderMessage::ReaderError {
                    session_id,
                    error: e.to_string(),
                })
                .await;
            return;
        }
    };

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF — file may still be written to, poll for new data.
                sleep(Duration::from_millis(200)).await;
            }
            Ok(_) => {
                if let Some(event) = parse_line(&line) {
                    let _ = tx
                        .send(ReaderMessage::Event {
                            session_id: session_id.clone(),
                            project: project.clone(),
                            event,
                        })
                        .await;
                }
            }
            Err(e) => {
                let _ = tx
                    .send(ReaderMessage::ReaderError {
                        session_id,
                        error: e.to_string(),
                    })
                    .await;
                break;
            }
        }
    }
}

/// Extract a human-readable project name from the file path.
/// Looks for the `projects/<encoded-path>` segment in the path.
pub fn extract_project(path: &Path) -> String {
    let components: Vec<_> = path.components().collect();
    for (i, comp) in components.iter().enumerate() {
        if comp.as_os_str() == "projects" {
            if let Some(next) = components.get(i + 1) {
                let dir_name = next.as_os_str().to_str().unwrap_or("unknown");
                return prettify_project_dir(dir_name);
            }
        }
    }
    // Fallback: use parent directory name.
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Claude Code encodes working directory paths as dash-separated names:
/// `-Users-aduong-Git-agents-story` → `/Users/aduong/Git/agents-story`
/// Extract the last path component as the project name.
fn prettify_project_dir(encoded: &str) -> String {
    // The encoding replaces '/' with '-'. The leading '-' represents root '/'.
    // We can't perfectly reverse this (hyphens in names are ambiguous),
    // but we can try: check if the decoded path exists on disk.
    let decoded = encoded.replacen('-', "/", encoded.len());
    let decoded_path = Path::new(&decoded);
    if decoded_path.is_dir() {
        return decoded_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(encoded)
            .to_string();
    }

    // Fallback: strip leading dash and return as-is.
    encoded.trim_start_matches('-').to_string()
}

/// Derive a session ID from the file path.
fn session_id_from_path(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .or_else(|| path.file_stem().and_then(|s| s.to_str()))
        .unwrap_or("live")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_active_jsonl_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_active_jsonl(dir.path()).is_empty());
    }

    #[test]
    fn test_find_active_jsonl_finds_recent_files() {
        let dir = tempfile::tempdir().unwrap();
        // Files just created are within the active threshold.
        fs::write(dir.path().join("a.jsonl"), "{}").unwrap();
        fs::write(dir.path().join("b.jsonl"), "{}").unwrap();
        fs::write(dir.path().join("c.txt"), "hello").unwrap();

        let results = find_active_jsonl(dir.path());
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|p| p.extension().unwrap() == "jsonl"));
    }

    #[test]
    fn test_find_active_jsonl_recursive() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("sessions").join("abc123");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("stream.jsonl"), "{}").unwrap();
        fs::write(dir.path().join("root.jsonl"), "{}").unwrap();

        let results = find_active_jsonl(dir.path());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_active_jsonl_nonexistent_dir() {
        let path = PathBuf::from("/nonexistent/dir/that/does/not/exist");
        assert!(find_active_jsonl(&path).is_empty());
    }

    #[test]
    fn test_extract_project_from_claude_path() {
        let path = PathBuf::from(
            "/Users/aduong/.claude/projects/-Users-aduong-Git-agents-story/sessions/abc/stream.jsonl",
        );
        let project = extract_project(&path);
        // Can't perfectly decode, but should contain something meaningful
        assert!(!project.is_empty());
        assert_ne!(project, "unknown");
    }

    #[test]
    fn test_extract_project_fallback() {
        let path = PathBuf::from("/some/random/dir/stream.jsonl");
        let project = extract_project(&path);
        assert_eq!(project, "dir");
    }

    #[test]
    fn test_prettify_project_dir_strips_leading_dash() {
        let result = prettify_project_dir("-Users-foo-bar");
        // Falls back to trimmed version since path won't exist on disk
        assert_eq!(result, "Users-foo-bar");
    }

    #[test]
    fn test_session_id_from_path_uses_parent() {
        let path = PathBuf::from("/home/user/.claude/sessions/abc123/stream.jsonl");
        assert_eq!(session_id_from_path(&path), "abc123");
    }

    #[test]
    fn test_session_id_from_path_falls_back_to_stem() {
        let path = PathBuf::from("/stream.jsonl");
        assert_eq!(session_id_from_path(&path), "stream");
    }
}
