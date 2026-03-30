use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

use crate::stream::protocol::parse_line;
use crate::stream::reader::ReaderMessage;

/// Read JSONL events from stdin and send them through the channel.
/// Used when stdin is piped (e.g. `tail -f session.jsonl | agents-story`).
pub async fn read_stdin(tx: mpsc::Sender<ReaderMessage>) {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    let session_id = "live".to_string();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(event) = parse_line(&line) {
            let _ = tx
                .send(ReaderMessage::Event {
                    session_id: session_id.clone(),
                    event,
                })
                .await;
        }
    }

    // EOF — stdin pipe closed
    let _ = tx.send(ReaderMessage::SessionEnded { session_id }).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::protocol::StreamEvent;

    /// Helper: simulate piped stdin by feeding lines through a channel
    /// and verifying the parsed output.
    async fn feed_lines(lines: Vec<&str>) -> Vec<ReaderMessage> {
        let (tx, mut rx) = mpsc::channel::<ReaderMessage>(64);

        // We can't easily mock stdin, so test the core logic directly:
        // parse each line and send through the channel, mirroring read_stdin behavior.
        let session_id = "live".to_string();
        for line in &lines {
            if let Some(event) = parse_line(line) {
                let _ = tx
                    .send(ReaderMessage::Event {
                        session_id: session_id.clone(),
                        event,
                    })
                    .await;
            }
        }
        let _ = tx
            .send(ReaderMessage::SessionEnded {
                session_id: session_id.clone(),
            })
            .await;
        drop(tx);

        let mut messages = Vec::new();
        while let Some(msg) = rx.recv().await {
            messages.push(msg);
        }
        messages
    }

    #[tokio::test]
    async fn test_parses_session_init() {
        let msgs = feed_lines(vec![
            r#"{"type":"system","session_id":"s1","model":"claude-opus-4-6"}"#,
        ])
        .await;

        assert_eq!(msgs.len(), 2); // 1 event + SessionEnded
        match &msgs[0] {
            ReaderMessage::Event { session_id, event } => {
                assert_eq!(session_id, "live");
                match event {
                    StreamEvent::SessionInit { model, .. } => {
                        assert_eq!(model, "claude-opus-4-6");
                    }
                    _ => panic!("expected SessionInit, got {:?}", event),
                }
            }
            _ => panic!("expected Event"),
        }
    }

    #[tokio::test]
    async fn test_parses_tool_use_and_agent_spawn() {
        let msgs = feed_lines(vec![
            r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Read","input":{"file_path":"/src/main.rs"}}}"#,
            r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Agent","input":{"agent_id":"a1","name":"Research","description":"Do research"}}}"#,
        ])
        .await;

        assert_eq!(msgs.len(), 3); // 2 events + SessionEnded
        match &msgs[0] {
            ReaderMessage::Event { event, .. } => match event {
                StreamEvent::ToolUse { tool, .. } => assert_eq!(tool, "Read"),
                _ => panic!("expected ToolUse"),
            },
            _ => panic!("expected Event"),
        }
        match &msgs[1] {
            ReaderMessage::Event { event, .. } => match event {
                StreamEvent::AgentSpawn { agent_id, name, .. } => {
                    assert_eq!(agent_id, "a1");
                    assert_eq!(name, "Research");
                }
                _ => panic!("expected AgentSpawn"),
            },
            _ => panic!("expected Event"),
        }
    }

    #[tokio::test]
    async fn test_skips_invalid_lines() {
        let msgs = feed_lines(vec![
            "not json",
            "",
            r#"{"type":"system","session_id":"s1","model":"opus"}"#,
            "also garbage {{{",
        ])
        .await;

        // Only the valid system line + SessionEnded
        assert_eq!(msgs.len(), 2);
    }

    #[tokio::test]
    async fn test_sends_session_ended_on_eof() {
        let msgs = feed_lines(vec![]).await;

        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            ReaderMessage::SessionEnded { session_id } => {
                assert_eq!(session_id, "live");
            }
            _ => panic!("expected SessionEnded"),
        }
    }
}
