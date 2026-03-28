use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::stream::protocol::StreamEvent;
use crate::stream::reader::ReaderMessage;

async fn send(tx: &mpsc::Sender<ReaderMessage>, session_id: &str, event: StreamEvent) {
    let _ = tx
        .send(ReaderMessage::Event {
            session_id: session_id.to_string(),
            event,
        })
        .await;
}

pub async fn run_demo(tx: mpsc::Sender<ReaderMessage>) {
    let session_id = "demo-session";

    // (0ms) SessionInit
    send(
        &tx,
        session_id,
        StreamEvent::SessionInit {
            session_id: session_id.to_string(),
            model: "claude-opus-4-6".to_string(),
        },
    )
    .await;

    // (500ms) AgentSpawn: agent-01
    sleep(Duration::from_millis(500)).await;
    send(
        &tx,
        session_id,
        StreamEvent::AgentSpawn {
            agent_id: "agent-01".to_string(),
            name: "Fix auth bug".to_string(),
            description: "Investigates and fixes the authentication middleware bug".to_string(),
        },
    )
    .await;

    // (1500ms) ToolUse: Read
    sleep(Duration::from_millis(1000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::ToolUse {
            tool: "Read".to_string(),
            args_hint: Some("src/auth/middleware.rs".to_string()),
        },
    )
    .await;

    // (3500ms) AgentSpawn: agent-02
    sleep(Duration::from_millis(2000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::AgentSpawn {
            agent_id: "agent-02".to_string(),
            name: "Add unit tests".to_string(),
            description: "Writes unit tests for the authentication module".to_string(),
        },
    )
    .await;

    // (4500ms) ToolUse: Edit
    sleep(Duration::from_millis(1000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::ToolUse {
            tool: "Edit".to_string(),
            args_hint: Some("src/auth/middleware.rs".to_string()),
        },
    )
    .await;

    // (6500ms) ToolUse: Bash
    sleep(Duration::from_millis(2000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::ToolUse {
            tool: "Bash".to_string(),
            args_hint: Some("cargo test auth::tests".to_string()),
        },
    )
    .await;

    // (9500ms) AgentSpawn: agent-03
    sleep(Duration::from_millis(3000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::AgentSpawn {
            agent_id: "agent-03".to_string(),
            name: "Code review".to_string(),
            description: "Reviews the auth changes for correctness and style".to_string(),
        },
    )
    .await;

    // (11500ms) AgentResult: agent-01
    sleep(Duration::from_millis(2000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::AgentResult {
            agent_id: "agent-01".to_string(),
        },
    )
    .await;

    // (13500ms) StatsUpdate
    sleep(Duration::from_millis(2000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::StatsUpdate {
            input_tokens: 14200,
            output_tokens: 3800,
            cost: 0.42,
        },
    )
    .await;

    // (16500ms) AgentResult: agent-02
    sleep(Duration::from_millis(3000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::AgentResult {
            agent_id: "agent-02".to_string(),
        },
    )
    .await;

    // (18500ms) Error
    sleep(Duration::from_millis(2000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::Error {
            message: "Lint check failed".to_string(),
        },
    )
    .await;

    // (21500ms) ToolUse: Edit
    sleep(Duration::from_millis(3000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::ToolUse {
            tool: "Edit".to_string(),
            args_hint: Some("src/lib.rs".to_string()),
        },
    )
    .await;

    // (23500ms) AgentResult: agent-03
    sleep(Duration::from_millis(2000)).await;
    send(
        &tx,
        session_id,
        StreamEvent::AgentResult {
            agent_id: "agent-03".to_string(),
        },
    )
    .await;

    // (23500ms) StatsUpdate (same timestamp as AgentResult: agent-03)
    send(
        &tx,
        session_id,
        StreamEvent::StatsUpdate {
            input_tokens: 28400,
            output_tokens: 7200,
            cost: 0.87,
        },
    )
    .await;

    // (28500ms) SessionEnded
    sleep(Duration::from_millis(5000)).await;
    let _ = tx
        .send(ReaderMessage::SessionEnded {
            session_id: session_id.to_string(),
        })
        .await;
}
