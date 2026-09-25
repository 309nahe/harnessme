//! Provider abstractions and OpenAI-compatible HTTP client.

use std::error::Error;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::core::types::{Message, ToolCall, ToolDefinition};

/// Strongly typed errors that can occur during LLM provider communication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    /// Network or transport level failure.
    HttpError(String),
    /// Failure during JSON payload serialization or response deserialization.
    SerializationError(String),
    /// HTTP non-2xx status code returned by the provider endpoint.
    ApiError { status: u16, message: String },
    /// Provider returned an empty response or no choices.
    EmptyResponse,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpError(err) => write!(f, "HTTP transport error: {err}"),
            Self::SerializationError(err) => write!(f, "Serialization error: {err}"),
            Self::ApiError { status, message } => {
                write!(f, "API error (status {status}): {message}")
            }
            Self::EmptyResponse => write!(f, "Provider returned an empty response with no choices"),
        }
    }
}

impl Error for ProviderError {}

/// Represents the output returned by an LLM provider completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderResponse {
    /// The model generated a direct conversational text answer.
    Text(String),
    /// The model requested execution of one or more tool calls.
    ToolCalls(Vec<ToolCall>),
}

/// Abstract trait for LLM inference providers.
pub trait Provider: Send + Sync {
    /// Performs a synchronous chat completion given the conversation history and available tools.
    fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError>;
}

/// Request payload sent to OpenAI-compatible `/chat/completions` endpoints.
#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [ToolDefinition]>,
}

/// Response choice structure returned by OpenAI-compatible endpoints.
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

/// Synchronous HTTP client for OpenAI-compatible LLM endpoints.
///
/// Works with OpenAI, Ollama (`/v1`), vLLM, Groq, LocalAI, Mistral, and DeepSeek endpoints.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleProvider {
    api_key: Option<String>,
    base_url: String,
    model: String,
    temperature: f32,
    timeout_secs: u64,
}

impl OpenAiCompatibleProvider {
    /// Creates a new `OpenAiCompatibleProvider` targeting the default OpenAI API base URL (`https://api.openai.com/v1`).
    pub fn new(model: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model: model.into(),
            temperature: 0.7,
            timeout_secs: 60,
        }
    }

    /// Customizes the API key for bearer authentication.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets or clears the optional API key for bearer authentication.
    pub fn with_optional_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    /// Customizes the base endpoint URL (e.g. `http://localhost:11434/v1` for Ollama).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Customizes the sampling temperature (e.g. `0.0` for deterministic outputs).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Customizes the HTTP request timeout in seconds.
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Returns the configured API key, if present.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Returns the target base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the target model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns the configured temperature.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns the request timeout in seconds.
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    /// Helper to parse a JSON response body into a `ProviderResponse`.
    pub fn parse_response_json(json_str: &str) -> Result<ProviderResponse, ProviderError> {
        let response: ChatCompletionResponse = serde_json::from_str(json_str)
            .map_err(|err| ProviderError::SerializationError(err.to_string()))?;

        let first_choice = response
            .choices
            .into_iter()
            .next()
            .ok_or(ProviderError::EmptyResponse)?;

        if let Some(tool_calls) = first_choice.message.tool_calls {
            if !tool_calls.is_empty() {
                return Ok(ProviderResponse::ToolCalls(tool_calls));
            }
        }

        if let Some(content) = first_choice.message.content {
            return Ok(ProviderResponse::Text(content));
        }

        Ok(ProviderResponse::Text(String::new()))
    }
}

impl Provider for OpenAiCompatibleProvider {
    fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError> {
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let tools_payload = if tools.is_empty() { None } else { Some(tools) };

        let request_payload = ChatCompletionRequest {
            model: &self.model,
            messages,
            temperature: self.temperature,
            tools: tools_payload,
        };

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build();

        let mut request = agent
            .post(&endpoint)
            .set("Content-Type", "application/json");

        if let Some(ref api_key) = self.api_key {
            request = request.set("Authorization", &format!("Bearer {api_key}"));
        }

        let response = match request.send_json(&request_payload) {
            Ok(resp) => resp,
            Err(ureq::Error::Status(status, resp)) => {
                let message = resp
                    .into_string()
                    .unwrap_or_else(|_| format!("HTTP {status} with unreadable error payload"));
                return Err(ProviderError::ApiError { status, message });
            }
            Err(ureq::Error::Transport(transport_err)) => {
                return Err(ProviderError::HttpError(transport_err.to_string()));
            }
        };

        let response_body = response.into_string().map_err(|err| {
            ProviderError::HttpError(format!("Failed to read response body: {err}"))
        })?;

        Self::parse_response_json(&response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::FunctionDefinition;
    use serde_json::json;

    #[test]
    fn test_provider_error_display() {
        assert_eq!(
            ProviderError::HttpError("connect timeout".to_string()).to_string(),
            "HTTP transport error: connect timeout"
        );
        assert_eq!(
            ProviderError::SerializationError("invalid json".to_string()).to_string(),
            "Serialization error: invalid json"
        );
        assert_eq!(
            ProviderError::ApiError {
                status: 401,
                message: "Unauthorized".to_string()
            }
            .to_string(),
            "API error (status 401): Unauthorized"
        );
        assert_eq!(
            ProviderError::EmptyResponse.to_string(),
            "Provider returned an empty response with no choices"
        );
    }

    #[test]
    fn test_openai_request_payload_serialization() {
        let messages = vec![Message::user("Hello agent")];
        let req_without_tools = ChatCompletionRequest {
            model: "gpt-4o-mini",
            messages: &messages,
            temperature: 0.5,
            tools: None,
        };

        let json_str = serde_json::to_string(&req_without_tools).unwrap();
        assert!(!json_str.contains("\"tools\""));
        assert!(json_str.contains("\"gpt-4o-mini\""));

        let tools = vec![ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "calculator".to_string(),
                description: "Compute arithmetic".to_string(),
                parameters: json!({"type": "object"}),
            },
        }];

        let req_with_tools = ChatCompletionRequest {
            model: "gpt-4o-mini",
            messages: &messages,
            temperature: 0.5,
            tools: Some(&tools),
        };

        let json_with_tools_str = serde_json::to_string(&req_with_tools).unwrap();
        assert!(json_with_tools_str.contains("\"tools\""));
        assert!(json_with_tools_str.contains("\"calculator\""));
    }

