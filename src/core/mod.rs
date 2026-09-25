//! Core abstractions and domain data models for the HarnessMe agent runtime.

pub mod agent;
pub mod provider;
pub mod tool;
pub mod types;

pub use agent::{Agent, AgentConfig, AgentError};
pub use provider::{
    AntigravityProvider, OpenAiCompatibleProvider, Provider, ProviderError, ProviderResponse,
};
pub use tool::{Tool, ToolError, ToolRegistry};
pub use types::{
    FunctionCall, FunctionDefinition, Message, Role, ToolCall, ToolDefinition, ToolResult,
};
