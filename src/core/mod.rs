//! Core abstractions and domain data models for the HarnessMe agent runtime.

pub mod tool;
pub mod types;

pub use tool::{Tool, ToolError, ToolRegistry};
pub use types::{
    FunctionCall, FunctionDefinition, Message, Role, ToolCall, ToolDefinition, ToolResult,
};