    #[test]
    fn test_openai_response_parsing_text() {
        let json_raw = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you today?"
                    },
                    "finish_reason": "stop"
                }
            ]
        }"#;

        let response = OpenAiCompatibleProvider::parse_response_json(json_raw).unwrap();
        match response {
            ProviderResponse::Text(content) => {
                assert_eq!(content, "Hello! How can I help you today?");
            }
            ProviderResponse::ToolCalls(_) => panic!("Expected text response, got ToolCalls"),
        }
    }

    #[test]
    fn test_openai_response_parsing_tool_calls() {
        let json_raw = r#"{
            "id": "chatcmpl-456",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [
                            {
                                "id": "call_abc123",
                                "type": "function",
                                "function": {
                                    "name": "calculator",
                                    "arguments": "{\"a\": 10, \"b\": 20, \"op\": \"add\"}"
                                }
                            }
                        ]
                    },
                    "finish_reason": "tool_calls"
                }
            ]
        }"#;

        let response = OpenAiCompatibleProvider::parse_response_json(json_raw).unwrap();
        match response {
            ProviderResponse::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].id, "call_abc123");
                assert_eq!(calls[0].function.name, "calculator");
                assert_eq!(
                    calls[0].function.arguments,
                    "{\"a\": 10, \"b\": 20, \"op\": \"add\"}"
                );
            }
            ProviderResponse::Text(_) => panic!("Expected tool calls, got Text"),
        }
    }

    #[test]
    fn test_openai_response_parsing_empty_choices() {
        let json_raw = r#"{"choices": []}"#;
        let result = OpenAiCompatibleProvider::parse_response_json(json_raw);
        assert_eq!(result.unwrap_err(), ProviderError::EmptyResponse);
    }

    #[test]
    fn test_provider_builder_pattern() {
        let provider = OpenAiCompatibleProvider::new("llama3.1", Some("key-123".to_string()))
            .with_base_url("http://localhost:11434/v1")
            .with_temperature(0.2)
            .with_timeout(15);

        assert_eq!(provider.model(), "llama3.1");
        assert_eq!(provider.api_key(), Some("key-123"));
        assert_eq!(provider.base_url(), "http://localhost:11434/v1");
        assert!((provider.temperature() - 0.2).abs() < f32::EPSILON);
        assert_eq!(provider.timeout_secs(), 15);
    }

    #[test]
    fn test_mock_provider_implementation() {
        struct MockEchoProvider;

        impl Provider for MockEchoProvider {
            fn complete(
                &self,
                messages: &[Message],
                _tools: &[ToolDefinition],
            ) -> Result<ProviderResponse, ProviderError> {
                if let Some(last_msg) = messages.last() {
                    if let Some(ref text) = last_msg.content {
                        if text.contains("call_calc") {
                            return Ok(ProviderResponse::ToolCalls(vec![ToolCall::new(
                                "call_mock_1",
                                "calculator",
                                r#"{"expression":"2+2"}"#,
                            )]));
                        }
                        return Ok(ProviderResponse::Text(format!("Echo: {text}")));
                    }
                }
                Err(ProviderError::EmptyResponse)
            }
        }

        let provider = MockEchoProvider;
        let res1 = provider.complete(&[Message::user("Hello")], &[]).unwrap();
        assert_eq!(res1, ProviderResponse::Text("Echo: Hello".to_string()));

        let res2 = provider
            .complete(&[Message::user("call_calc now")], &[])
            .unwrap();
        match res2 {
            ProviderResponse::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].id, "call_mock_1");
            }
            _ => panic!("Expected tool calls"),
        }
    }
}
