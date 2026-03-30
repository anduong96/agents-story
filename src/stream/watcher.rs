use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::stream::protocol::parse_line;
use crate::stream::reader::ReaderMessage;

/// Recursively find the most recently modified `.jsonl` file under `base`.
fn find_latest_jsonl(base: &Path) -> Option<PathBuf> {
    let mut best: Option<(PathBuf, SystemTime)> = None;
    walk_dir(base, &mut best);
    best.map(|(path, _)| path)
}

fn walk_dir(dir: &Path, best: &mut Option<(PathBuf, SystemTime)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, best);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if best.as_ref().is_none_or(|(_, t)| modified > *t) {
                        *best = Some((path, modified));
                    }
                }
            }
        }
    }
}

/// Watch `~/.claude/` for session `.jsonl` files and tail the latest one.
/// Polls every 2 seconds until a file is found, then tails it for new lines.
pub async fn watch_sessions(tx: mpsc::Sender<ReaderMessage>) {
    let base = match std::env::var_os("HOME").map(PathBuf::from) {
        Some(h) => h.join(".claude"),
        None => return,
    };

    // Poll until a session file appears.
    let path = loop {
        if let Some(p) = find_latest_jsonl(&base) {
            break p;
        }
        sleep(Duration::from_secs(2)).await;
    };

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

/// Derive a session ID from the file path.
/// Uses the parent directory name if available, otherwise the file stem.
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
    fn test_find_latest_jsonl_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_latest_jsonl(dir.path()).is_none());
    }

    #[test]
    fn test_find_latest_jsonl_picks_newest() {
        let dir = tempfile::tempdir().unwrap();

        // Create two .jsonl files with different mtimes.
        let old = dir.path().join("old.jsonl");
        let new = dir.path().join("new.jsonl");
        fs::write(&old, "{}").unwrap();
        // Ensure different mtime by touching the newer file after a small delay.
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&new, "{}").unwrap();

        let result = find_latest_jsonl(dir.path()).unwrap();
        assert_eq!(result, new);
    }

    #[test]
    fn test_find_latest_jsonl_recursive() {
        let dir = tempfile::tempdir().unwrap();

        let nested = dir.path().join("sessions").join("abc123");
        fs::create_dir_all(&nested).unwrap();
        let file = nested.join("stream.jsonl");
        fs::write(&file, "{}").unwrap();

        let result = find_latest_jsonl(dir.path()).unwrap();
        assert_eq!(result, file);
    }

    #[test]
    fn test_find_latest_jsonl_ignores_non_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("data.json"), "{}").unwrap();
        fs::write(dir.path().join("log.txt"), "hello").unwrap();

        assert!(find_latest_jsonl(dir.path()).is_none());
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

    #[test]
    fn test_find_latest_nonexistent_dir() {
        let path = PathBuf::from("/nonexistent/dir/that/does/not/exist");
        assert!(find_latest_jsonl(&path).is_none());
    }
}
