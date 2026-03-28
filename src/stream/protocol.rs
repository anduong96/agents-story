use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: Option<String>,
    pub text: Option<String>,
    pub name: Option<String>,
    pub input: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    #[allow(dead_code)]
    pub cache_creation_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RawMessage {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    #[allow(dead_code)]
    pub subtype: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub content_block: Option<ContentBlock>,
    #[allow(dead_code)]
    pub tool_name: Option<String>,
    #[allow(dead_code)]
    pub tool_input: Option<Value>,
    #[allow(dead_code)]
    pub tool_result: Option<Value>,
    pub usage: Option<Usage>,
    pub cost_usd: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StreamEvent {
    SessionInit { session_id: String, model: String },
    ToolUse { tool: String, args_hint: Option<String> },
    ToolResult { tool: String },
    AgentSpawn { agent_id: String, name: String, description: String },
    AgentResult { agent_id: String },
    StatsUpdate { input_tokens: u64, output_tokens: u64, cost: f64 },
    TextDelta { text: String },
    SessionEnd,
    Error { message: String },
}

#[allow(dead_code)]
pub fn parse_line(line: &str) -> Option<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let msg: RawMessage = serde_json::from_str(trimmed).ok()?;

    match msg.msg_type.as_deref() {
        Some("system") => {
            let session_id = msg.session_id?;
            let model = msg.model?;
            Some(StreamEvent::SessionInit { session_id, model })
        }
        Some("assistant") => {
            if let Some(cb) = msg.content_block {
                match cb.block_type.as_deref() {
                    Some("tool_use") => {
                        let tool = cb.name.unwrap_or_default();
                        if tool == "Agent" {
                            // Extract agent_id, name, description from input
                            let agent_id = cb.input.as_ref()
                                .and_then(|v| v.get("agent_id"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let name = cb.input.as_ref()
                                .and_then(|v| v.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Agent")
                                .to_string();
                            let description = cb.input.as_ref()
                                .and_then(|v| v.get("description"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            Some(StreamEvent::AgentSpawn { agent_id, name, description })
                        } else {
                            // Extract first string value from input as args_hint
                            let args_hint = cb.input.as_ref().and_then(|input| {
                                if let Some(obj) = input.as_object() {
                                    obj.values().find_map(|v| v.as_str().map(|s| s.to_string()))
                                } else {
                                    None
                                }
                            });
                            Some(StreamEvent::ToolUse { tool, args_hint })
                        }
                    }
                    Some("text") => {
                        let text = cb.text.unwrap_or_default();
                        Some(StreamEvent::TextDelta { text })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Some("result") => {
            let usage = msg.usage.unwrap_or(Usage {
                input_tokens: None,
                output_tokens: None,
                cache_read_input_tokens: None,
                cache_creation_input_tokens: None,
            });
            let input_tokens = usage.input_tokens.unwrap_or(0)
                + usage.cache_read_input_tokens.unwrap_or(0);
            let output_tokens = usage.output_tokens.unwrap_or(0);
            let cost = msg.cost_usd.unwrap_or(0.0);
            Some(StreamEvent::StatsUpdate { input_tokens, output_tokens, cost })
        }
        Some("error") => {
            let message = msg.extra.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Some(StreamEvent::Error { message })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_system_init() {
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
    fn test_parse_tool_use() {
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
    fn test_parse_agent_spawn() {
        let line = r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Agent","input":{"agent_id":"agent-001","name":"ResearchAgent","description":"Does research"}}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::AgentSpawn { agent_id, name, description } => {
                assert_eq!(agent_id, "agent-001");
                assert_eq!(name, "ResearchAgent");
                assert_eq!(description, "Does research");
            }
            _ => panic!("unexpected event: {:?}", event),
        }
    }

    #[test]
    fn test_parse_result_with_stats() {
        let line = r#"{"type":"result","cost_usd":0.42,"usage":{"input_tokens":1000,"output_tokens":500,"cache_read_input_tokens":200}}"#;
        let event = parse_line(line).expect("should parse");
        match event {
            StreamEvent::StatsUpdate { input_tokens, output_tokens, cost } => {
                assert_eq!(input_tokens, 1200);
                assert_eq!(output_tokens, 500);
                assert!((cost - 0.42).abs() < 1e-9);
            }
            _ => panic!("unexpected event: {:?}", event),
        }
    }

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
