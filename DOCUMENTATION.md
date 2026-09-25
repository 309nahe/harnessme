# HarnessMe: Technical Documentation

A lightweight, robust, and extensible Agentic AI Harness in Rust with minimal dependencies, clean trait-based modularity, and strict safety guardrails.

---

## Table of Contents

1. [Introduction & Architectural Goals](#1-introduction--architectural-goals)
2. [High-Level Architecture](#2-high-level-architecture)
3. [Core Data Types & Message Protocol](#3-core-data-types--message-protocol)
4. [Tool Subsystem](#4-tool-subsystem)
5. [Provider Subsystem](#5-provider-subsystem)
6. [Agent Execution Engine](#6-agent-execution-engine)
7. [Configuration & Environment Reference](#7-configuration--environment-reference)
8. [Extension & Integration Guide](#8-extension--integration-guide)
9. [Error Handling & Edge Cases](#9-error-handling--edge-cases)
10. [Testing & Verification Strategy](#10-testing--verification-strategy)
11. [Future Roadmap](#11-future-roadmap)

---

## 1. Introduction & Architectural Goals

**HarnessMe** is an AI agent harness written in Rust. It provides the core runtime engine that allows Large Language Models (LLMs) to function as autonomous agents: accepting user prompts, managing conversation history, reasoning about available tools, executing tool calls in a secure/controlled environment, and iterating until the task is resolved.

### Core Design Principles

- **Minimal Dependencies**: Fast compilation and small binary footprints. Core runtime relies strictly on standard library primitives, `serde`/`serde_json` for serialization, and a lightweight HTTP client (e.g., `ureq`).
- **Trait-Based Modularity**: Providers (`Provider`) and Tools (`Tool`) are defined as decoupled traits. The harness is completely agnostic to the underlying LLM provider (OpenAI, Anthropic, Ollama, LocalAI, Groq, vLLM).
- **Strict Guardrails & Safety**: Enforces maximum execution iterations (`max_iterations`) to prevent runaway loops and runaway token consumption.
- **Explicit Error Propagation**: Uses strongly typed errors with no unhandled panics or silent failures.
- **Synchronous / Lean Async**: Designed to run cleanly without requiring an heavyweight asynchronous runtime for basic use cases, while remaining extensible for async pipelines.

---

## 2. High-Level Architecture

The harness is centered around the **Agent Loop**, connecting the Conversation State, the Provider (LLM), and the Tool Registry.

```
┌────────────────────────────────────────────────────────────────────────┐
│                              Agent Loop                                │
│                                                                        │
│   ┌──────────────────────┐   Messages + Tools   ┌───────────────────┐  │
│   │ Conversation Context │ ───────────────────> │  Provider Client  │  │
│   │   (Message History)  │ <─────────────────── │ (OpenAI / Local)  │  │
│   └──────────┬───────────┘    ProviderResponse  └─────────┬─────────┘  │
│              ▲                                            │            │
│              │                                            ▼            │
│              │             Tool Execution Result     ┌───────────┐     │
│              └───────────────────────────────────────┤   Tool    │     │
│                                                      │ Registry  │     │
│                                                      └───────────┘     │
└────────────────────────────────────────────────────────────────────────┘
```

### Module Structure

```
src/
├── core/
│   ├── mod.rs
│   ├── types.rs       # Role, Message, ToolCall, ToolResult, ToolDefinition
│   ├── tool.rs        # Tool trait, ToolRegistry, ToolError
│   ├── provider.rs    # Provider trait, ProviderResponse, OpenAI-compatible client
│   └── agent.rs       # AgentConfig, Agent struct, execution loop & lifecycle hooks
├── tools/             # Built-in reference tools (e.g. calculator, echo, clock)
│   ├── mod.rs
│   ├── calculator.rs
│   └── echo.rs
└── main.rs            # CLI Entrypoint and interactive REPL
```

---

## 3. Core Data Types & Message Protocol

Located in `src/core/types.rs`, these types form the universal domain language for conversation turns and tool invocations.

### `Role`
Represents the actor in a conversation turn:
- `Role::System`: Directives and persona definitions for the model.
- `Role::User`: Input provided by the human user.
- `Role::Assistant`: Responses and generated tool calls emitted by the LLM.
- `Role::Tool`: Outputs returned from executed tools matching a prior `ToolCall`.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}
```

### `Message`
A single turn in the conversation history:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}
```

### `ToolCall`
A structured request from the LLM to invoke a specific tool:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // Raw JSON string as produced by the model
}
```

### `ToolDefinition`
The JSON schema representation advertised to the LLM during prompt construction:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub r#type: String, // usually "function"
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

---

## 4. Tool Subsystem

The tool subsystem provides a uniform interface for defining, validating, registering, and executing tools.

### The `Tool` Trait

```rust
pub trait Tool: Send + Sync {
    /// Unique identifier for the tool (e.g. "calculator")
    fn name(&self) -> &str;

    /// Human/LLM-readable description of functionality and instructions
    fn description(&self) -> &str;

    /// JSON Schema describing input parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Executes the tool given parsed or raw JSON arguments
    fn execute(&self, args: serde_json::Value) -> Result<String, ToolError>;
}
```

### `ToolRegistry`
Maintains an in-memory dictionary of registered tools and translates them to provider-ready `ToolDefinition` vectors:

```rust
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self { ... }
    pub fn register<T: Tool + 'static>(&mut self, tool: T) { ... }
    pub fn get(&self, name: &str) -> Option<&dyn Tool> { ... }
    pub fn definitions(&self) -> Vec<ToolDefinition> { ... }
    pub fn execute(&self, name: &str, args: serde_json::Value) -> Result<String, ToolError> { ... }
}
```

---

## 5. Provider Subsystem

The provider layer decouples the harness from specific LLM endpoints.

### The `Provider` Trait

```rust
pub trait Provider: Send + Sync {
    fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError>;
}
```

### `ProviderResponse`
```rust
#[derive(Debug, Clone)]
pub enum ProviderResponse {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}
```

### `OpenAiCompatibleProvider`
Implements the `Provider` trait for standard OpenAI chat completion endpoints (`/v1/chat/completions`). Compatible with:
- OpenAI (GPT-4o, GPT-4, GPT-3.5)
- LocalAI
- Ollama (`/v1`)
- vLLM
- Groq / Mistral / DeepSeek OpenAI-compatible gateways

---

## 6. Agent Execution Engine

### Configuration (`AgentConfig`)
```rust
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub model: String,
    pub temperature: f32,
    pub system_prompt: Option<String>,
    pub max_iterations: usize, // Default: 10
}
```

### Step Lifecycle (`Agent::step`)
1. **Prepare Context**: Assembles message history (prefixed with system prompt if configured) and tool schemas.
2. **LLM Invocation**: Calls `Provider::complete`.
3. **Dispatch**:
   - If `ProviderResponse::Text(content)`: Appends `Role::Assistant` message. Iteration completes with final answer.
   - If `ProviderResponse::ToolCalls(calls)`:
     - Appends assistant message recording tool calls.
     - For each tool call:
       - Parse arguments from JSON string.
       - Execute tool in `ToolRegistry`.
       - Append `Role::Tool` message with matching `tool_call_id`.
     - Repeats step until text response is emitted or `max_iterations` is hit.

### Guardrails
- **Max Iterations Check**: Prevents infinite tool-invocation loops.
- **Tool Error Handling**: If a tool errors out or fails to parse arguments, the error is recorded into the `Role::Tool` message so the LLM can self-correct on the next iteration.

---

## 7. Configuration & Environment Reference

| Environment Variable | Description | Default |
|---|---|---|
| `OPENAI_API_KEY` | API Key for provider authentication | `None` (required for OpenAI) |
| `OPENAI_BASE_URL` | Base endpoint URL | `https://api.openai.com/v1` |
| `HARNESS_MODEL` | Target model name | `gpt-4o-mini` |
| `HARNESS_MAX_STEPS` | Maximum tool execution loop iterations | `10` |
| `HARNESS_TEMPERATURE`| Sampling temperature | `0.7` |

---

## 8. Extension & Integration Guide

### Creating a Custom Tool

```rust
use harnessme::core::tool::{Tool, ToolError};
use serde_json::json;

pub struct TimeTool;

impl Tool for TimeTool {
    fn name(&self) -> &str {
        "get_current_time"
    }

    fn description(&self) -> &str {
        "Returns the current UTC date and time in ISO-8601 format."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _args: serde_json::Value) -> Result<String, ToolError> {
        Ok(chrono::Utc::now().to_rfc3339())
    }
}
```

---

## 9. Error Handling & Edge Cases

1. **Malformed JSON Arguments**: When an LLM outputs broken JSON in `ToolCall::arguments`, the harness wraps the parse failure into a `ToolError::InvalidArguments` and feeds it back to the model as a `Role::Tool` message.
2. **Unknown Tool Invocations**: If the LLM invents a non-existent tool name, a `ToolError::ToolNotFound` is returned in context.
3. **Provider Network Errors**: HTTP failures and non-2xx status codes are converted to `ProviderError::HttpError` or `ProviderError::ApiError`.
4. **Max Iterations Exceeded**: An `AgentError::MaxIterationsExceeded` error is returned when the guardrail triggers.

---

## 10. Testing & Verification Strategy

- **Unit Tests**:
  - Serialization/deserialization tests for `Role`, `Message`, `ToolCall`, `ToolDefinition`.
  - In-memory `ToolRegistry` lookup and execution.
- **Mock Provider Tests**:
  - Deterministic testing using a `MockProvider` returning pre-arranged tool calls and text responses to verify loop behavior without external API requests.
- **Integration Tests**:
  - Live tests against OpenAI or local Ollama endpoints.

---

## 11. Future Roadmap

- **Token & Context Window Pruning**: Sliding window algorithms to discard older conversation turns while retaining system directives.
- **Asynchronous & Streaming Pipeline**: SSE (Server-Sent Events) streaming for token-by-token output and tool call chunk reassembly.
- **State Persistence**: Serialization of `Agent` context to SQLite or disk files.
- **Sandboxed Execution**: Subprocess/Wasm isolation for dangerous tools.
