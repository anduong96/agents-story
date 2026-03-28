use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
#[allow(dead_code)]
pub enum DiscoveryEvent {
    NewSession { session_id: String, path: PathBuf },
    SessionEnded { session_id: String },
}

#[allow(dead_code)]
pub async fn discover_sessions(watch_dir: PathBuf, tx: mpsc::Sender<DiscoveryEvent>) {
    let mut known: HashSet<String> = HashSet::new();

    loop {
        match std::fs::read_dir(&watch_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                        let session_id = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        if known.insert(session_id.clone()) {
                            let _ = tx.send(DiscoveryEvent::NewSession {
                                session_id,
                                path: path.clone(),
                            }).await;
                        }
                    }
                }
            }
            Err(_) => {
                // Directory not accessible; wait and retry
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}
