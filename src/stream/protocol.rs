use serde::{Deserialize, Deserializer};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Deserialization types for Claude Code JSONL
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub input: Option<Value>,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePayload {
    pub model: Option<String>,
    #[serde(default, deserialize_with = "deserialize_content")]
    pub content: Option<Vec<ContentItem>>,
    #[allow(dead_code)]
    pub usage: Option<Usage>,
}

/// Content can be a string (user messages) or an array of content blocks (assistant messages).
fn deserialize_content<'de, D>(deserializer: D) -> Result<Option<Vec<ContentItem>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(Value::Array(arr)) => {
            let items: Vec<ContentItem> = arr
                .into_iter()
                .filter_map(|v| serde_json::from_value(v).ok())
                .collect();
            Ok(Some(items))
        }
        Some(Value::String(s)) => Ok(Some(vec![ContentItem {
            item_type: Some("text".to_string()),
            id: None,
            name: None,
            input: None,
            text: Some(s),
        }])),
        Some(_) => Ok(None),
    }
}

/// Supports both real Claude Code format and legacy/demo format.
///
/// Real format: `{ "type": "assistant", "sessionId": "...", "message": { "model": "...", "content": [...] } }`
/// Legacy format: `{ "type": "assistant", "content_block": { "type": "tool_use", ... } }`
#[derive(Debug, Deserialize)]
pub struct RawMessage {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,

    /// Real format uses camelCase `sessionId`, legacy uses `session_id`.
    #[serde(rename = "sessionId", alias = "session_id")]
    pub session_id: Option<String>,

    /// Real format: nested message payload.
    pub message: Option<MessagePayload>,

    /// Legacy format: top-level model.
    pub model: Option<String>,

    /// Legacy format: single content block.
    pub content_block: Option<ContentItem>,

    /// Legacy format: top-level usage.
    pub usage: Option<Usage>,

    /// Legacy format: top-level cost.
    pub cost_usd: Option<f64>,
}

// ---------------------------------------------------------------------------
// Stream events (produced by the parser, consumed by main.rs)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StreamEvent {
    SessionInit {
        session_id: String,
        model: String,
    },
    ToolUse {
        tool: String,
        args_hint: Option<String>,
    },
    ToolResult {
        tool: String,
    },
    AgentSpawn {
        agent_id: String,
        name: String,
        description: String,
    },
    AgentResult {
        agent_id: String,
    },
    StatsUpdate {
        input_tokens: u64,
        output_tokens: u64,
        cost: f64,
    },
    TextDelta {
        text: String,
    },
    UserPrompt {
        text: String,
    },
    SessionEnd,
    Error {
        message: String,
    },
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

pub fn parse_line(line: &str) -> Option<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let msg: RawMessage = serde_json::from_str(trimmed).ok()?;

    match msg.msg_type.as_deref() {
        Some("assistant") => parse_assistant(&msg),
        Some("user") => parse_user(&msg),
        Some("system") => parse_legacy_system(&msg),
        Some("result") => parse_legacy_result(&msg),
        Some("error") => {
            let message = msg
                .session_id
                .as_deref()
                .unwrap_or("unknown error")
                .to_string();
            Some(StreamEvent::Error { message })
        }
        _ => None,
    }
}

/// Parse an assistant message. Tries real format first, then legacy.
fn parse_assistant(msg: &RawMessage) -> Option<StreamEvent> {
    // Real format: message.content[] array
    if let Some(ref payload) = msg.message {
        if let Some(ref content) = payload.content {
            for item in content {
                if let Some(event) = parse_content_item(item) {
                    return Some(event);
                }
            }
        }

        // No tool_use or text — if we have model info, emit SessionInit.
        if let Some(ref model) = payload.model {
            return Some(StreamEvent::SessionInit {
                session_id: msg.session_id.clone().unwrap_or_default(),
                model: model.clone(),
            });
        }

        return None;
    }

    // Legacy format: single content_block
    if let Some(ref cb) = msg.content_block {
        return parse_content_item(cb);
    }

    None
}

