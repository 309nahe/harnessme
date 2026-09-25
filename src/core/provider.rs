//! Provider abstractions and OpenAI-compatible HTTP client.

use std::error::Error;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::core::types::{Message, Role, ToolCall, ToolDefinition};

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

impl<P: Provider + ?Sized> Provider for Box<P> {
    fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError> {
        (**self).complete(messages, tools)
    }
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

/// Synchronous HTTP client for Google Antigravity (AGY) endpoints and language servers.
///
/// Supports local Antigravity Language Server addresses (`ANTIGRAVITY_LS_ADDRESS`),
/// cloud gateway endpoints (`ANTIGRAVITY_BASE_URL`), bearer token authentication,
/// and `X-Antigravity-CSRF-Token` / `X-Antigravity-Source` header injection.
#[derive(Debug, Clone)]
pub struct AntigravityProvider {
    api_key: Option<String>,
    account_email: Option<String>,
    base_url: String,
    csrf_token: Option<String>,
    model: String,
    temperature: f32,
    timeout_secs: u64,
    source_metadata: Option<String>,
}

impl AntigravityProvider {
    /// Default model preset for Antigravity provider (Google AI Pro Flagship).
    pub const DEFAULT_MODEL: &'static str = "gemini-3.1-pro-high";
    /// Default connection label for native Google AI Pro subscription via agy.
    pub const DEFAULT_BASE_URL: &'static str = "Google Cloud (Antigravity Native)";

    /// Curated Google Gemini & Antigravity model catalogue with descriptions.
    pub const SUPPORTED_MODELS: &'static [(&'static str, &'static str)] = &[
        (
            "gemini-3.1-pro-high",
            "Gemini 3.1 Pro (High) - Google AI Pro Flagship (Deep reasoning & code)",
        ),
        (
            "gemini-3.1-pro-low",
            "Gemini 3.1 Pro (Low) - Lightweight reasoning & fast completions",
        ),
        (
            "gemini-3.8-flash-high",
            "Gemini 3.8 Flash (High) - Fast, capable multimodal model",
        ),
        (
            "gemini-3.8-flash-medium",
            "Gemini 3.8 Flash (Medium) - Balanced speed & reasoning",
        ),
        (
            "gemini-3.8-flash-low",
            "Gemini 3.8 Flash (Low) - Fast, low-latency completions",
        ),
        (
            "gemini-3.7-flash-high",
            "Gemini 3.7 Flash (High) - Extended reasoning & low latency",
        ),
        (
            "gemini-3.7-flash-medium",
            "Gemini 3.7 Flash (Medium) - Balanced reasoning & latency",
        ),
        (
            "gemini-3.7-flash-low",
            "Gemini 3.7 Flash (Low) - Quick responses",
        ),
        (
            "gemini-3.6-flash-high",
            "Gemini 3.6 Flash (High) - Ultra-low latency conversational model",
        ),
        (
            "gemini-3.6-flash-medium",
            "Gemini 3.6 Flash (Medium) - Fast conversational model",
        ),
        (
            "gemini-3.6-flash-low",
            "Gemini 3.6 Flash (Low) - Instant conversational responses",
        ),
        (
            "claude-sonnet-4-6",
            "Claude Sonnet 4.6 (Thinking - Google AI Pro Gateway)",
        ),
        (
            "claude-opus-4-6-thinking",
            "Claude Opus 4.6 (Thinking - Google AI Pro Gateway)",
        ),
        (
            "gpt-oss-120b-medium",
            "GPT-OSS 120B (Medium - Open-source weight gateway)",
        ),
    ];

    /// Resolves a numeric shortcut index or alias into a canonical model name if matched.
    pub fn resolve_model_name(input: &str) -> String {
        let trimmed = input.trim();
        match trimmed {
            "1" | "pro" => "gemini-3.1-pro-high".to_string(),
            "2" => "gemini-3.1-pro-low".to_string(),
            "3" | "flash" => "gemini-3.8-flash-high".to_string(),
            "4" => "gemini-3.8-flash-medium".to_string(),
            "5" => "gemini-3.8-flash-low".to_string(),
            "6" => "gemini-3.7-flash-high".to_string(),
            "7" => "gemini-3.7-flash-medium".to_string(),
            "8" => "gemini-3.7-flash-low".to_string(),
            "9" => "gemini-3.6-flash-high".to_string(),
            "10" => "gemini-3.6-flash-medium".to_string(),
            "11" => "gemini-3.6-flash-low".to_string(),
            "12" | "sonnet" => "claude-sonnet-4-6".to_string(),
            "13" | "opus" => "claude-opus-4-6-thinking".to_string(),
            "14" => "gpt-oss-120b-medium".to_string(),
            other => other.to_string(),
        }
    }

