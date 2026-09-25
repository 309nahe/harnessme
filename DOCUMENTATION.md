# HarnessMe: Technical Documentation

A lightweight, robust, and extensible Agentic AI Harness in Rust with minimal dependencies, clean trait-based modularity, and strict safety guardrails.

---

## Table of Contents

1. [Introduction & Architectural Goals](#1-introduction--architectural-goals)
2. [High-Level Architecture](#2-high-level-architecture)
3. [STEP 1: Milestone 1 — Minimal Working Agent Harness ("Make It Work First")](#3-step-1-milestone-1--minimal-working-agent-harness-make-it-work-first)
   - [3.1 Philosophy: Make It Work, Make It Perfect Later](#31-philosophy-make-it-work-make-it-perfect-later)
   - [3.2 Part 1: Manifest & Minimal Dependencies (`Cargo.toml`)](#32-part-1-manifest--minimal-dependencies-cargotoml)
   - [3.3 Part 2: Core Domain Models & Message IR (`src/core/types.rs`)](#33-part-2-core-domain-models--message-ir-srccoretypesrs)
   - [3.4 Part 3: Tool Abstraction & Registry (`src/core/tool.rs`)](#34-part-3-tool-abstraction--registry-srccoretoolrs)
   - [3.5 Part 4: Provider Abstraction & HTTP Client (`src/core/provider.rs`)](#35-part-4-provider-abstraction--http-client-srccoreproviderrs)
   - [3.6 Part 5: Agent Execution Loop & Safety Limits (`src/core/agent.rs`)](#36-part-5-agent-execution-loop--safety-limits-srccoreagentrs)
   - [3.7 Part 6: CLI Interactive Demo & REPL (`src/main.rs`)](#37-part-6-cli-interactive-demo--repl-srcmainrs)
   - [3.8 Milestone 1 Acceptance Criteria](#38-milestone-1-acceptance-criteria)
4. [Core Data Types & Message Protocol](#4-core-data-types--message-protocol)
5. [Tool Subsystem](#5-tool-subsystem)
6. [Provider Subsystem](#6-provider-subsystem)
7. [Agent Execution Engine](#7-agent-execution-engine)
8. [Configuration & Environment Reference](#8-configuration--environment-reference)
9. [Extension & Integration Guide](#9-extension--integration-guide)
10. [Error Handling & Edge Cases](#10-error-handling--edge-cases)
11. [Testing & Verification Strategy](#11-testing--verification-strategy)
12. [Future Roadmap](#12-future-roadmap)
13. [Living Changelog & Evolution Ledger](#13-living-changelog--evolution-ledger)

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
├── lib.rs             # Library root exposing core and tools modules
├── core/
│   ├── mod.rs
│   ├── types.rs       # Role, Message, ToolCall, ToolResult, ToolDefinition
│   ├── tool.rs        # Tool trait, ToolRegistry, ToolError
│   ├── provider.rs    # Provider trait, ProviderResponse, OpenAI-compatible client
│   └── agent.rs       # AgentConfig, Agent struct, execution loop & lifecycle hooks
├── tools/             # Built-in reference tools (e.g. calculator, echo)
│   ├── mod.rs
│   ├── calculator.rs
│   └── echo.rs
└── main.rs            # CLI Entrypoint and interactive REPL
```

---

## 3. STEP 1: Milestone 1 — Minimal Working Agent Harness ("Make It Work First")

### 3.1 Philosophy: Make It Work, Make It Perfect Later

The sole objective of Milestone 1 is to build a functional, reliable end-to-end agent harness without premature optimization or unnecessary complexity. 

- **No Killer Features**: Skip streaming tokens, complex vector stores, multi-agent swarms, or sandbox runtimes.
- **Sequential Simplicity**: The interaction between an LLM and tools is naturally sequential (Think $\rightarrow$ Tool Call $\rightarrow$ Tool Result $\rightarrow$ Think). We use synchronous blocking I/O (`ureq`) to avoid runtime executors and pinning complexities.
- **Self-Correcting Robustness**: When a tool fails or the model passes invalid JSON arguments, capture the error and feed it back into the conversation context as a `Role::Tool` message so the LLM can recover gracefully.

---

## 3.2 Part 1: Manifest & Minimal Dependencies (`Cargo.toml`)

#### The Idea
Milestone 1 must compile in under 5 seconds with zero dependency bloat. We avoid large async runtimes (Tokio/Actix) and heavy utility crates. The harness relies exclusively on the Rust standard library plus two essential crates: one for JSON serialization and one for HTTP requests.

#### Technological Specifications

```toml
[package]
name = "harnessme"
version = "0.1.0"
edition = "2021"
authors = ["309nahe"]
description = "A lightweight, minimal agentic AI harness in Rust"

[dependencies]
# Serialization and JSON schema parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Minimal synchronous HTTP client (no tokio dependency required)
ureq = { version = "2.10", features = ["json"] }
```

---

## 3.3 Part 2: Core Domain Models & Message IR (`src/core/types.rs`)

#### The Idea
We need a unified intermediate representation (IR) that models all conversational turns. We match the OpenAI function-calling standard because it is the de-facto protocol implemented by OpenAI, Ollama, vLLM, Groq, Mistral, and LocalAI.

#### Technological Specifications

1. **`Role`**:
   - Enumeration with values: `System`, `User`, `Assistant`, `Tool`.
   - Serialized as lowercase strings (`"system"`, `"user"`, `"assistant"`, `"tool"`).
   
2. **`Message`**:
   - `role: Role`: Sender of the message.
   - `content: Option<String>`: Optional text payload (omitted or null when generating tool calls only).
   - `tool_calls: Option<Vec<ToolCall>>`: Populated when the assistant requests one or more tool executions.
   - `tool_call_id: Option<String>`: Populated only on `Role::Tool` messages to link the result back to the corresponding call.
   - Serde annotations: `#[serde(skip_serializing_if = "Option::is_none")]` to produce clean JSON payloads.

3. **`ToolCall` & `FunctionCall`**:
   - `ToolCall`: `id: String`, `r#type: String` (defaults to `"function"`), `function: FunctionCall`.
   - `FunctionCall`: `name: String`, `arguments: String` (raw unparsed JSON string emitted by LLM).

4. **`ToolDefinition` & `FunctionDefinition`**:
   - Describes available tools to the LLM.
   - `ToolDefinition`: `r#type: String = "function"`, `function: FunctionDefinition`.
   - `FunctionDefinition`: `name: String`, `description: String`, `parameters: serde_json::Value` (standard JSON Schema object).

5. **`ToolResult`**:
   - Internal helper representing the outcome of a tool run: `tool_call_id: String`, `content: String`, `is_error: bool`.

---

## 3.4 Part 3: Tool Abstraction & Registry (`src/core/tool.rs`)

#### The Idea
The agent engine must not be tightly coupled to any specific tool. Any capability (calculator, file reader, web fetcher) must implement a common trait. A registry manages registration, provides schema definitions to the provider, and handles dynamic dispatch.

#### Technological Specifications

1. **`ToolError` Enum**:
   ```rust
   #[derive(Debug)]
   pub enum ToolError {
       InvalidArguments(String),
       ExecutionFailed(String),
       ToolNotFound(String),
   }
   ```

2. **`Tool` Trait**:
   - `fn name(&self) -> &str`: Unique name matching LLM tool call.
   - `fn description(&self) -> &str`: Concise natural language explanation of functionality.
   - `fn parameters_schema(&self) -> serde_json::Value`: Valid JSON Schema specifying required and optional arguments.
   - `fn execute(&self, args: serde_json::Value) -> Result<String, ToolError>`: Executes the business logic and returns a text output.

3. **`ToolRegistry`**:
   - Internal storage: `HashMap<String, Box<dyn Tool>>`.
   - `pub fn new() -> Self`: Initializes an empty registry.
   - `pub fn register<T: Tool + 'static>(&mut self, tool: T)`: Inserts a tool into the map.
   - `pub fn definitions(&self) -> Vec<ToolDefinition>`: Converts registered tools into standard schemas for LLM prompts.
   - `pub fn execute(&self, name: &str, raw_args: &str) -> Result<String, ToolError>`:
     - Checks if `name` exists in map.
     - Parses `raw_args` using `serde_json::from_str`.
     - Calls `tool.execute(args)`.

4. **Reference Tools (`src/tools/`)**:
   - `EchoTool`: Accepts `{"message": "string"}` and returns the message.
   - `CalculatorTool`: Accepts `{"expression": "string"}` or `{"a": number, "b": number, "op": "add|sub|mul|div"}` and returns computed results.

---

## 3.5 Part 4: Provider Abstraction & HTTP Client (`src/core/provider.rs`)

#### The Idea
Decouple the agent loop from the LLM network layer. The provider accepts the current history of messages and available tool definitions, makes an HTTP POST request to `/v1/chat/completions`, and returns either final text or requested tool calls.

#### Technological Specifications

1. **`ProviderError` Enum**:
   ```rust
   #[derive(Debug)]
   pub enum ProviderError {
       HttpError(String),
       SerializationError(String),
       ApiError { status: u16, message: String },
       EmptyResponse,
   }
   ```

2. **`ProviderResponse` Enum**:
   ```rust
   #[derive(Debug, Clone)]
   pub enum ProviderResponse {
       Text(String),
       ToolCalls(Vec<ToolCall>),
   }
   ```

3. **`Provider` Trait**:
   ```rust
   pub trait Provider: Send + Sync {
       fn complete(
           &self,
           messages: &[Message],
           tools: &[ToolDefinition],
        ) -> Result<ProviderResponse, ProviderError>;
   }
   ```

4. **`OpenAiCompatibleProvider`**:
   - Fields:
     - `api_key: Option<String>`
     - `base_url: String` (e.g., `https://api.openai.com/v1` or `http://localhost:11434/v1`)
     - `model: String` (e.g., `gpt-4o-mini` or `llama3.1`)
     - `temperature: f32`
   - Request Construction:
     - Endpoint: `{base_url}/chat/completions`
     - Headers: `Content-Type: application/json`, `Authorization: Bearer {api_key}` (if set).
     - Body Payload:
       ```json
       {
         "model": "gpt-4o-mini",
         "messages": [...],
         "tools": [...],
         "temperature": 0.7
       }
       ```
     - Note: If `tools` is empty, omit the `"tools"` field to support non-tool-calling models.
   - Response Handling:
     - Uses `ureq::post(...).send_json(...)`.
     - Inspects `choices[0].message`.
     - If `tool_calls` is present and non-empty $\rightarrow$ return `ProviderResponse::ToolCalls(calls)`.
     - Otherwise $\rightarrow$ return `ProviderResponse::Text(content)`.

---

## 3.6 Part 5: Agent Execution Loop & Safety Limits (`src/core/agent.rs`)

#### The Idea
The agent manages conversational memory and drives the recursive loop: send messages to provider $\rightarrow$ check response $\rightarrow$ execute tools $\rightarrow$ record outputs $\rightarrow$ repeat until the model answers in text or hits the iteration guardrail.

#### Technological Specifications

1. **`AgentConfig`**:
   ```rust
   #[derive(Debug, Clone)]
   pub struct AgentConfig {
       pub system_prompt: Option<String>,
       pub max_iterations: usize, // Default: 10
   }
   ```

2. **`Agent` Struct**:
   ```rust
   pub struct Agent {
       config: AgentConfig,
       provider: Box<dyn Provider>,
       registry: ToolRegistry,
       history: Vec<Message>,
   }
   ```

3. **Execution Loop Algorithm (`Agent::run(&mut self, user_prompt: &str) -> Result<String, AgentError>`)**:
   1. If `history` is empty and `system_prompt` is configured, prepend `Message { role: Role::System, content: system_prompt, .. }`.
   2. Append `Message { role: Role::User, content: Some(user_prompt.to_string()), .. }` to `history`.
   3. Initialize `iteration = 0`.
   4. **Loop**:
      - Check `if iteration >= config.max_iterations` $\rightarrow$ Return `Err(AgentError::MaxIterationsExceeded)`.
      - Increment `iteration += 1`.
      - Call `provider.complete(&self.history, &self.registry.definitions())`.
      - Match `ProviderResponse`:
        - **Case A: `ProviderResponse::Text(text)`**:
          - Append `Message { role: Role::Assistant, content: Some(text.clone()), .. }` to `history`.
          - Return `Ok(text)`.
        - **Case B: `ProviderResponse::ToolCalls(calls)`**:
          - Append `Message { role: Role::Assistant, content: None, tool_calls: Some(calls.clone()), .. }` to `history`.
          - For each `call` in `calls`:
            - Execute tool via `registry.execute(&call.function.name, &call.function.arguments)`.
            - If execution fails (e.g. invalid arguments or runtime error), format error string: `"Error: {err}"`.
            - Append tool response message:
              ```rust
              Message {
                  role: Role::Tool,
                  content: Some(result_text),
                  tool_calls: None,
                  tool_call_id: Some(call.id.clone()),
              }
              ```
          - Continue loop to next iteration.

---

## 3.7 Part 6: CLI Interactive Demo & REPL (`src/main.rs`)

#### The Idea
Provide an immediate, human-usable terminal binary to interact with the agent, test tool calls in real time, and verify model behavior.

#### Technological Specifications

- Reads environment variables:
  - `OPENAI_API_KEY`: API key.
  - `OPENAI_BASE_URL`: Base URL (default: `https://api.openai.com/v1`).
  - `HARNESS_MODEL`: Model name (default: `gpt-4o-mini`).
- Registers default sample tools (`CalculatorTool`, `EchoTool`).
- Enters interactive stdin loop:
  - Prints prompt symbol `user > `.
  - Reads line from `std::io::stdin()`.
  - Exits on `exit`, `quit`, or EOF.
  - Calls `agent.run(&input)`.
  - Prints `agent > {response}`.

---

## 3.8 Milestone 1 Acceptance Criteria

Before declaring Milestone 1 complete, the following criteria must be satisfied:

1. **Clean Compilation**: `cargo check` and `cargo test` pass with 0 warnings and 0 errors.
2. **Zero Bloat Verified**: No asynchronous runtimes or heavy dependencies beyond `serde`, `serde_json`, and `ureq`.
3. **Unit Test Coverage**:
   - `core::types`: Serialization/deserialization of Messages, ToolCalls, and Schemas.
   - `core::tool`: Registration and argument parsing of reference tools.
4. **Mock Provider Test**:
   - A mock provider that returns a tool call on step 1 and a final text answer on step 2 passes through `Agent::run` deterministically.
5. **Interactive REPL Functional**: Running `cargo run` allows conversational interaction and correctly triggers tool calls when asked (e.g., "calculate 45 * 12").

---

## 4. Core Data Types & Message Protocol Deep-Dive

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

## 5. Tool Subsystem

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

## 6. Provider Subsystem

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

## 7. Agent Execution Engine

### Configuration (`AgentConfig`)
```rust
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub system_prompt: Option<String>,
    pub max_iterations: usize, // Default: 10
}

impl AgentConfig {
    pub fn new() -> Self { ... }
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self { ... }
    pub fn with_max_iterations(mut self, max: usize) -> Self { ... }
}
```

### Agent Errors (`AgentError`)
```rust
#[derive(Debug)]
pub enum AgentError {
    Provider(ProviderError),
    MaxIterationsExceeded { max_iterations: usize },
    Tool(ToolError),
}
```

### Agent Lifecycle & Methods (`Agent`)
```rust
pub struct Agent {
    history: Vec<Message>,
    provider: Box<dyn Provider>,
    registry: ToolRegistry,
    config: AgentConfig,
}

impl Agent {
    pub fn new(provider: impl Provider + 'static, registry: ToolRegistry) -> Self;
    pub fn with_config(provider: impl Provider + 'static, registry: ToolRegistry, config: AgentConfig) -> Self;
    pub fn history(&self) -> &[Message];
    pub fn history_mut(&mut self) -> &mut Vec<Message>;
    pub fn clear_history(&mut self);
    pub fn step(&mut self) -> Result<Option<String>, AgentError>;
    pub fn run(&mut self, user_prompt: &str) -> Result<String, AgentError>;
}
```

### Step & Run Execution Lifecycle
1. **Initialize Context**: On `run(user_prompt)`, if conversation `history` is empty and `config.system_prompt` is configured, a `Role::System` message is prepended, followed by the user's prompt as `Role::User`.
2. **Execution Loop & Limits**:
   - The loop runs up to `config.max_iterations` turns.
   - If the loop exceeds `config.max_iterations` without reaching a text response, it returns `Err(AgentError::MaxIterationsExceeded)`.
3. **Step Dispatch**:
   - Calls `provider.complete(&self.history, &self.registry.definitions())`.
   - If `ProviderResponse::Text(content)`: Appends `Role::Assistant` message to history and returns `Ok(Some(content))` (completing execution).
   - If `ProviderResponse::ToolCalls(calls)`:
     - Appends `Message::assistant_tool_calls(calls)` to history.
     - For each `ToolCall`: executes tool via `registry.execute(&call.function.name, &call.function.arguments)`.
     - In case of failure (`ToolError`), formats error as `"Error: {err}"` so the LLM can self-correct without crashing the engine.
     - Appends `Message::tool_result(call.id, result_text)` to history.
     - Returns `Ok(None)` indicating an intermediate tool-turn completed.

### Guardrails
- **Max Iterations Guardrail**: Terminating infinite tool-invocation loops deterministically with `AgentError::MaxIterationsExceeded`.
- **Self-Correcting Tool Error Handling**: Tool execution failures (missing tool, invalid arguments, division by zero) are caught cleanly and fed back as `Role::Tool` messages into the prompt context for model self-correction.


---

## 8. Configuration & Environment Reference

| Environment Variable | Description | Default |
|---|---|---|
| `OPENAI_API_KEY` | API Key for provider authentication | `None` (required for OpenAI) |
| `OPENAI_BASE_URL` | Base endpoint URL | `https://api.openai.com/v1` |
| `HARNESS_MODEL` | Target model name | `gpt-4o-mini` |
| `HARNESS_MAX_STEPS` | Maximum tool execution loop iterations | `10` |
| `HARNESS_TEMPERATURE`| Sampling temperature | `0.7` |

---

## 9. Extension & Integration Guide

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

## 10. Error Handling & Edge Cases

1. **Malformed JSON Arguments**: When an LLM outputs broken JSON in `ToolCall::arguments`, the harness wraps the parse failure into a `ToolError::InvalidArguments` and feeds it back to the model as a `Role::Tool` message.
2. **Unknown Tool Invocations**: If the LLM invents a non-existent tool name, a `ToolError::ToolNotFound` is returned in context.
3. **Provider Network Errors**: HTTP failures and non-2xx status codes are converted to `ProviderError::HttpError` or `ProviderError::ApiError`.
4. **Max Iterations Exceeded**: An `AgentError::MaxIterationsExceeded` error is returned when the guardrail triggers.

---

## 11. Testing & Verification Strategy

- **Unit Tests**:
  - Serialization/deserialization tests for `Role`, `Message`, `ToolCall`, `ToolDefinition`.
  - In-memory `ToolRegistry` lookup and execution.
- **Mock Provider Tests**:
  - Deterministic testing using a `MockProvider` returning pre-arranged tool calls and text responses to verify loop behavior without external API requests.
- **Integration Tests**:
  - Live tests against OpenAI or local Ollama endpoints.

---

## 12. Future Roadmap

- **Token & Context Window Pruning**: Sliding window algorithms to discard older conversation turns while retaining system directives.
- **Asynchronous & Streaming Pipeline**: SSE (Server-Sent Events) streaming for token-by-token output and tool call chunk reassembly.
- **State Persistence**: Serialization of `Agent` context to SQLite or disk files.
- **Sandboxed Execution**: Subprocess/Wasm isolation for dangerous tools.

---

## 13. Living Changelog & Evolution Ledger

> **Mandatory Agent Instruction**: Every autonomous agent or contributor interacting with this codebase must append an entry below whenever implementing a feature, refactoring, fixing a bug, or completing a phase from `PLAN.md`.

### Changelog Format Template
```markdown
### [YYYY-MM-DD] - <Agent / Model / Author Name>
- **Objective**: Brief statement of task.
- **Changes Made**:
  - Itemized list of files created, modified, or deleted.
- **Architectural Decisions**:
  - Key rationale and trade-offs.
- **Verification**:
  - Tests executed and outcomes.
- **Next Steps**:
  - Follow-up actions for the next agent/developer.
```

---

### [2026-09-25] - System Initialization & Comprehensive Documentation
- **Objective**: Establish the repository foundation, generate `DOCUMENTATION.md`, create `AGENTS.md` as an agent system manual, and prepare GitHub repository `harnessme`.
- **Changes Made**:
  - Analyzed [PLAN.md](file:///home/nana/dev/harness/PLAN.md) specifications for the minimal Rust agent harness.
  - Created [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md): Complete technical reference detailing system architecture, core domain models, tool subsystem, provider interface, agent execution loop, configuration, and testing strategy.
  - Created [AGENTS.md](file:///home/nana/dev/harness/AGENTS.md): Established agent operating directives, documentation protocols, and architecture deep dives.
- **Architectural Decisions**:
  - Standardized on zero-bloat dependencies (`serde`, `serde_json`, `ureq`).
- **Verification**:
  - Verified Markdown schemas, file structures, and alignment with [PLAN.md](file:///home/nana/dev/harness/PLAN.md).
- **Next Steps**:
  - Add standard project `.gitignore`.
  - Establish `dev` working branch workflow.
  - Begin Phase 1 implementation (`Cargo.toml` and `src/core/types.rs`).

---

### [2026-09-25] - Project Hygiene & Branching Strategy Setup
- **Objective**: Add comprehensive `.gitignore` configuration and establish `dev` working branch workflow.
- **Changes Made**:
  - Created `.gitignore` ignoring Rust compilation targets (`/target/`), environment files (`.env*`), secrets, IDE configurations, OS artifacts, and scratch directories.
  - Pushed updates and created the `dev` branch on remote and local workspace.
  - Established branching model: `main` reserved strictly for stable releases; `dev` used for active feature development and experimentation.
- **Architectural Decisions**:
  - Prevent accidental leakage of API keys (`OPENAI_API_KEY`) and secret tokens by strictly ignoring environment credential files.
  - Adopt git flow where active development happens on `dev`.
- **Verification**:
  - Verified untracked files are correctly ignored by git.
  - Verified `dev` branch creation and remote tracking.
- **Next Steps**:
  - Write exhaustive STEP 1 Milestone 1 specifications into `DOCUMENTATION.md`.
  - Begin Phase 1 implementation (`Cargo.toml` and `src/core/types.rs`) on `dev` branch.

---

### [2026-09-25] - STEP 1 Milestone 1 Technical Specification
- **Objective**: Author exhaustive STEP 1 specifications in `DOCUMENTATION.md` detailing all components, ideas, and technological requirements for the minimal working agent harness.
- **Changes Made**:
  - Updated [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md) Section 3: "STEP 1: Milestone 1 — Minimal Working Agent Harness ("Make It Work First")".
  - Detailed philosophy (sequential simplicity, zero killer features, make it work first).
  - Detailed Part 1 (`Cargo.toml` minimal manifest), Part 2 (`src/core/types.rs` IR), Part 3 (`src/core/tool.rs` registry & trait), Part 4 (`src/core/provider.rs` synchronous HTTP OpenAI client), Part 5 (`src/core/agent.rs` execution loop and safety limits), Part 6 (`src/main.rs` terminal REPL), and Part 7 (acceptance criteria).
- **Architectural Decisions**:
  - Concrete definition of domain types, traits, error enums, and request schemas before code writing begins.
  - Retained strict zero-bloat philosophy: standard library + `serde` + `serde_json` + `ureq`.
- **Verification**:
  - Markdown layout and schema consistency verified across `PLAN.md`, `DOCUMENTATION.md`, and `AGENTS.md`.
- **Next Steps**:
  - Implement Phase 1: `Cargo.toml`, `src/lib.rs`, and `src/core/types.rs`.

---

### [2026-09-25] - Implementation of Phase 1 / Issue #1: Core Domain Types & Manifest
- **Objective**: Implement Issue #1 (`Cargo.toml` minimal manifest, `src/lib.rs`, `src/core/mod.rs`, and foundational message IR in `src/core/types.rs`).
- **Changes Made**:
  - Created [`Cargo.toml`](file:///home/nana/dev/harness/Cargo.toml) with zero-bloat dependencies: `serde` (with derive), `serde_json`, and `ureq` (with json feature).
  - Created [`src/lib.rs`](file:///home/nana/dev/harness/src/lib.rs) and [`src/core/mod.rs`](file:///home/nana/dev/harness/src/core/mod.rs).
  - Implemented [`src/core/types.rs`](file:///home/nana/dev/harness/src/core/types.rs) containing:
    - `Role`: `System`, `User`, `Assistant`, `Tool` (lowercase serde serialization).
    - `Message`: conversation turn envelope with optional content, tool_calls, and tool_call_id.
    - `ToolCall`, `FunctionCall`, `ToolDefinition`, `FunctionDefinition`, `ToolResult`.
    - Ergonomic constructor helper functions (`Message::system`, `Message::user`, `Message::assistant`, `Message::tool`, `ToolCall::new`, `ToolResult::success`, `ToolResult::error`).
    - 5 comprehensive unit tests covering roundtrip serialization, OpenAI JSON format compatibility, and constructor semantics.
  - Checked off Phase 1 in [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md).
- **Architectural Decisions**:
  - Kept domain structures strictly pure with `#[serde(skip_serializing_if = "Option::is_none")]` to produce clean standard OpenAI payloads without unwanted null keys.
  - Implemented `Send + Sync + Clone` across all domain types to allow effortless passing across threads and future async adapters.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 5 tests with 5/5 passing (0 failures).
- **Next Steps**:
  - Implement Issue #2 / Phase 2 (`src/core/tool.rs` and reference tools in `src/tools/`).

---

### [2026-09-25] - Implementation of Phase 2 / Issue #2: Tool Trait, Registry & Reference Tools
- **Objective**: Implement Issue #2 (`Tool` trait, `ToolError` domain enum, in-memory `ToolRegistry`, and reference tools `EchoTool` and `CalculatorTool`).
- **Changes Made**:
  - Created [`src/core/tool.rs`](file:///home/nana/dev/harness/src/core/tool.rs):
    - `ToolError`: strongly typed errors for invalid arguments, execution failure, and tool-not-found, implementing `Display` and `std::error::Error`.
    - `Tool` trait: `name()`, `description()`, `parameters_schema()`, and `execute()` with `Send + Sync`.
    - `ToolRegistry`: in-memory hash map management with deterministic schema generation (`definitions()`) and dynamic argument JSON parsing and tool dispatch (`execute()`).
  - Created [`src/tools/echo.rs`](file:///home/nana/dev/harness/src/tools/echo.rs): reference tool that validates and echoes back a message string.
  - Created [`src/tools/calculator.rs`](file:///home/nana/dev/harness/src/tools/calculator.rs): arithmetic evaluator supporting basic operations (add, subtract, multiply, divide) with division-by-zero protection.
  - Created [`src/tools/mod.rs`](file:///home/nana/dev/harness/src/tools/mod.rs) and updated [`src/core/mod.rs`](file:///home/nana/dev/harness/src/core/mod.rs) and [`src/lib.rs`](file:///home/nana/dev/harness/src/lib.rs) re-exports.
  - Checked off Phase 2 in [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md).
- **Architectural Decisions**:
  - Isolated all tool failures in `ToolError` results without throwing panics or unwraps.
  - Automated JSON deserialization in `ToolRegistry::execute` so callers don't have to duplicate JSON parsing boilerplate.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 10 tests with 10/10 passing (0 failures).
- **Next Steps**:
  - Implement Issue #3 / Phase 3 (`src/core/provider.rs` OpenAI-compatible client).

---

### [2026-09-25] - AGENTS.md Operational Update: Exclusive MCP GitHub Operations
- **Objective**: Update `AGENTS.md` to establish strict directives requiring all GitHub repository operations (push, commit, issues, comments) to be performed exclusively via MCP tools (`github-mcp-server`) with zero CLI `git`/`gh` usage.
- **Changes Made**:
  - Updated [`AGENTS.md`](file:///home/nana/dev/harness/AGENTS.md):
    - Added Rule 5: "Exclusively Use MCP for GitHub Operations (Zero CLI `git`/`gh`)".
    - Added Section 7: "GitHub Operations via MCP Protocol (Mandatory)" including a full MCP tool mapping reference table.
    - Clarified that local terminal commands are strictly restricted to local Rust compiler/linter/test commands.
    - Updated Phase 2 status to Completed in roadmap table.
- **Architectural Decisions**:
  - Enforce clear boundary between local execution (Rust toolchain) and remote integration (GitHub MCP tools).
- **Verification**:
  - Verified Markdown links, formatting, and structural integrity.
- **Next Steps**:
  - Implement Issue #3 / Phase 3 (`src/core/provider.rs` OpenAI-compatible client).

---

### [2026-09-25] - Implementation of Phase 3 / Issue #3: Provider Trait & OpenAI-Compatible Client
- **Objective**: Implement Issue #3 (`Provider` trait, `ProviderResponse` enum, `ProviderError` domain enum, and `OpenAiCompatibleProvider` synchronous HTTP client using `ureq`).
- **Changes Made**:
  - Created [`src/core/provider.rs`](file:///home/nana/dev/harness/src/core/provider.rs):
    - `ProviderError`: strongly typed errors (`HttpError`, `SerializationError`, `ApiError`, `EmptyResponse`) implementing `Display` and `std::error::Error`.
    - `ProviderResponse`: enum representing either text responses (`ProviderResponse::Text`) or tool invocation requests (`ProviderResponse::ToolCalls`).
    - `Provider` trait: asynchronous-ready `Send + Sync` trait with `complete(&self, messages: &[Message], tools: &[ToolDefinition]) -> Result<ProviderResponse, ProviderError>`.
    - `OpenAiCompatibleProvider`: flexible synchronous HTTP client supporting custom base URLs (OpenAI, Ollama, vLLM, LocalAI, Groq), configurable timeouts, temperature, and optional API key bearer authentication.
    - Added helper `parse_response_json` to parse OpenAI-standard `/chat/completions` JSON responses.
    - Added 7 unit tests covering error display, request payload formatting (conditional tool schema omission), text response parsing, tool calls parsing, empty choices error handling, builder patterns, and custom mock provider implementations.
  - Updated [`src/core/types.rs`](file:///home/nana/dev/harness/src/core/types.rs) with `FunctionCall::new` constructor.
  - Updated [`src/core/mod.rs`](file:///home/nana/dev/harness/src/core/mod.rs) and [`src/lib.rs`](file:///home/nana/dev/harness/src/lib.rs) re-exports.
  - Checked off Phase 3 in [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) and [`AGENTS.md`](file:///home/nana/dev/harness/AGENTS.md).
- **Architectural Decisions**:
  - Maintained zero dependency bloat: relying strictly on `ureq` + `serde`/`serde_json` + Rust `std`.
  - Kept network failure paths non-panicking, mapping HTTP status error bodies cleanly into `ProviderError::ApiError`.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 17 tests with 17/17 passing (0 failures).
- **Next Steps**:
  - Implement Issue #4 / Phase 4 (`src/core/agent.rs` execution loop and safety limits).

---

### [2026-09-25] - Implementation of Phase 4 / Issue #4: Agent Execution Loop & Safety Limits
- **Objective**: Implement Issue #4 (`Agent` struct, `AgentConfig` with configurable system prompt and max iterations, `AgentError` domain enum, recursive execution loop with tool dispatch, and runtime error feedback).
- **Changes Made**:
  - Created [`src/core/agent.rs`](file:///home/nana/dev/harness/src/core/agent.rs):
    - `AgentConfig`: controls `system_prompt` and `max_iterations` (default: 10) with builder methods `with_system_prompt()` and `with_max_iterations()`.
    - `AgentError`: strongly typed error enum (`Provider(ProviderError)`, `MaxIterationsExceeded { max_iterations }`, `Tool(ToolError)`) implementing `Display`, `std::error::Error`, and `From` conversions.
    - `Agent` struct: manages `history: Vec<Message>`, `provider: Box<dyn Provider>`, `registry: ToolRegistry`, and `config: AgentConfig`.
    - `Agent::step(&mut self) -> Result<Option<String>, AgentError>`: single turn execution that invokes provider, pushes assistant turns to history, parses and executes tool calls, records formatted error messages `"Error: {err}"` on tool failure without crashing, and returns `Some(text)` when finished or `None` on tool turns.
    - `Agent::run(&mut self, user_prompt: &str) -> Result<String, AgentError>`: initializes context (system prompt + user message), runs the execution loop, and enforces `max_iterations` guardrail.
    - Added 6 unit and integration mock tests covering default configuration, builder pattern, error display traits, direct text responses, multi-turn tool calling loops, tool error recovery/self-correction feedback, and max iterations exceeded termination.
  - Updated [`src/core/mod.rs`](file:///home/nana/dev/harness/src/core/mod.rs) and [`src/lib.rs`](file:///home/nana/dev/harness/src/lib.rs) re-exports.
  - Updated Section 7 and checked off Phase 4 in [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) and [`AGENTS.md`](file:///home/nana/dev/harness/AGENTS.md).
- **Architectural Decisions**:
  - Zero dependency bloat: standard library and existing crate minimal footprint (`serde`, `serde_json`, `ureq`).
  - Strict safety: no panics or unwraps in production code paths; all tool execution errors are captured and returned to the LLM as tool result messages so the model can inspect error output and self-correct.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 23 tests with 23/23 passing (0 failures).
- **Next Steps**:
  - Implement Issue #5 / Phase 5 (`src/main.rs` CLI interactive REPL & environment loading).

---

### [2026-09-25] - Implementation of Phase 5 / Issue #5: CLI Interactive REPL & Environment Configuration
- **Objective**: Implement Issue #5 (terminal CLI binary entrypoint `src/main.rs` with environment variable loading, tool initialization, error recovery, and interactive REPL loop).
- **Changes Made**:
  - Created [`src/main.rs`](file:///home/nana/dev/harness/src/main.rs):
    - Configured dynamic environment parsing: `OPENAI_API_KEY` (optional for local models), `OPENAI_BASE_URL` (default: `https://api.openai.com/v1`), `HARNESS_MODEL` (default: `gpt-4o-mini`), `HARNESS_SYSTEM_PROMPT`, `HARNESS_MAX_STEPS` (default: 10), and `HARNESS_TEMPERATURE` (default: 0.7).
    - Initialized `ToolRegistry` with built-in reference tools `EchoTool` and `CalculatorTool`.
    - Initialized `OpenAiCompatibleProvider` and configured `Agent` with `AgentConfig`.
    - Added interactive terminal REPL loop over standard input (`std::io::stdin()`) with prompt `user > ` and output `agent > {response}`.
    - Added special terminal commands: `exit`/`quit` (terminate loop), `clear`/`reset` (reset conversation history), `history` (inspect full multi-turn conversation memory with tool call identifiers).
    - Gracefully handled runtime errors and connection failures (`error > {err}`) without terminating the REPL loop.
  - Enhanced [`src/core/provider.rs`](file:///home/nana/dev/harness/src/core/provider.rs) with `with_api_key` and `with_optional_api_key` builder methods.
  - Updated [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) and [`AGENTS.md`](file:///home/nana/dev/harness/AGENTS.md) roadmap tables marking Phase 5 as completed.
- **Architectural Decisions**:
  - Maintained zero dependency bloat: standard library `std::env` and `std::io` for CLI parsing and terminal I/O without introducing heavy CLI frameworks (e.g., `clap`).
  - Guaranteed robust interactive UX with non-fatal error recovery on network drops or invalid tool inputs.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 23 library tests + 0 binary unit tests with 23/23 passing (0 failures).
- **Next Steps**:
  - Milestone 1 / STEP 1 is now fully complete and verified. Ready for tagging/release or next milestone planning.
