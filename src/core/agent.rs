//! Agent execution engine, conversation memory management, and safety guardrails.

use std::error::Error;
use std::fmt;

use crate::core::provider::{Provider, ProviderError, ProviderResponse};
use crate::core::tool::{ToolError, ToolRegistry};
use crate::core::types::Message;

/// Configuration options for the `Agent`.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Optional system instruction prepended to conversation history.
    pub system_prompt: Option<String>,
    /// Maximum number of tool-execution loop iterations before terminating.
    pub max_iterations: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: None,
            max_iterations: 10,
        }
    }
}

impl AgentConfig {
    /// Creates a new `AgentConfig` with default settings (10 max iterations, no system prompt).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Sets the maximum execution loop iterations guardrail.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }
}

/// Strongly typed errors produced by the agent execution engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentError {
    /// Failure during LLM provider communication.
    Provider(ProviderError),
    /// Execution loop exceeded the configured `max_iterations` limit.
    MaxIterationsExceeded { max_iterations: usize },
    /// Failure during tool dispatch or execution.
    Tool(ToolError),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(err) => write!(f, "Provider error: {err}"),
            Self::MaxIterationsExceeded { max_iterations } => {
                write!(
                    f,
                    "Agent exceeded maximum allowed iterations ({max_iterations})"
                )
            }
            Self::Tool(err) => write!(f, "Tool error: {err}"),
        }
    }
}

impl Error for AgentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Provider(err) => Some(err),
            Self::Tool(err) => Some(err),
            Self::MaxIterationsExceeded { .. } => None,
        }
    }
}

impl From<ProviderError> for AgentError {
    fn from(err: ProviderError) -> Self {
        Self::Provider(err)
    }
}

impl From<ToolError> for AgentError {
    fn from(err: ToolError) -> Self {
        Self::Tool(err)
    }
}

/// Autonomous agent orchestrating conversation history, LLM reasoning, and tool dispatch.
pub struct Agent {
    config: AgentConfig,
    provider: Box<dyn Provider>,
    registry: ToolRegistry,
    history: Vec<Message>,
}

impl Agent {
    /// Creates a new `Agent` with default configuration.
    pub fn new(provider: impl Provider + 'static, registry: ToolRegistry) -> Self {
        Self::with_config(provider, registry, AgentConfig::default())
    }

    /// Creates a new `Agent` with custom configuration.
    pub fn with_config(
        provider: impl Provider + 'static,
        registry: ToolRegistry,
        config: AgentConfig,
    ) -> Self {
        Self {
            config,
            provider: Box::new(provider),
            registry,
            history: Vec::new(),
        }
    }

    /// Returns a reference to the agent configuration.
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Returns a mutable reference to the agent configuration.
    pub fn config_mut(&mut self) -> &mut AgentConfig {
        &mut self.config
    }

    /// Returns an immutable slice of the conversation history.
    pub fn history(&self) -> &[Message] {
        &self.history
    }

    /// Returns a mutable reference to the conversation history.
    pub fn history_mut(&mut self) -> &mut Vec<Message> {
        &mut self.history
    }

