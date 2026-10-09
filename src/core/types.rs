//! Universal message protocol and tool data structures.
//!
//! This module defines the intermediate representation (IR) shared by
//! every layer of the harness. It mirrors the OpenAI tool-calling
//! protocol so that messages can be serialized to, and deserialized
//! from, any OpenAI-compatible inference endpoint (OpenAI, Ollama,
//! vLLM, Groq, LocalAI, ...).
//!
//! All types derive `Serialize`/`Deserialize` via `serde` and are
//! wire-compatible with the `POST /v1/chat/completions` payload shape.

use serde::{Deserialize, Serialize};

/// The actor in a conversation turn.
///
/// Serialized in lowercase form (`"system"`, `""user`, ...) to match
/// the OpenAI chat protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Directives and persona definitions for the model.
    System,
    /// Input provided by the human user.
    User,
    /// Responses and generated tool calls emitted by the LLM.
    Assistant,
    /// Outputs returned from executed tools matching a prior [`ToolCall`].
    Tool,
}

/// A structured request from the LLM to invoke a specific tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier of the call, used to correlate [`Role::Tool`] replies.
    pub id: String,
    /// The function invocation requested by the model.
    pub function: FunctionCall,
}

/// The function part of a [`ToolCall`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the tool to invoke; must match a registered [`crate::core::types::ToolDefinition`] name.
    pub name: String,
    /// Raw JSON string arguments as produced by the model.
    ///
    /// Kept as a string (not parsed eagerly) to preserve the exact
    /// wire format; the tool registry parses it at execution time.
    pub arguments: String,
}

/// The JSON schema representation of a tool advertised to the LLM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Object type discriminator; usually `"function"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Static metadata and JSON schema describing the tool.
    pub function: FunctionDefinition,
}

/// Static metadata and parameter schema of a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Unique tool identifier (e.g. `"calculator"`).
    pub name: String,
    /// Human/LLM-readable description of the tool's behavior.
    pub description: String,
    /// JSON Schema (`serde_json::Value`) describing the input parameters.
    pub parameters: serde_json::Value,
}

/// The result of executing a tool, ready to be recorded into history.
///
/// `content` holds the tool output on success, or a human/LLM-readable
/// error message on failure; either way the agent loop feeds it back to
/// the model as a `Role::Tool` message so the LLM can self-correct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Identifier of the originating [`ToolCall`].
    pub tool_call_id: String,
    /// Output text of the tool, or an error message on failure.
    pub content: String,
}