/// Parse a single content item (tool_use, text, etc.)
fn parse_content_item(item: &ContentItem) -> Option<StreamEvent> {
    match item.item_type.as_deref() {
        Some("tool_use") => {
            let tool = item.name.as_deref().unwrap_or_default();

            if tool == "Agent" {
                let input = item.input.as_ref();

                // Real format: input has `description` (short) and `prompt` (full task)
                // Legacy format: input has `agent_id`, `name`, `description`
                let agent_id = input
                    .and_then(|v| v.get("agent_id"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .or_else(|| item.id.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let name = input
                    .and_then(|v| v.get("name"))
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        input
                            .and_then(|v| v.get("description"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("Agent")
                    .to_string();

                let description = input
                    .and_then(|v| v.get("description"))
                    .and_then(|v| v.as_str())
                    .or_else(|| input.and_then(|v| v.get("prompt")).and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();

                Some(StreamEvent::AgentSpawn {
                    agent_id,
                    name,
                    description,
                })
            } else {
                let args_hint = item.input.as_ref().and_then(|input| {
                    input
                        .as_object()?
                        .values()
                        .find_map(|v| v.as_str().map(String::from))
                });
                Some(StreamEvent::ToolUse {
                    tool: tool.to_string(),
                    args_hint,
                })
            }
        }
        Some("text") => {
            let text = item.text.clone().unwrap_or_default();
            if text.is_empty() {
                None
            } else {
                Some(StreamEvent::TextDelta { text })
            }
        }
        _ => None,
    }
}

/// Parse user message to extract the prompt text.
/// Content can be a string or an array of content blocks.
fn parse_user(msg: &RawMessage) -> Option<StreamEvent> {
    let payload = msg.message.as_ref()?;
    let content = payload.content.as_ref()?;

    for item in content {
        if item.item_type.as_deref() == Some("text") || item.item_type.is_none() {
            if let Some(ref text) = item.text {
                if !text.is_empty() {
                    // Truncate to a reasonable length for display
                    let truncated = if text.len() > 80 {
                        format!("{}...", &text[..77])
                    } else {
                        text.clone()
                    };
                    return Some(StreamEvent::UserPrompt { text: truncated });
                }
            }
        }
    }
    None
}

/// Legacy format: `{"type":"system","session_id":"...","model":"..."}`
fn parse_legacy_system(msg: &RawMessage) -> Option<StreamEvent> {
    let session_id = msg.session_id.clone()?;
    let model = msg.model.clone()?;
    Some(StreamEvent::SessionInit { session_id, model })
}

/// Legacy format: `{"type":"result","cost_usd":0.42,"usage":{...}}`
fn parse_legacy_result(msg: &RawMessage) -> Option<StreamEvent> {
    let usage = msg.usage.as_ref()?;
    let input_tokens = usage.input_tokens.unwrap_or(0) + usage.cache_read_input_tokens.unwrap_or(0);
    let output_tokens = usage.output_tokens.unwrap_or(0);
    let cost = msg.cost_usd.unwrap_or(0.0);
    Some(StreamEvent::StatsUpdate {
        input_tokens,
        output_tokens,
        cost,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Real Claude Code format tests ---

    #[test]
    fn test_parse_real_tool_use() {
        let line = r#"{"type":"assistant","sessionId":"abc","message":{"model":"claude-opus-4-6","content":[{"type":"tool_use","id":"toolu_01","name":"Bash","input":{"command":"git status","description":"Show status"}}]}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::ToolUse { tool, args_hint } => {
                assert_eq!(tool, "Bash");
                assert!(args_hint.is_some());
            }
            _ => panic!("expected ToolUse, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_real_agent_spawn() {
        let line = r#"{"type":"assistant","sessionId":"abc","message":{"model":"claude-opus-4-6","content":[{"type":"tool_use","id":"toolu_02","name":"Agent","input":{"description":"Research auth bug","prompt":"Find and fix the auth bug in login.rs"}}]}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::AgentSpawn {
                agent_id,
                name,
                description,
            } => {
                assert_eq!(agent_id, "toolu_02");
                assert_eq!(name, "Research auth bug");
                assert_eq!(description, "Research auth bug");
            }
            _ => panic!("expected AgentSpawn, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_real_thinking_emits_session_init() {
        let line = r#"{"type":"assistant","sessionId":"sess-1","message":{"model":"claude-opus-4-6","content":[{"type":"thinking","thinking":"let me think..."}]}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::SessionInit { session_id, model } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(model, "claude-opus-4-6");
            }
            _ => panic!("expected SessionInit, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_real_skips_file_history() {
        let line = r#"{"type":"file-history-snapshot","messageId":"abc"}"#;
        assert!(parse_line(line).is_none());
    }

    #[test]
    fn test_parse_real_user_prompt() {
        let line =
            r#"{"type":"user","sessionId":"abc","message":{"role":"user","content":"do stuff"}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::UserPrompt { text } => {
                assert_eq!(text, "do stuff");
            }
            _ => panic!("expected UserPrompt, got {:?}", event),
        }
    }

    // --- Legacy format tests (backward compat for demo) ---

    #[test]
    fn test_parse_legacy_system_init() {
        let line = r#"{"type":"system","session_id":"abc123","model":"claude-opus-4-6"}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::SessionInit { session_id, model } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(model, "claude-opus-4-6");
            }
            _ => panic!("unexpected event: {:?}", event),
        }
    }

    #[test]
    fn test_parse_legacy_tool_use() {
        let line = r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Read","input":{"file_path":"/src/main.rs"}}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::ToolUse { tool, args_hint } => {
                assert_eq!(tool, "Read");
                assert_eq!(args_hint, Some("/src/main.rs".to_string()));
            }
            _ => panic!("unexpected event: {:?}", event),
        }
    }

    #[test]
    fn test_parse_legacy_agent_spawn() {
        let line = r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Agent","input":{"agent_id":"agent-001","name":"ResearchAgent","description":"Does research"}}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::AgentSpawn {
                agent_id,
                name,
                description,
            } => {
                assert_eq!(agent_id, "agent-001");
                assert_eq!(name, "ResearchAgent");
                assert_eq!(description, "Does research");
            }
            _ => panic!("unexpected event: {:?}", event),
        }
    }

    #[test]
    fn test_parse_legacy_result_with_stats() {
        let line = r#"{"type":"result","cost_usd":0.42,"usage":{"input_tokens":1000,"output_tokens":500,"cache_read_input_tokens":200}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::StatsUpdate {
                input_tokens,
                output_tokens,
                cost,
            } => {
                assert_eq!(input_tokens, 1200);
                assert_eq!(output_tokens, 500);
                assert!((cost - 0.42).abs() < 1e-9);
            }
            _ => panic!("unexpected event: {:?}", event),
        }
    }

    // --- Edge cases ---

    #[test]
    fn test_parse_empty_line() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
    }

    #[test]
    fn test_parse_invalid_json() {
        assert!(parse_line("not json at all {{{").is_none());
    }

    #[test]
    fn test_parse_unknown_type() {
        let line = r#"{"type":"something_unknown","data":"value"}"#;
        assert!(parse_line(line).is_none());
    }
}