    /// Returns a reference to the tool registry.
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Returns a mutable reference to the tool registry.
    pub fn registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }

    /// Appends a message directly to conversation history.
    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
    }

    /// Clears the conversation history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Executes a single LLM turn.
    ///
    /// - If the model returns plain text, appends an assistant message and returns `Ok(Some(text))`.
    /// - If the model requests tool calls, appends the assistant tool-call message, executes each tool,
    ///   appends matching `Role::Tool` messages (capturing any runtime errors as feedback), and returns `Ok(None)`.
    pub fn step(&mut self) -> Result<Option<String>, AgentError> {
        let tool_definitions = self.registry.definitions();
        let response = self.provider.complete(&self.history, &tool_definitions)?;

        match response {
            ProviderResponse::Text(text) => {
                self.history.push(Message::assistant(&text));
                Ok(Some(text))
            }
            ProviderResponse::ToolCalls(calls) => {
                self.history
                    .push(Message::assistant_tool_calls(calls.clone()));

                for call in calls {
                    let tool_result_text = match self
                        .registry
                        .execute(&call.function.name, &call.function.arguments)
                    {
                        Ok(output) => output,
                        Err(err) => format!("Error: {err}"),
                    };

                    self.history.push(Message::tool(tool_result_text, call.id));
                }

                Ok(None)
            }
        }
    }

    /// Runs the complete agent loop starting from a human user prompt.
    ///
    /// Automatically prepends `system_prompt` if history is empty, appends the user prompt,
    /// and loops `step()` until a final text answer is produced or `max_iterations` is reached.
    pub fn run(&mut self, user_prompt: &str) -> Result<String, AgentError> {
        if self.history.is_empty() {
            if let Some(ref prompt) = self.config.system_prompt {
                self.history.push(Message::system(prompt));
            }
        }

        self.history.push(Message::user(user_prompt));

        let mut iteration = 0;
        loop {
            if iteration >= self.config.max_iterations {
                return Err(AgentError::MaxIterationsExceeded {
                    max_iterations: self.config.max_iterations,
                });
            }

            iteration += 1;

            if let Some(final_response) = self.step()? {
                return Ok(final_response);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Role, ToolCall, ToolDefinition};
    use crate::tools::{CalculatorTool, EchoTool};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_agent_config_default_and_builder() {
        let default_config = AgentConfig::default();
        assert_eq!(default_config.max_iterations, 10);
        assert!(default_config.system_prompt.is_none());

        let custom_config = AgentConfig::new()
            .with_system_prompt("Be concise")
            .with_max_iterations(5);
        assert_eq!(custom_config.max_iterations, 5);
        assert_eq!(custom_config.system_prompt.as_deref(), Some("Be concise"));
    }

    #[test]
    fn test_agent_error_display_and_traits() {
        let err_max = AgentError::MaxIterationsExceeded { max_iterations: 10 };
        assert_eq!(
            err_max.to_string(),
            "Agent exceeded maximum allowed iterations (10)"
        );

        let err_prov: AgentError = ProviderError::EmptyResponse.into();
        assert_eq!(
            err_prov.to_string(),
            "Provider error: Provider returned an empty response with no choices"
        );

        let err_tool: AgentError = ToolError::ToolNotFound("foo".to_string()).into();
        assert_eq!(err_tool.to_string(), "Tool error: Tool not found: 'foo'");
    }

    struct MockScriptedProvider {
        responses: Vec<ProviderResponse>,
        call_count: Arc<AtomicUsize>,
    }

    impl MockScriptedProvider {
        fn new(responses: Vec<ProviderResponse>) -> (Self, Arc<AtomicUsize>) {
            let count = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    responses,
                    call_count: Arc::clone(&count),
                },
                count,
            )
        }
    }

    impl Provider for MockScriptedProvider {
        fn complete(
            &self,
            _messages: &[Message],
            _tools: &[ToolDefinition],
        ) -> Result<ProviderResponse, ProviderError> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            if idx < self.responses.len() {
                Ok(self.responses[idx].clone())
            } else {
                Err(ProviderError::EmptyResponse)
            }
        }
    }

    #[test]
    fn test_agent_run_direct_text_response() {
        let (provider, call_count) =
            MockScriptedProvider::new(vec![ProviderResponse::Text("Hello user!".to_string())]);

        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);

        let mut agent = Agent::new(provider, registry);
        let result = agent.run("Hi there").unwrap();

        assert_eq!(result, "Hello user!");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(agent.history().len(), 2);
        assert_eq!(agent.history()[0].role, Role::User);
        assert_eq!(agent.history()[1].role, Role::Assistant);
        assert_eq!(agent.history()[1].content.as_deref(), Some("Hello user!"));
    }

    #[test]
    fn test_agent_run_with_tool_calling_loop() {
        let (provider, call_count) = MockScriptedProvider::new(vec![
            ProviderResponse::ToolCalls(vec![ToolCall::new(
                "call_1",
                "calculator",
                r#"{"a": 20, "b": 22, "op": "add"}"#,
            )]),
            ProviderResponse::Text("The answer is 42.".to_string()),
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(CalculatorTool);

        let config = AgentConfig::new().with_system_prompt("Math Assistant");
        let mut agent = Agent::with_config(provider, registry, config);

        let result = agent.run("What is 20 + 22?").unwrap();
        assert_eq!(result, "The answer is 42.");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        // History check: System, User, Assistant(tool_calls), Tool(result), Assistant(final text)
        let history = agent.history();
        assert_eq!(history.len(), 5);
        assert_eq!(history[0].role, Role::System);
        assert_eq!(history[1].role, Role::User);
        assert_eq!(history[2].role, Role::Assistant);
        assert!(history[2].tool_calls.is_some());
        assert_eq!(history[3].role, Role::Tool);
        assert_eq!(history[3].content.as_deref(), Some("42"));
        assert_eq!(history[3].tool_call_id.as_deref(), Some("call_1"));
        assert_eq!(history[4].role, Role::Assistant);
        assert_eq!(history[4].content.as_deref(), Some("The answer is 42."));
    }

    #[test]
    fn test_agent_tool_error_feedback_and_recovery() {
        let (provider, call_count) = MockScriptedProvider::new(vec![
            ProviderResponse::ToolCalls(vec![ToolCall::new(
                "call_err_1",
                "calculator",
                r#"{"a": 10, "b": 0, "op": "div"}"#,
            )]),
            ProviderResponse::Text("Cannot divide by zero.".to_string()),
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(CalculatorTool);

        let mut agent = Agent::new(provider, registry);
        let result = agent.run("Divide 10 by 0").unwrap();

        assert_eq!(result, "Cannot divide by zero.");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        let history = agent.history();
        assert_eq!(history.len(), 4);
        assert_eq!(history[2].role, Role::Tool);
        assert!(history[2]
            .content
            .as_ref()
            .unwrap()
            .contains("Division by zero"));
    }

    #[test]
    fn test_agent_max_iterations_exceeded_guardrail() {
        let infinite_tool_calls = (0..15)
            .map(|i| {
                ProviderResponse::ToolCalls(vec![ToolCall::new(
                    format!("call_{i}"),
                    "echo",
                    r#"{"message": "loop"}"#,
                )])
            })
            .collect();

        let (provider, call_count) = MockScriptedProvider::new(infinite_tool_calls);

        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);

        let config = AgentConfig::new().with_max_iterations(3);
        let mut agent = Agent::with_config(provider, registry, config);

        let err = agent.run("Start looping").unwrap_err();
        assert_eq!(err, AgentError::MaxIterationsExceeded { max_iterations: 3 });
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_agent_multiple_tool_calls_in_single_turn() {
        let (provider, call_count) = MockScriptedProvider::new(vec![
            ProviderResponse::ToolCalls(vec![
                ToolCall::new("call_a", "calculator", r#"{"a": 10, "b": 5, "op": "add"}"#),
                ToolCall::new("call_b", "echo", r#"{"message": "Calculated"}"#),
            ]),
            ProviderResponse::Text("10 + 5 is 15. Calculated.".to_string()),
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(CalculatorTool);
        registry.register(EchoTool);

        let mut agent = Agent::new(provider, registry);
        let res = agent.run("Add 10 and 5 and confirm").unwrap();
        assert_eq!(res, "10 + 5 is 15. Calculated.");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        let history = agent.history();
        // User -> Assistant(2 calls) -> Tool(call_a) -> Tool(call_b) -> Assistant(final text)
        assert_eq!(history.len(), 5);
        assert_eq!(history[0].role, Role::User);
        assert_eq!(history[1].role, Role::Assistant);
        assert_eq!(history[1].tool_calls.as_ref().unwrap().len(), 2);
        assert_eq!(history[2].role, Role::Tool);
        assert_eq!(history[2].content.as_deref(), Some("15"));
        assert_eq!(history[2].tool_call_id.as_deref(), Some("call_a"));
        assert_eq!(history[3].role, Role::Tool);
        assert_eq!(history[3].content.as_deref(), Some("Calculated"));
        assert_eq!(history[3].tool_call_id.as_deref(), Some("call_b"));
        assert_eq!(history[4].role, Role::Assistant);
    }

    #[test]
    fn test_agent_multi_turn_history_persistence_and_clear() {
        let (provider, _call_count) = MockScriptedProvider::new(vec![
            ProviderResponse::Text("Nice to meet you, Alice!".to_string()),
            ProviderResponse::Text("Your name is Alice.".to_string()),
        ]);

        let mut agent = Agent::new(provider, ToolRegistry::new());

        let res1 = agent.run("My name is Alice.").unwrap();
        assert_eq!(res1, "Nice to meet you, Alice!");
        assert_eq!(agent.history().len(), 2);

        let res2 = agent.run("What is my name?").unwrap();
        assert_eq!(res2, "Your name is Alice.");
        assert_eq!(agent.history().len(), 4);

        agent.clear_history();
        assert_eq!(agent.history().len(), 0);

        agent.add_message(Message::system("Custom System"));
        assert_eq!(agent.history().len(), 1);
        assert_eq!(agent.history()[0].content.as_deref(), Some("Custom System"));
    }

    #[test]
    fn test_agent_config_and_registry_mutators() {
        let (provider, _) = MockScriptedProvider::new(vec![]);
        let mut agent = Agent::new(provider, ToolRegistry::new());

        agent.config_mut().max_iterations = 42;
        assert_eq!(agent.config().max_iterations, 42);

        agent.registry_mut().register(EchoTool);
        assert!(agent.registry().contains("echo"));
    }
}
