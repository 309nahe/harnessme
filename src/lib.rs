//! # HarnessMe
//!
//! A lightweight, minimal, and extensible Agentic AI Harness in Rust.

pub mod core;
pub mod tools;

pub use core::{
    FunctionCall, FunctionDefinition, Message, Role, Tool, ToolCall, ToolDefinition, ToolError,
    ToolRegistry, ToolResult,
};
pub use tools::{CalculatorTool, EchoTool};
