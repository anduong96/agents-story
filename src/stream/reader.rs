use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::stream::protocol::{parse_line, StreamEvent};

#[derive(Debug)]
pub enum ReaderMessage {
    Event { session_id: String, event: StreamEvent },
    SessionEnded { session_id: String },
    ReaderError { session_id: String, error: String },
}

pub struct SessionReader {
    pub session_id: String,
    pub path: PathBuf,
}

impl SessionReader {
    pub fn new(session_id: impl Into<String>, path: PathBuf) -> Self {
        Self {
            session_id: session_id.into(),
            path,
        }
    }

    pub fn spawn(self, tx: mpsc::Sender<ReaderMessage>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let file = match File::open(&self.path).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(ReaderMessage::ReaderError {
                        session_id: self.session_id.clone(),
                        error: e.to_string(),
                    }).await;
                    return;
                }
            };

            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if let Some(event) = parse_line(&line) {
                            let _ = tx.send(ReaderMessage::Event {
                                session_id: self.session_id.clone(),
                                event,
                            }).await;
                        }
                    }
                    Ok(None) => {
                        // EOF
                        let _ = tx.send(ReaderMessage::SessionEnded {
                            session_id: self.session_id.clone(),
                        }).await;
                        break;
                    }
                    Err(e) => {
                        let _ = tx.send(ReaderMessage::ReaderError {
                            session_id: self.session_id.clone(),
                            error: e.to_string(),
                        }).await;
                        break;
                    }
                }
            }
        })
    }
}