/// A single turn in the conversation history.
///
/// Optional fields are skipped during serialization so that simple
/// user/assistant messages stay free of `null` clutter on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The actor that produced this turn.
    pub role: Role,
    /// Text content of the turn; may be absent for pure tool-call turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Tool calls emitted by the assistant, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Correlation ID for `Role::Tool` replies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Creates a system message with the given content.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates a user message with the given content.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates an assistant message with plain text content.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates an assistant message carrying tool calls.
    ///
    /// `content` may be empty on the wire for pure tool-call turns.
    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    /// Creates a `Role::Tool` reply correlated to a tool call ID.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn role_serializes_lowercase() {
        assert_eq!(serde_json::to_value(Role::System).unwrap(), json!("system"));
        assert_eq!(serde_json::to_value(Role::User).unwrap(), json!("user"));
        assert_eq!(serde_json::to_value(Role::Assistant).unwrap(), json!("assistant"));
        assert_eq!(serde_json::to_value(Role::Tool).unwrap(), json!("tool"));
    }

    #[test]
    fn role_deserializes_from_wire_format() {
        assert_eq!(serde_json::from_value::<Role>(json!("user")).unwrap(), Role::User);
        assert!(serde_json::from_value::<Role>(json!("unknown")).is_err());
    }

    #[test]
    fn simple_message_round_trip_and_omits_empty_fields() {
        let msg = Message::user("hello harness");
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(value, json!({"role": "user", "content": "hello harness"}));

        let back: Message = serde_json::from_value(value).unwrap();
        assert_eq!(back.role, Role::User);
        assert_eq!(back.content.as_deref(), Some("hello harness"));
        assert!(back.tool_calls.is_none());
        assert!(back.tool_call_id.is_none());
    }

    #[test]
    fn tool_call_message_round_trip() {
        let msg = Message::assistant_with_tool_calls(vec![ToolCall {
            id: "call_1".to_string(),
            function: FunctionCall {
                name: "calculator".to_string(),
                arguments: "{\"expr\": \"2+2\"}".to_string(),
            },
        }]);
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(
            value,
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_1",
                    "function": {"name": "calculator", "arguments": "{\"expr\": \"2+2\"}"}
                }]
            })
        );

        let back: Message = serde_json::from_value(value).unwrap();
        let calls = back.tool_calls.expect("tool calls must survive round trip");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].function.name, "calculator");
        assert_eq!(calls[0].function.arguments, "{\"expr\": \"2+2\"}");
    }

    #[test]
    fn tool_reply_message_round_trip() {
        let msg = Message::tool("call_1", "4");
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(
            value,
            json!({"role": "tool", "content": "4", "tool_call_id": "call_1"})
        );

        let back: Message = serde_json::from_value(value).unwrap();
        assert_eq!(back.role, Role::Tool);
        assert_eq!(back.tool_call_id.as_deref(), Some("call_1"));
        assert_eq!(back.content.as_deref(), Some("4"));
    }

    #[test]
    fn tool_definition_serializes_openai_schema_shape() {
        let def = ToolDefinition {
            kind: "function".to_string(),
            function: FunctionDefinition {
                name: "echo".to_string(),
                description: "Echoes the input".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {"text": {"type": "string"}},
                    "required": ["text"]
                }),
            },
        };
        let value = serde_json::to_value(&def).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "function",
                "function": {
                    "name": "echo",
                    "description": "Echoes the input",
                    "parameters": {
                        "type": "object",
                        "properties": {"text": {"type": "string"}},
                        "required": ["text"]
                    }
                }
            })
        );

        let back: ToolDefinition = serde_json::from_value(value).unwrap();
        assert_eq!(back.kind, "function");
        assert_eq!(back.function.name, "echo");
        assert_eq!(back.function.parameters["properties"]["text"]["type"], "string");
    }

    #[test]
    fn tool_result_round_trip() {
        let result = ToolResult {
            tool_call_id: "call_42".to_string(),
            content: "division by zero".to_string(),
        };
        let value = serde_json::to_value(&result).unwrap();
        let back: ToolResult = serde_json::from_value(value).unwrap();
        assert_eq!(back.tool_call_id, result.tool_call_id);
        assert_eq!(back.content, result.content);
    }

    #[test]
    fn message_constructors_set_roles() {
        assert_eq!(Message::system("s").role, Role::System);
        assert_eq!(Message::user("u").role, Role::User);
        assert_eq!(Message::assistant("a").role, Role::Assistant);
        assert_eq!(
            Message::assistant_with_tool_calls(vec![]).role,
            Role::Assistant
        );
        assert_eq!(Message::tool("id", "ok").role, Role::Tool);
    }

    #[test]
    fn all_roles_round_trip_through_json_string() {
        let messages = [
            Message::system("you are a harness"),
            Message::user("hello"),
            Message::assistant("hi there"),
            Message::assistant_with_tool_calls(vec![ToolCall {
                id: "call_1".to_string(),
                function: FunctionCall {
                    name: "echo".to_string(),
                    arguments: "{\"text\": \"hi\"}".to_string(),
                },
            }]),
            Message::tool("call_1", "hi"),
        ];

        for original in &messages {
            let json_str = serde_json::to_string(original).unwrap();
            let back: Message = serde_json::from_str(&json_str).unwrap();
            assert_eq!(&back, original, "round trip failed for {json_str}");
        }
    }

    #[test]
    fn assistant_message_with_content_and_tool_calls_round_trip() {
        let original = Message {
            role: Role::Assistant,
            content: Some("let me compute that".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_7".to_string(),
                function: FunctionCall {
                    name: "calculator".to_string(),
                    arguments: "{\"expr\": \"6*7\"}".to_string(),
                },
            }]),
            tool_call_id: None,
        };
        let value = serde_json::to_value(&original).unwrap();
        assert_eq!(
            value,
            json!({
                "role": "assistant",
                "content": "let me compute that",
                "tool_calls": [{
                    "id": "call_7",
                    "function": {"name": "calculator", "arguments": "{\"expr\": \"6*7\"}"}
                }]
            })
        );

        let back: Message = serde_json::from_value(value).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn assistant_message_without_content_or_tool_calls_omits_both() {
        let msg = Message {
            role: Role::Assistant,
            content: None,
            tool_calls: None,
            tool_call_id: None,
        };
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(value, json!({"role": "assistant"}));

        let back: Message = serde_json::from_value(value).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn multiple_tool_calls_round_trip_in_order() {
        let calls = vec![
            ToolCall {
                id: "call_a".to_string(),
                function: FunctionCall {
                    name: "echo".to_string(),
                    arguments: "{}".to_string(),
                },
            },
            ToolCall {
                id: "call_b".to_string(),
                function: FunctionCall {
                    name: "calculator".to_string(),
                    arguments: "{\"expr\": \"1+1\"}".to_string(),
                },
            },
        ];
        let original = Message::assistant_with_tool_calls(calls);
        let json_str = serde_json::to_string(&original).unwrap();
        let back: Message = serde_json::from_str(&json_str).unwrap();
        assert_eq!(back, original);

        let calls = back.tool_calls.unwrap();
        assert_eq!(calls[0].id, "call_a");
        assert_eq!(calls[1].id, "call_b");
    }

    #[test]
    fn tool_call_round_trip_preserves_exact_wire_fields() {
        let original = ToolCall {
            id: "call_abc".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: "{\"city\": \"Paris\", \"unit\": \"c\"}".to_string(),
            },
        };
        let value = serde_json::to_value(&original).unwrap();
        assert_eq!(
            value,
            json!({
                "id": "call_abc",
                "function": {
                    "name": "get_weather",
                    "arguments": "{\"city\": \"Paris\", \"unit\": \"c\"}"
                }
            })
        );

        let back: ToolCall = serde_json::from_value(value).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn tool_definition_round_trips_from_canonical_openai_json() {
        let wire_json = r#"{
            "type": "function",
            "function": {
                "name": "calculator",
                "description": "Evaluates arithmetic expressions",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expr": {"type": "string", "description": "Expression to evaluate"}
                    },
                    "required": ["expr"]
                }
            }
        }"#;
        let parsed: ToolDefinition = serde_json::from_str(wire_json).unwrap();
        assert_eq!(parsed.kind, "function");
        assert_eq!(parsed.function.name, "calculator");
        assert_eq!(parsed.function.description, "Evaluates arithmetic expressions");

        // Re-serialization must reproduce the exact wire field names.
        let back = serde_json::to_string_pretty(&parsed).unwrap();
        let reparsed: ToolDefinition = serde_json::from_str(&back).unwrap();
        assert_eq!(reparsed, parsed);
        let value = serde_json::to_value(&parsed).unwrap();
        assert!(value.get("type").is_some());
        assert!(value.get("function").is_some());
        assert!(value["function"].get("name").is_some());
        assert!(value["function"].get("description").is_some());
        assert!(value["function"].get("parameters").is_some());
    }

    #[test]
    fn tool_definition_with_empty_schema_round_trip() {
        let original = ToolDefinition {
            kind: "function".to_string(),
            function: FunctionDefinition {
                name: "ping".to_string(),
                description: "No-op liveness probe".to_string(),
                parameters: json!({"type": "object", "properties": {}}),
            },
        };
        let json_str = serde_json::to_string(&original).unwrap();
        let back: ToolDefinition = serde_json::from_str(&json_str).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn malformed_wire_json_is_rejected() {
        // Message without a role.
        assert!(serde_json::from_str::<Message>(r#"{"content": "x"}"#).is_err());
        // Message with an unknown role.
        assert!(serde_json::from_str::<Message>(r#"{"role": "bot"}"#).is_err());
        // ToolCall without an id.
        assert!(serde_json::from_str::<ToolCall>(r#"{"function": {"name": "e", "arguments": "{}"}}"#).is_err());
        // FunctionCall without arguments.
        assert!(serde_json::from_str::<FunctionCall>(r#"{"name": "e"}"#).is_err());
        // ToolDefinition without the "type" discriminator.
        assert!(serde_json::from_str::<ToolDefinition>(
            r#"{"function": {"name": "e", "description": "d", "parameters": {}}}"#
        )
        .is_err());
        // FunctionDefinition without parameters.
        assert!(serde_json::from_str::<FunctionDefinition>(
            r#"{"name": "e", "description": "d"}"#
        )
        .is_err());
    }

    #[test]
    fn empty_and_unicode_payloads_survive_round_trip() {
        let original = ToolCall {
            id: String::new(),
            function: FunctionCall {
                name: "echo".to_string(),
                arguments: String::new(),
            },
        };
        let back: ToolCall =
            serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
        assert_eq!(back, original);

        let msg = Message::user("héllo — 世界 \u{1F9E0}");
        let back: Message =
            serde_json::from_str(&serde_json::to_string(&msg).unwrap()).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn full_conversation_history_round_trip() {
        let history = vec![
            Message::system("You are a helpful harness."),
            Message::user("what is 2+2?"),
            Message::assistant_with_tool_calls(vec![ToolCall {
                id: "call_1".to_string(),
                function: FunctionCall {
                    name: "calculator".to_string(),
                    arguments: "{\"expr\": \"2+2\"}".to_string(),
                },
            }]),
            Message::tool("call_1", "4"),
            Message::assistant("2+2 equals 4."),
        ];
        let json_str = serde_json::to_string(&history).unwrap();
        let back: Vec<Message> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(back, history);
    }
}