    /// Creates a new `AntigravityProvider` with the specified model name.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            api_key: None,
            account_email: None,
            base_url: Self::DEFAULT_BASE_URL.to_string(),
            csrf_token: None,
            model: model.into(),
            temperature: 0.7,
            timeout_secs: 60,
            source_metadata: None,
        }
    }

    /// Attempts to discover local Google account email from `~/.gemini/google_accounts.json`
    /// or other local Antigravity configuration files.
    pub fn read_local_google_account() -> Option<String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        let path = std::path::Path::new(&home)
            .join(".gemini")
            .join("google_accounts.json");
        let content = std::fs::read_to_string(path).ok()?;
        let value: serde_json::Value = serde_json::from_str(&content).ok()?;
        let email = value.get("active")?.as_str()?.trim();
        if email.is_empty() {
            None
        } else {
            Some(email.to_string())
        }
    }

    /// Attempts to discover local OAuth access/bearer token from `~/.gemini/oauth_creds.json`.
    pub fn read_local_oauth_token() -> Option<String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        let path = std::path::Path::new(&home)
            .join(".gemini")
            .join("oauth_creds.json");
        let content = std::fs::read_to_string(path).ok()?;
        let value: serde_json::Value = serde_json::from_str(&content).ok()?;
        let token = value.get("access_token")?.as_str()?.trim();
        if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        }
    }

    /// Automatically constructs an `AntigravityProvider` discovering configuration from
    /// environment variables and local Google/Antigravity credentials:
    /// - Account: `ANTIGRAVITY_ACCOUNT`, `GOOGLE_ACCOUNT`, `AGY_ACCOUNT`, or `~/.gemini/google_accounts.json`
    /// - Base URL: `ANTIGRAVITY_BASE_URL`, `AGY_BASE_URL`, or `http://{ANTIGRAVITY_LS_ADDRESS}/v1`
    /// - API Key: `ANTIGRAVITY_API_KEY`, `AGY_API_KEY`, or `~/.gemini/oauth_creds.json`
    /// - CSRF Token: `ANTIGRAVITY_CSRF_TOKEN`
    /// - Model: `ANTIGRAVITY_MODEL`, `HARNESS_MODEL`, or `DEFAULT_MODEL` (`gemini-2.5-flash`)
    /// - Source Metadata: `ANTIGRAVITY_SOURCE_METADATA`
    pub fn from_env() -> Self {
        let account_email = std::env::var("ANTIGRAVITY_ACCOUNT")
            .or_else(|_| std::env::var("GOOGLE_ACCOUNT"))
            .or_else(|_| std::env::var("AGY_ACCOUNT"))
            .ok()
            .or_else(Self::read_local_google_account);

        let api_key = std::env::var("ANTIGRAVITY_API_KEY")
            .or_else(|_| std::env::var("AGY_API_KEY"))
            .ok()
            .or_else(Self::read_local_oauth_token);

        let base_url = std::env::var("ANTIGRAVITY_BASE_URL")
            .or_else(|_| std::env::var("AGY_BASE_URL"))
            .unwrap_or_else(|_| Self::DEFAULT_BASE_URL.to_string());

        let csrf_token = std::env::var("ANTIGRAVITY_CSRF_TOKEN").ok();
        let source_metadata = std::env::var("ANTIGRAVITY_SOURCE_METADATA").ok();

        let model = std::env::var("ANTIGRAVITY_MODEL")
            .or_else(|_| std::env::var("HARNESS_MODEL"))
            .unwrap_or_else(|_| Self::DEFAULT_MODEL.to_string());

        let temperature = std::env::var("HARNESS_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.7);

        Self {
            api_key,
            account_email,
            base_url,
            csrf_token,
            model,
            temperature,
            timeout_secs: 60,
            source_metadata,
        }
    }

    /// Sets the Google account email associated with Antigravity.
    pub fn with_account(mut self, email: impl Into<String>) -> Self {
        self.account_email = Some(email.into());
        self
    }

    /// Sets or clears the optional Google account email.
    pub fn with_optional_account(mut self, email: Option<String>) -> Self {
        self.account_email = email;
        self
    }

    /// Sets the API key for bearer authentication.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets or clears the optional API key.
    pub fn with_optional_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    /// Sets the endpoint base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets the CSRF token for Antigravity Language Server requests.
    pub fn with_csrf_token(mut self, csrf_token: impl Into<String>) -> Self {
        self.csrf_token = Some(csrf_token.into());
        self
    }

    /// Sets or clears the optional CSRF token.
    pub fn with_optional_csrf_token(mut self, csrf_token: Option<String>) -> Self {
        self.csrf_token = csrf_token;
        self
    }

    /// Sets the model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Sets the HTTP request timeout in seconds.
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Sets optional source tracking metadata.
    pub fn with_source_metadata(mut self, source: impl Into<String>) -> Self {
        self.source_metadata = Some(source.into());
        self
    }

    /// Returns the Google account email, if configured.
    pub fn account_email(&self) -> Option<&str> {
        self.account_email.as_deref()
    }

    /// Returns the API key, if configured.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Returns the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the CSRF token, if configured.
    pub fn csrf_token(&self) -> Option<&str> {
        self.csrf_token.as_deref()
    }

    /// Returns the model identifier.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns the temperature.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns the timeout in seconds.
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    /// Returns the source metadata, if configured.
    pub fn source_metadata(&self) -> Option<&str> {
        self.source_metadata.as_deref()
    }

    /// Attempts to locate the `agy` CLI binary on the system.
    pub fn find_agy_binary() -> Option<std::path::PathBuf> {
        if let Ok(path) = std::env::var("AGY_BIN") {
            let p = std::path::PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let local_bin = std::path::Path::new(&home)
                .join(".local")
                .join("bin")
                .join("agy");
            if local_bin.is_file() {
                return Some(local_bin);
            }
        }
        if let Ok(output) = std::process::Command::new("which").arg("agy").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    let p = std::path::PathBuf::from(stdout);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
        None
    }

    /// Fetches currently available models from the system's `agy` CLI or falls back to `SUPPORTED_MODELS`.
    pub fn fetch_available_models() -> Vec<(String, String)> {
        if let Some(agy_bin) = Self::find_agy_binary() {
            if let Ok(output) = std::process::Command::new(agy_bin).arg("models").output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut list = Vec::new();
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty()
                            || trimmed.starts_with('⠋')
                            || trimmed.starts_with('⠙')
                            || trimmed.starts_with('⠹')
                            || trimmed.starts_with('⠸')
                            || trimmed.starts_with('⠼')
                            || trimmed.starts_with('⠴')
                            || trimmed.starts_with('⠦')
                            || trimmed.starts_with('⠧')
                            || trimmed.starts_with('⠇')
                            || trimmed.starts_with('⠏')
                        {
                            continue;
                        }
                        let mut parts = trimmed.split_whitespace();
                        if let Some(model_name) = parts.next() {
                            let description = parts.collect::<Vec<_>>().join(" ");
                            list.push((model_name.to_string(), description));
                        }
                    }
                    if !list.is_empty() {
                        return list;
                    }
                }
            }
        }

        Self::SUPPORTED_MODELS
            .iter()
            .map(|(m, d)| (m.to_string(), d.to_string()))
            .collect()
    }

    /// Formats the conversation history and available tools into a prompt for `agy`.
    pub fn build_prompt_with_tools(messages: &[Message], tools: &[ToolDefinition]) -> String {
        let mut prompt = String::new();

        if !tools.is_empty() {
            prompt.push_str("You have access to the following tools:\n");
            if let Ok(tools_json) = serde_json::to_string_pretty(tools) {
                prompt.push_str(&tools_json);
                prompt.push('\n');
            }
            prompt.push_str("\nInstructions for tool calling:\n");
            prompt.push_str("If you need to call one or more tools to answer the user request, respond ONLY with a JSON object in this exact format:\n");
            prompt.push_str("{\n  \"tool_calls\": [\n    {\n      \"id\": \"call_1\",\n      \"type\": \"function\",\n      \"function\": {\n        \"name\": \"<tool_name>\",\n        \"arguments\": \"{\\\"arg_name\\\": \\\"value\\\"}\"\n      }\n    }\n  ]\n}\n");
            prompt.push_str("Do NOT include markdown formatting or extra commentary around the JSON when calling tools.\n");
            prompt.push_str("If no tool call is needed, provide your normal conversational answer directly.\n\n");
        }

        prompt.push_str("Conversation History:\n");
        for msg in messages {
            match msg.role {
                Role::System => {
                    prompt.push_str("System: ");
                    prompt.push_str(msg.content.as_deref().unwrap_or(""));
                    prompt.push('\n');
                }
                Role::User => {
                    prompt.push_str("User: ");
                    prompt.push_str(msg.content.as_deref().unwrap_or(""));
                    prompt.push('\n');
                }
                Role::Assistant => {
                    prompt.push_str("Assistant: ");
                    if let Some(tool_calls) = &msg.tool_calls {
                        let calls_json = serde_json::json!({ "tool_calls": tool_calls });
                        prompt.push_str(&calls_json.to_string());
                    } else if let Some(content) = &msg.content {
                        prompt.push_str(content);
                    }
                    prompt.push('\n');
                }
                Role::Tool => {
                    let id = msg.tool_call_id.as_deref().unwrap_or("call");
                    let content = msg.content.as_deref().unwrap_or("");
                    prompt.push_str(&format!("Tool [{id}]: {content}\n"));
                }
            }
        }
        prompt.push_str("Assistant: ");
        prompt
    }

    /// Extracts tool calls from model output if the response contains a tool call payload.
    pub fn extract_tool_calls_from_text(text: &str) -> Option<Vec<ToolCall>> {
        let trimmed = text.trim();
        let cleaned = if let Some(stripped) = trimmed.strip_prefix("```json") {
            stripped.strip_suffix("```").unwrap_or(stripped).trim()
        } else if let Some(stripped) = trimmed.strip_prefix("```") {
            stripped.strip_suffix("```").unwrap_or(stripped).trim()
        } else {
            trimmed
        };

        #[derive(Deserialize)]
        struct ToolCallsWrapper {
            tool_calls: Vec<ToolCall>,
        }

        if let Ok(wrapper) = serde_json::from_str::<ToolCallsWrapper>(cleaned) {
            if !wrapper.tool_calls.is_empty() {
                return Some(wrapper.tool_calls);
            }
        }

        // Also check if text contains embedded JSON with "tool_calls"
        if let Some(start) = cleaned.find('{') {
            if let Some(end) = cleaned.rfind('}') {
                if start < end {
                    let slice = &cleaned[start..=end];
                    if let Ok(wrapper) = serde_json::from_str::<ToolCallsWrapper>(slice) {
                        if !wrapper.tool_calls.is_empty() {
                            return Some(wrapper.tool_calls);
                        }
                    }
                }
            }
        }

        None
    }

    /// Sends a prompt to the Google AI Pro subscription via the local `agy` CLI binary.
    pub fn complete_agy_cli(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError> {
        let agy_bin = Self::find_agy_binary().ok_or_else(|| {
            ProviderError::HttpError(
                "The `agy` CLI binary was not found. Please ensure `agy` is installed to use your Google AI Pro subscription."
                    .to_string(),
            )
        })?;

        let prompt = Self::build_prompt_with_tools(messages, tools);

        let mut cmd = std::process::Command::new(agy_bin);
        cmd.arg("--model")
            .arg(&self.model)
            .arg("--output-format")
            .arg("json")
            .arg("--print")
            .arg(&prompt);

        let output = cmd
            .output()
            .map_err(|err| ProviderError::HttpError(format!("Failed to execute agy CLI: {err}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProviderError::ApiError {
                status: output.status.code().unwrap_or(1) as u16,
                message: format!("agy command failed: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct AgyCliResponse {
            #[serde(default)]
            status: Option<String>,
            #[serde(default)]
            response: Option<String>,
            #[serde(default)]
            error: Option<String>,
        }

        if let Ok(parsed) = serde_json::from_str::<AgyCliResponse>(&stdout) {
            if let Some(ref err) = parsed.error {
                return Err(ProviderError::ApiError {
                    status: 500,
                    message: err.clone(),
                });
            }
            if let Some(text) = parsed.response {
                if let Some(tool_calls) = Self::extract_tool_calls_from_text(&text) {
                    return Ok(ProviderResponse::ToolCalls(tool_calls));
                }
                return Ok(ProviderResponse::Text(text.trim().to_string()));
            }
        }

        if let Some(tool_calls) = Self::extract_tool_calls_from_text(&stdout) {
            return Ok(ProviderResponse::ToolCalls(tool_calls));
        }

        if stdout.trim().is_empty() {
            Err(ProviderError::EmptyResponse)
        } else {
            Ok(ProviderResponse::Text(stdout.trim().to_string()))
        }
    }

    /// Sends completion request via HTTP endpoint.
    pub fn complete_http(
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

        if let Some(ref csrf_token) = self.csrf_token {
            request = request.set("X-Antigravity-CSRF-Token", csrf_token);
        }

        if let Some(ref source) = self.source_metadata {
            request = request.set("X-Antigravity-Source", source);
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

        OpenAiCompatibleProvider::parse_response_json(&response_body)
    }
}

impl Provider for AntigravityProvider {
    fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError> {
        let is_custom_http = (self.base_url.starts_with("http://")
            || self.base_url.starts_with("https://"))
            && !self.base_url.contains("127.0.0.1:38035")
            && !self.base_url.contains("localhost:38035");

        if is_custom_http {
            self.complete_http(messages, tools)
        } else {
            self.complete_agy_cli(messages, tools)
        }
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

    #[test]
    fn test_openai_response_parsing_multiple_tool_calls() {
        let json_raw = r#"{
            "id": "chatcmpl-multi",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [
                            {
                                "id": "call_1",
                                "type": "function",
                                "function": {
                                    "name": "calc",
                                    "arguments": "{\"a\": 1, \"b\": 2, \"op\": \"add\"}"
                                }
                            },
                            {
                                "id": "call_2",
                                "type": "function",
                                "function": {
                                    "name": "echo",
                                    "arguments": "{\"message\": \"done\"}"
                                }
                            }
                        ]
                    }
                }
            ]
        }"#;

        let response = OpenAiCompatibleProvider::parse_response_json(json_raw).unwrap();
        match response {
            ProviderResponse::ToolCalls(calls) => {
                assert_eq!(calls.len(), 2);
                assert_eq!(calls[0].id, "call_1");
                assert_eq!(calls[0].function.name, "calc");
                assert_eq!(calls[1].id, "call_2");
                assert_eq!(calls[1].function.name, "echo");
            }
            _ => panic!("Expected ToolCalls"),
        }
    }

    #[test]
    fn test_openai_response_parsing_null_content_and_no_tool_calls() {
        let json_raw = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": null
                    }
                }
            ]
        }"#;

        let response = OpenAiCompatibleProvider::parse_response_json(json_raw).unwrap();
        match response {
            ProviderResponse::Text(content) => {
                assert_eq!(content, "");
            }
            _ => panic!("Expected empty Text response"),
        }
    }

    #[test]
    fn test_antigravity_provider_builder_and_defaults() {
        let provider = AntigravityProvider::new("gemini-2.5-pro")
            .with_base_url("http://127.0.0.1:38035/v1")
            .with_api_key("agy_secret_token")
            .with_account("user@example.com")
            .with_csrf_token("csrf_123")
            .with_temperature(0.4)
            .with_timeout(30)
            .with_source_metadata("{\"agent\":\"antigravity\"}");

        assert_eq!(provider.model(), "gemini-2.5-pro");
        assert_eq!(provider.base_url(), "http://127.0.0.1:38035/v1");
        assert_eq!(provider.api_key(), Some("agy_secret_token"));
        assert_eq!(provider.account_email(), Some("user@example.com"));
        assert_eq!(provider.csrf_token(), Some("csrf_123"));
        assert!((provider.temperature() - 0.4).abs() < f32::EPSILON);
        assert_eq!(provider.timeout_secs(), 30);
        assert_eq!(
            provider.source_metadata(),
            Some("{\"agent\":\"antigravity\"}")
        );

        let provider_cleared = provider.with_optional_account(None);
        assert_eq!(provider_cleared.account_email(), None);
    }

    #[test]
    fn test_antigravity_provider_from_env_defaults() {
        let provider = AntigravityProvider::from_env();
        assert_eq!(provider.timeout_secs(), 60);
        assert!(!provider.base_url().is_empty());
        assert!(!provider.model().is_empty());
    }

    #[test]
    fn test_antigravity_model_resolution_and_catalogue() {
        assert_eq!(
            AntigravityProvider::resolve_model_name("1"),
            "gemini-3.1-pro-high"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("pro"),
            "gemini-3.1-pro-high"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("2"),
            "gemini-3.1-pro-low"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("3"),
            "gemini-3.8-flash-high"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("flash"),
            "gemini-3.8-flash-high"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("6"),
            "gemini-3.7-flash-high"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("9"),
            "gemini-3.6-flash-high"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("12"),
            "claude-sonnet-4-6"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("sonnet"),
            "claude-sonnet-4-6"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("13"),
            "claude-opus-4-6-thinking"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("opus"),
            "claude-opus-4-6-thinking"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("14"),
            "gpt-oss-120b-medium"
        );
        assert_eq!(
            AntigravityProvider::resolve_model_name("custom-gemini-preview"),
            "custom-gemini-preview"
        );
        assert_eq!(AntigravityProvider::SUPPORTED_MODELS.len(), 14);

        let available = AntigravityProvider::fetch_available_models();
        assert!(!available.is_empty());
    }

    #[test]
    fn test_local_google_credentials_discovery() {
        // Test that read_local_google_account and read_local_oauth_token don't panic
        // and safely read credentials if files exist in the user's home directory.
        let _account = AntigravityProvider::read_local_google_account();
        let _token = AntigravityProvider::read_local_oauth_token();

        let provider = AntigravityProvider::from_env();
        // Provider is successfully constructed with discovered or default settings
        assert!(!provider.model().is_empty());
    }

    #[test]
    fn test_build_prompt_with_tools() {
        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("Calculate 10 + 20"),
            Message::assistant_with_tool_calls(
                None::<String>,
                vec![ToolCall {
                    id: "call_1".to_string(),
                    r#type: "function".to_string(),
                    function: crate::core::types::FunctionCall {
                        name: "calculator".to_string(),
                        arguments: "{\"expression\": \"10 + 20\"}".to_string(),
                    },
                }],
            ),
            Message::tool("30", "call_1"),
        ];

        let tools = vec![ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "calculator".to_string(),
                description: "Compute math expression".to_string(),
                parameters: json!({"type": "object"}),
            },
        }];

        let prompt = AntigravityProvider::build_prompt_with_tools(&messages, &tools);
        assert!(prompt.contains("You have access to the following tools:"));
        assert!(prompt.contains("calculator"));
        assert!(prompt.contains("System: You are a helpful assistant."));
        assert!(prompt.contains("User: Calculate 10 + 20"));
        assert!(prompt.contains("call_1"));
        assert!(prompt.contains("Tool [call_1]: 30"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_extract_tool_calls_from_text() {
        let raw_json = r#"{
            "tool_calls": [
                {
                    "id": "call_123",
                    "type": "function",
                    "function": {
                        "name": "calculator",
                        "arguments": "{\"expression\": \"42 * 99\"}"
                    }
                }
            ]
        }"#;

        let calls = AntigravityProvider::extract_tool_calls_from_text(raw_json).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_123");
        assert_eq!(calls[0].function.name, "calculator");

        // Code block wrapped
        let fenced_json = format!("```json\n{raw_json}\n```");
        let calls_fenced = AntigravityProvider::extract_tool_calls_from_text(&fenced_json).unwrap();
        assert_eq!(calls_fenced.len(), 1);

        // Plain text returns None
        assert!(AntigravityProvider::extract_tool_calls_from_text("The answer is 42.").is_none());
    }

    #[test]
    fn test_find_agy_binary() {
        // In this environment, agy is installed
        let bin = AntigravityProvider::find_agy_binary();
        assert!(bin.is_some());
    }
}
