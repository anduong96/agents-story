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

async fn spawn(tx: &mpsc::Sender<ReaderMessage>, sid: &str, id: &str, name: &str, desc: &str) {
    send(
        tx,
        sid,
        StreamEvent::AgentSpawn {
            agent_id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
        },
    )
    .await;
}

async fn tool(tx: &mpsc::Sender<ReaderMessage>, sid: &str, t: &str, hint: Option<&str>) {
    send(
        tx,
        sid,
        StreamEvent::ToolUse {
            tool: t.to_string(),
            args_hint: hint.map(|s| s.to_string()),
        },
    )
    .await;
}

async fn done(tx: &mpsc::Sender<ReaderMessage>, sid: &str, id: &str) {
    send(
        tx,
        sid,
        StreamEvent::AgentResult {
            agent_id: id.to_string(),
        },
    )
    .await;
}

async fn stats(tx: &mpsc::Sender<ReaderMessage>, sid: &str, inp: u64, out: u64, cost: f64) {
    send(
        tx,
        sid,
        StreamEvent::StatsUpdate {
            input_tokens: inp,
            output_tokens: out,
            cost,
        },
    )
    .await;
}

pub async fn run_demo(tx: mpsc::Sender<ReaderMessage>) {
    let s = "demo-session";

    // Init session
    send(
        &tx,
        s,
        StreamEvent::SessionInit {
            session_id: s.to_string(),
            model: "claude-opus-4-6".to_string(),
        },
    )
    .await;

    // === Phase 1: All 6 staff start working (rapid spawn) ===
    sleep(Duration::from_millis(500)).await;
    spawn(
        &tx,
        s,
        "a1",
        "Fix auth bug",
        "Fix authentication middleware",
    )
    .await;

    sleep(Duration::from_millis(400)).await;
    spawn(
        &tx,
        s,
        "a2",
        "Add tests",
        "Write unit tests for auth module",
    )
    .await;

    sleep(Duration::from_millis(400)).await;
    spawn(&tx, s, "a3", "Code review", "Review auth changes").await;

    sleep(Duration::from_millis(400)).await;
    spawn(&tx, s, "a4", "Update docs", "Update API documentation").await;

    sleep(Duration::from_millis(400)).await;
    spawn(
        &tx,
        s,
        "a5",
        "Refactor DB",
        "Refactor session storage queries",
    )
    .await;

    sleep(Duration::from_millis(400)).await;
    spawn(
        &tx,
        s,
        "a6",
        "Add logging",
        "Add structured logging to auth",
    )
    .await;

    // === Phase 2: Busy working — tool use activity ===
    sleep(Duration::from_millis(1500)).await;
    tool(&tx, s, "Read", Some("src/auth/middleware.rs")).await;

    sleep(Duration::from_millis(1200)).await;
    tool(&tx, s, "Grep", Some("session_token")).await;

    sleep(Duration::from_millis(1000)).await;
    tool(&tx, s, "Edit", Some("src/auth/middleware.rs")).await;

    sleep(Duration::from_millis(800)).await;
    tool(&tx, s, "Bash", Some("cargo test")).await;

    sleep(Duration::from_millis(1500)).await;
    tool(&tx, s, "Read", Some("src/db/sessions.rs")).await;

    sleep(Duration::from_millis(1000)).await;
    tool(&tx, s, "Edit", Some("src/db/sessions.rs")).await;
    stats(&tx, s, 18000, 5200, 0.56).await;

    // === Phase 3: Need more help — spawn 3 temp agents (exceed 6 staff) ===
    sleep(Duration::from_millis(2000)).await;
    spawn(
        &tx,
        s,
        "t1",
        "Perf testing",
        "Run performance benchmarks on auth flow",
    )
    .await;

    sleep(Duration::from_millis(500)).await;
    spawn(
        &tx,
        s,
        "t2",
        "Security audit",
        "Audit token handling for vulnerabilities",
    )
    .await;

    sleep(Duration::from_millis(500)).await;
    spawn(
        &tx,
        s,
        "t3",
        "CI pipeline",
        "Fix broken CI pipeline for auth module",
    )
    .await;

    // More tool activity while everyone works
    sleep(Duration::from_millis(1500)).await;
    tool(&tx, s, "Bash", Some("cargo bench auth")).await;

    sleep(Duration::from_millis(1200)).await;
    tool(&tx, s, "Read", Some("Cargo.toml")).await;

    sleep(Duration::from_millis(1000)).await;
    tool(&tx, s, "Edit", Some(".github/workflows/ci.yml")).await;
    stats(&tx, s, 38000, 11000, 1.18).await;

    // === Phase 4: Temp agents finish and leave ===
    sleep(Duration::from_millis(3000)).await;
    done(&tx, s, "t1").await; // temp leaves

    sleep(Duration::from_millis(2000)).await;
    done(&tx, s, "t3").await; // temp leaves

    sleep(Duration::from_millis(1500)).await;
    done(&tx, s, "t2").await; // temp leaves

    // === Phase 5: Some staff finish (return to lounge), others keep working ===
    sleep(Duration::from_millis(2000)).await;
    done(&tx, s, "a4").await; // staff → lounge

    sleep(Duration::from_millis(1500)).await;
    done(&tx, s, "a6").await; // staff → lounge

    sleep(Duration::from_millis(1000)).await;
    tool(&tx, s, "Edit", Some("src/auth/token.rs")).await;

    // === Phase 6: Remaining staff finish ===
    sleep(Duration::from_millis(3000)).await;
    done(&tx, s, "a1").await;
    stats(&tx, s, 62000, 18000, 1.94).await;

    sleep(Duration::from_millis(2000)).await;
    done(&tx, s, "a3").await;

    sleep(Duration::from_millis(2000)).await;
    done(&tx, s, "a2").await;

    sleep(Duration::from_millis(1500)).await;
    done(&tx, s, "a5").await;
    stats(&tx, s, 78000, 22000, 2.42).await;

    // === Phase 7: Idle in lounge for a bit, then session ends ===
    sleep(Duration::from_millis(8000)).await;
    let _ = tx
        .send(ReaderMessage::SessionEnded {
            session_id: s.to_string(),
        })
        .await;
}
