# HarnessMe: Technical Documentation

A lightweight, robust, and extensible Agentic AI Harness in Rust with minimal dependencies, clean trait-based modularity, and strict safety guardrails.

---

## Table of Contents

1. [Quickstart & Usage Guide](#1-quickstart--usage-guide)
   - [1.1 Prerequisites & Compilation](#11-prerequisites--compilation)
   - [1.2 Launching the Interactive CLI REPL](#12-launching-the-interactive-cli-repl)
   - [1.3 Environment Variables Reference](#13-environment-variables-reference)
   - [1.4 REPL Commands & Example Workflows](#14-repl-commands--example-workflows)
   - [1.5 Using HarnessMe as a Library (Rust API)](#15-using-harnessme-as-a-library-rust-api)
2. [Introduction & Architectural Goals](#2-introduction--architectural-goals)
3. [High-Level Architecture](#3-high-level-architecture)
4. [STEP 1: Milestone 1 — Minimal Working Agent Harness ("Make It Work First")](#4-step-1-milestone-1--minimal-working-agent-harness-make-it-work-first)
   - [4.1 Philosophy: Make It Work, Make It Perfect Later](#41-philosophy-make-it-work-make-it-perfect-later)
   - [4.2 Part 1: Manifest & Minimal Dependencies (`Cargo.toml`)](#42-part-1-manifest--minimal-dependencies-cargotoml)
   - [4.3 Part 2: Core Domain Models & Message IR (`src/core/types.rs`)](#43-part-2-core-domain-models--message-ir-srccoretypesrs)
   - [4.4 Part 3: Tool Abstraction & Registry (`src/core/tool.rs`)](#44-part-3-tool-abstraction--registry-srccoretoolrs)
   - [4.5 Part 4: Provider Abstraction & HTTP Client (`src/core/provider.rs`)](#45-part-4-provider-abstraction--http-client-srccoreproviderrs)
   - [4.6 Part 5: Agent Execution Loop & Safety Limits (`src/core/agent.rs`)](#46-part-5-agent-execution-loop--safety-limits-srccoreagentrs)
   - [4.7 Part 6: CLI Interactive Demo & REPL (`src/main.rs`)](#47-part-6-cli-interactive-demo--repl-srcmainrs)
   - [4.8 Milestone 1 Acceptance Criteria](#48-milestone-1-acceptance-criteria)
5. [STEP 2: Milestone 2 — Observability, Memory Pruning & Filesystem Capabilities](#5-step-2-milestone-2--observability-memory-pruning--filesystem-capabilities)
   - [5.1 Architectural Scope & Core Drivers](#51-architectural-scope--core-drivers)
   - [5.2 Part 1: Token Usage Tracking & Provider Metadata (`src/core/types.rs` & `src/core/provider.rs`)](#52-part-1-token-usage-tracking--provider-metadata-srccoretypesrs--srccoreproviderrs)
   - [5.3 Part 2: Agent Event Hooks & Lifecycle Observers (`src/core/agent.rs`)](#53-part-2-agent-event-hooks--lifecycle-observers-srccoreagentrs)
   - [5.4 Part 3: Sandboxed Filesystem Tools (`src/tools/fs.rs`)](#54-part-3-sandboxed-filesystem-tools-srctoolsfsrs)
   - [5.5 Part 4: Context Retention & Sliding Window Pruning (`src/core/agent.rs`)](#55-part-4-context-retention--sliding-window-pruning-srccoreagentrs)
   - [5.6 Part 5: CLI REPL Observability & Diagnostic Commands (`src/main.rs`)](#56-part-5-cli-repl-observability--diagnostic-commands-srcmainrs)
   - [5.7 Milestone 2 Acceptance Criteria](#57-milestone-2-acceptance-criteria)
6. [Core Data Types & Message Protocol Deep-Dive](#6-core-data-types--message-protocol-deep-dive)
7. [Tool Subsystem](#7-tool-subsystem)
8. [Provider Subsystem](#8-provider-subsystem)
9. [Agent Execution Engine](#9-agent-execution-engine)
10. [Configuration & Environment Reference](#10-configuration--environment-reference)
11. [Extension & Integration Guide](#11-extension--integration-guide)
12. [Error Handling & Edge Cases](#12-error-handling--edge-cases)
13. [Testing & Verification Strategy](#13-testing--verification-strategy)
14. [Future Roadmap](#14-future-roadmap)
15. [Living Changelog & Evolution Ledger](#15-living-changelog--evolution-ledger)

---

## 1. Quickstart & Usage Guide

This section explains how to compile, configure, launch, and interact with HarnessMe both as a standalone terminal REPL and as an embedded Rust library.

---

### 1.1 Prerequisites & Compilation

HarnessMe is built with the standard Rust toolchain (edition 2021). No additional C libraries or runtime daemons are required.

```bash
# Clone the repository
git clone https://github.com/309nahe/harnessme.git
cd harnessme

# Verify code formatting and lint rules
cargo fmt --check
cargo clippy -- -D warnings

# Execute test suite (100% pass required)
cargo test

# Build release binary
cargo build --release
```

---

### 1.2 Launching the Interactive CLI REPL

The CLI entrypoint (`src/main.rs`) provides an interactive REPL with live tool execution, multi-turn conversation memory, and error self-correction, powered exclusively by **Google AI / Antigravity**:

1. **Automatic Discovery & Launch**:
   Launch HarnessMe directly:
   ```bash
   cargo run
   ```
   HarnessMe will automatically scan your local environment:
   - It discovers your local Antigravity Language Server (`ANTIGRAVITY_LS_ADDRESS`, `ANTIGRAVITY_CSRF_TOKEN`).
   - It reads your active local Google Account from `~/.gemini/google_accounts.json` and OAuth token from `~/.gemini/oauth_creds.json`.
   - If found, it connects immediately and greets you ready to chat.

2. **One-Click Google Sign-In (`/agy` or `/agy login`)**:
   If not yet signed in or you wish to authenticate via browser:
   ```text
   user > /agy
   ================ Google Sign-In (Google AI / Antigravity) ================
   Opening your web browser to authenticate with Google...
   If your browser does not open automatically, visit:
     https://accounts.google.com/

   system > Browser opened successfully.
   system > Detected local Google credentials!
   --------------------------------------------------------------------------
     Status   : SIGNED IN
     Account  : your.email@gmail.com
     Model    : gemini-2.5-flash
     Endpoint : http://127.0.0.1:38035/v1
   system > Google AI (Antigravity) is active and ready to use.
   ==========================================================================
   ```
   HarnessMe will automatically spawn your default system browser (via `xdg-open` on Linux, `open` on macOS, or `cmd /C start` on Windows).

3. **Explicit Antigravity Configuration**:
   ```bash
   export ANTIGRAVITY_BASE_URL="http://127.0.0.1:38035/v1"
   export ANTIGRAVITY_CSRF_TOKEN="session_token_123"
   export ANTIGRAVITY_MODEL="gemini-2.5-flash"
   export ANTIGRAVITY_ACCOUNT="user@example.com"
   cargo run
   ```

---

### 1.3 Environment Variables Reference

| Variable | Default Value | Description |
|---|---|---|
| `ANTIGRAVITY_BASE_URL` | `http://127.0.0.1:38035/v1` | Base HTTP endpoint for Google Antigravity |
| `ANTIGRAVITY_MODEL` | `gemini-2.5-flash` | Default Gemini / AGY model |
| `ANTIGRAVITY_ACCOUNT` | Auto-detected from `~/.gemini/` | Linked Google account email |
| `ANTIGRAVITY_CSRF_TOKEN`| Auto-detected | CSRF validation token forwarded via `X-Antigravity-CSRF-Token` |
| `ANTIGRAVITY_API_KEY` | Auto-detected | Bearer authorization token |
| `ANTIGRAVITY_LS_ADDRESS`| Auto-detected | Local Antigravity Language Server host/port |
| `HARNESS_MAX_STEPS` | `10` | Maximum autonomous tool iterations per user prompt |
| `HARNESS_SYSTEM_PROMPT` | Concise assistant | Base instructions given to the agent |

---

### 1.4 REPL Commands & Example Workflows

The interactive terminal supports the following commands:

- `/agy`: Trigger Google Sign-In, opening your default browser and auto-linking local credentials.
- `/agy login`: Same as `/agy`, signs in with Google via default browser.
- `/agy menu`: Display the interactive Google AI configuration and options menu.
- `/agy 1`: Shortcut to sign in with Google via browser.
- `/agy 2`: Shortcut to list and switch Gemini models.
- `/agy 3`: Shortcut to view detailed status and endpoints.
- `/agy models`: List all supported Gemini and Antigravity models.
- `/agy model <name|num>`: Switch models (e.g. `/agy model 2` or `/agy model gemini-2.5-pro`).
- `/agy link [token]`: Link Google account or Bearer API token.
- `/agy account <email>`: Link Google account email.
- `/agy status`: Show current Google AI / Antigravity settings and active status.
- `/agy help`: Display all Antigravity options.
- `/provider`: Show details on active provider.
- `/history` (or `history`): Print all turns in the active conversation context.
- `/clear` (or `clear` / `/reset`): Reset the conversation memory.
- `/exit` (or `exit` / `/quit`): Terminate the REPL session.

#### Example Terminal Session:
```text
====================================================
      HarnessMe Agent Interactive CLI (Google AI)   
====================================================
 Provider     : Google AI / Antigravity
 Account      : mavepith762@gmail.com
 Model        : gemini-2.5-flash
 Base URL     : http://127.0.0.1:38035/v1
 Auth Key     : Configured (hidden)
 Max Steps    : 10
 Temperature  : 0.7
----------------------------------------------------
 Registered Tools: echo, calculator
 Commands: '/agy' to sign in with Google, '/help' for manual, '/exit' to quit.
====================================================

user > What is 123 * 456?
agent > 123 * 456 = 56088.

user > /history
system > Conversation history (4 turns):
  [0] system: You are a helpful and concise AI assistant equipped with tools...
  [1] user: What is 123 * 456?
  [2] assistant (tool_calls: calculator): [No text content]
  [3] tool (call_id: call_123): 56088
  [4] assistant: 123 * 456 = 56088.

user > /exit
Goodbye!
```

---

### 1.5 Using HarnessMe as a Library (Rust API)

You can embed HarnessMe directly in your Rust applications:

```rust
use harnessme::{
    Agent, AgentConfig, CalculatorTool, EchoTool,
    OpenAiCompatibleProvider, ToolRegistry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize tool registry and register desired tools
    let mut registry = ToolRegistry::new();
    registry.register(EchoTool);
    registry.register(CalculatorTool);

    // 2. Configure provider (OpenAI, Ollama, etc.)
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    let provider = OpenAiCompatibleProvider::new("gpt-4o-mini", api_key)
        .with_base_url("https://api.openai.com/v1")
        .with_temperature(0.2);

    // 3. Configure agent settings and guardrails
    let config = AgentConfig::new()
        .with_system_prompt("You are an autonomous engineering assistant.")
        .with_max_iterations(10);

    // 4. Instantiate and run the agent
    let mut agent = Agent::with_config(provider, registry, config);
    let answer = agent.run("What is 42 * 100?")?;

    println!("Agent Answer: {answer}");
    Ok(())
}
```

---

## 2. Introduction & Architectural Goals

**HarnessMe** is an AI agent harness written in Rust. It provides the core runtime engine that allows Large Language Models (LLMs) to function as autonomous agents: accepting user prompts, managing conversation history, reasoning about available tools, executing tool calls in a secure/controlled environment, and iterating until the task is resolved.

### Core Design Principles

- **Minimal Dependencies**: Fast compilation and small binary footprints. Core runtime relies strictly on standard library primitives, `serde`/`serde_json` for serialization, and a lightweight HTTP client (e.g., `ureq`).
- **Trait-Based Modularity**: Providers (`Provider`) and Tools (`Tool`) are defined as decoupled traits. The harness is completely agnostic to the underlying LLM provider (OpenAI, Anthropic, Ollama, LocalAI, Groq, vLLM).
- **Strict Guardrails & Safety**: Enforces maximum execution iterations (`max_iterations`) to prevent runaway loops and runaway token consumption.
- **Explicit Error Propagation**: Uses strongly typed errors with no unhandled panics or silent failures.
- **Synchronous / Lean Async**: Designed to run cleanly without requiring an heavyweight asynchronous runtime for basic use cases, while remaining extensible for async pipelines.

---

## 3. High-Level Architecture

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

## 4. STEP 1: Milestone 1 — Minimal Working Agent Harness ("Make It Work First")

### 4.1 Philosophy: Make It Work, Make It Perfect Later

The sole objective of Milestone 1 is to build a functional, reliable end-to-end agent harness without premature optimization or unnecessary complexity. 

- **No Killer Features**: Skip streaming tokens, complex vector stores, multi-agent swarms, or sandbox runtimes.
- **Sequential Simplicity**: The interaction between an LLM and tools is naturally sequential (Think $\rightarrow$ Tool Call $\rightarrow$ Tool Result $\rightarrow$ Think). We use synchronous blocking I/O (`ureq`) to avoid runtime executors and pinning complexities.
- **Self-Correcting Robustness**: When a tool fails or the model passes invalid JSON arguments, capture the error and feed it back into the conversation context as a `Role::Tool` message so the LLM can recover gracefully.

---

### 4.2 Part 1: Manifest & Minimal Dependencies (`Cargo.toml`)

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

### 4.3 Part 2: Core Domain Models & Message IR (`src/core/types.rs`)

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

### 4.4 Part 3: Tool Abstraction & Registry (`src/core/tool.rs`)

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

### 4.5 Part 4: Provider Abstraction & HTTP Client (`src/core/provider.rs`)

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

### 4.6 Part 5: Agent Execution Loop & Safety Limits (`src/core/agent.rs`)

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

### 4.7 Part 6: CLI Interactive Demo & REPL (`src/main.rs`)

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

### 4.8 Milestone 1 Acceptance Criteria

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

## 5. STEP 2: Milestone 2 — Observability, Memory Pruning & Filesystem Capabilities

### 5.1 Philosophy & Architectural Objectives

With the foundation of Milestone 1 in place ("make it work"), Milestone 2 elevates the harness into a production-ready autonomous runtime without introducing unnecessary architectural bloat.

- **Atomic 5-Phase Scoping**: Milestone 2 is strictly partitioned into 5 independent, single-focus issues (Token Metrics, Lifecycle Hooks, Filesystem Tools, Context Pruning, REPL Observability).
- **Non-Invasive Observability**: Decouple tracing and event streaming through the `AgentHook` trait (Observer pattern) rather than hardcoding stdout prints into the agent core.
- **Defensive Resource Management**: Track token budgets per step and lifetime, and prevent unbounded context growth via deterministic sliding window retention while strictly pinning initial system directives.
- **Sandboxed File Operations**: Equip the agent with essential read/write capabilities guarded by root-jail path checks to prevent unauthorized path traversal outside the designated workspace.

---

### 5.2 Part 1: Token Usage Tracking & Provider Metadata (`src/core/types.rs`, `src/core/provider.rs`)

#### The Idea
Autonomous agents can rapidly consume tokens during multi-turn loops. The harness needs explicit accounting of prompt, completion, and total tokens per LLM completion, aggregating cumulative session totals on the `Agent`.

#### Technological Specifications

1. **`Usage` Domain Struct (`src/core/types.rs`)**:
   ```rust
   #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
   pub struct Usage {
       pub prompt_tokens: usize,
       pub completion_tokens: usize,
       pub total_tokens: usize,
   }
   ```
2. **`ProviderResponse` Enhancement (`src/core/provider.rs`)**:
   - Extend `ProviderResponse` to include optional completion usage:
     - `ProviderResponse::Text { content: String, usage: Option<Usage> }`
     - `ProviderResponse::ToolCalls { calls: Vec<ToolCall>, usage: Option<Usage> }`
   - Update `OpenAiCompatibleProvider::parse_response_json` to extract `usage.prompt_tokens`, `usage.completion_tokens`, and `usage.total_tokens` when present in the endpoint JSON payload.
3. **Session Usage Aggregation (`src/core/agent.rs`)**:
   - Add `cumulative_usage: Usage` to `Agent`.
   - Expose `agent.cumulative_usage() -> Usage` and `agent.last_turn_usage() -> Option<Usage>`.

---

### 5.3 Part 2: Agent Event Hooks & Lifecycle Observers (`src/core/agent.rs`)

#### The Idea
Callers (such as CLI interfaces, web servers, or evaluation harnesses) need visibility into intermediate agent thought steps and tool invocations without altering the core loop logic.

#### Technological Specifications

1. **`AgentHook` Trait (`src/core/agent.rs`)**:
   ```rust
   pub trait AgentHook: Send + Sync {
       fn on_step_start(&self, step: usize, messages: &[Message]) {}
       fn on_tool_call(&self, step: usize, call: &ToolCall) {}
       fn on_tool_result(&self, step: usize, call: &ToolCall, result: &str, is_error: bool) {}
       fn on_step_complete(&self, step: usize, response: &str, usage: Option<Usage>) {}
       fn on_error(&self, step: usize, error: &AgentError) {}
   }
   ```
2. **Hook Management & Dispatch**:
   - `AgentConfig::with_hook(mut self, hook: Arc<dyn AgentHook>) -> Self`
   - `Agent::add_hook(&mut self, hook: Arc<dyn AgentHook>)`
   - In `Agent::step()` and `Agent::run()`, trigger the respective callback hooks at each lifecycle event safely.

---

### 5.4 Part 3: Standard Sandboxed Filesystem Tools (`src/tools/fs.rs`)

#### The Idea
Agents require standard primitives to inspect and modify project workspaces. File operations must be strictly sandboxed within a configured base directory to prevent arbitrary directory traversal (`../`).

#### Technological Specifications

1. **`ReadFileTool`**:
   - Parameters schema: `{"path": "string"}`.
   - Validates that the canonicalized target path resides within the configured base directory root.
   - Returns file contents as UTF-8 string, or returns `ToolError::ExecutionFailed` on missing file / out-of-jail paths.
2. **`WriteFileTool`**:
   - Parameters schema: `{"path": "string", "content": "string"}`.
   - Validates sandboxed boundary; creates parent directories if needed and writes UTF-8 text.
3. **Module & Re-exports**:
   - Located in `src/tools/fs.rs`, re-exported under `src/tools/mod.rs` and `src/lib.rs`.

---

### 5.5 Part 4: Conversation Context Pruning & History Retention (`src/core/agent.rs`)

#### The Idea
Long-running conversations or loops with extensive tool payloads can exceed model context limits. The agent must support automated history pruning that enforces a maximum message window while strictly preserving the initial `Role::System` directive.

#### Technological Specifications

1. **`ContextPolicy` Configuration**:
   ```rust
   #[derive(Debug, Clone)]
   pub enum ContextPolicy {
       /// Keep all messages without pruning.
       Unbounded,
       /// Keep at most `max_messages` turns, always preserving the initial System message.
       SlidingWindow { max_messages: usize },
   }
   ```
2. **Pruning Algorithm (`Agent::prune_history(&mut self)`)**:
   - When history exceeds `max_messages` under `SlidingWindow`:
     - Keep initial `Role::System` prompt at index 0 (if present).
     - Retain the most recent `(max_messages - 1)` messages.
     - Safely drop older intermediate turns without breaking tool-call / tool-result sequence validity.

---

### 5.6 Part 5: CLI REPL Observability & Diagnostic Commands (`src/main.rs`)

#### The Idea
Surface the new Milestone 2 features directly to human operators in the interactive REPL with live tool execution indicators, token usage tracking, and diagnostic commands.

#### Technological Specifications

1. **Terminal Observer Hook (`CliObserverHook`)**:
   - Implements `AgentHook` to print clear, formatted indicators during execution:
     - `[tool-call] ⚙ Invoking calculator with {"a": 20, "b": 22, "op": "add"}...`
     - `[tool-result] ✓ Result: 42`
2. **Diagnostic REPL Commands**:
   - `/stats` or `/tokens`: Displays lifetime prompt, completion, total token usage, and step counts.
   - `/tools`: Lists all active registered tools and descriptions.
   - `/help`: Displays summary of available interactive terminal commands.
3. **Workspace File Tool Integration**:
   - Configures `ReadFileTool` and `WriteFileTool` targeting the current working directory.

---

### 5.7 Milestone 2 Acceptance Criteria

1. **Zero Bloat Preserved**: Only `serde`, `serde_json`, and `ureq` remain as external runtime dependencies.
2. **100% Test Suite Pass**: All new modules (`Usage`, `AgentHook`, `fs`, `ContextPolicy`) backed by thorough unit tests.
3. **Deterministic Safety**: Filesystem tools cannot read/write outside designated sandboxes; context pruner preserves system instructions.
4. **End-to-End CLI Verification**: REPL outputs live tool logs, executes filesystem tools safely, and reports token counts on `/stats`.

---

## 6. Core Data Types & Message Protocol Deep-Dive

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

## 7. Tool Subsystem

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

## 8. Provider Subsystem

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

### `AntigravityProvider`
First-class provider implementation designed for Google Antigravity (AGY) endpoints, cloud gateways, and local Antigravity Language Servers.

#### Key Features
- **Zero Configuration Discovery**: `AntigravityProvider::from_env()` automatically discovers local language server addresses from `ANTIGRAVITY_LS_ADDRESS` and maps them to `http://{address}/v1`.
- **Security & CSRF Tokens**: Automatically captures and forwards `ANTIGRAVITY_CSRF_TOKEN` in the `X-Antigravity-CSRF-Token` header.
- **Source Metadata**: Forwards `X-Antigravity-Source` for provenance and telemetry tracking.
- **Model Presets**: Pre-configured defaults for `gemini-2.5-flash`, `gemini-2.5-pro`, and `agy-pro`.

#### Rust Usage Example
```rust
use harnessme::core::provider::AntigravityProvider;
use harnessme::{Agent, AgentConfig, ToolRegistry};

// 1. Auto-discover from local environment or environment variables
let provider = AntigravityProvider::from_env();

// 2. Or configure manually with builder methods
let custom_provider = AntigravityProvider::new("gemini-2.5-pro")
    .with_base_url("http://127.0.0.1:38035/v1")
    .with_csrf_token("session_token_123")
    .with_temperature(0.2);
```

---

## 9. Agent Execution Engine

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

## 10. Configuration & Environment Reference

| Environment Variable | Description | Default |
|---|---|---|
| `HARNESS_PROVIDER` | LLM provider backend (`"openai"` or `"antigravity"` / `"agy"`) | Auto-detected |
| `OPENAI_API_KEY` | API Key for OpenAI provider authentication | `None` (required for OpenAI cloud) |
| `OPENAI_BASE_URL` | Base endpoint URL for OpenAI-compatible completions | `https://api.openai.com/v1` |
| `ANTIGRAVITY_BASE_URL` | Base endpoint URL for Antigravity (AGY) completions | `http://127.0.0.1:38035/v1` |
| `ANTIGRAVITY_API_KEY` / `AGY_API_KEY` | Optional bearer token for Antigravity gateway | `None` |
| `ANTIGRAVITY_LS_ADDRESS` | Address of local Antigravity Language Server | `None` |
| `ANTIGRAVITY_CSRF_TOKEN` | CSRF token for secure Antigravity Language Server RPC | `None` |
| `ANTIGRAVITY_MODEL` | Target model for Antigravity provider | `gemini-2.5-flash` |
| `HARNESS_MODEL` | Universal fallback model identifier | `gpt-4o-mini` / `gemini-2.5-flash` |
| `HARNESS_MAX_STEPS` | Maximum tool execution loop iterations | `10` |
| `HARNESS_TEMPERATURE`| Sampling temperature | `0.7` |

---

## 11. Extension & Integration Guide

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

## 12. Error Handling & Edge Cases

1. **Malformed JSON Arguments**: When an LLM outputs broken JSON in `ToolCall::arguments`, the harness wraps the parse failure into a `ToolError::InvalidArguments` and feeds it back to the model as a `Role::Tool` message.
2. **Unknown Tool Invocations**: If the LLM invents a non-existent tool name, a `ToolError::ToolNotFound` is returned in context.
3. **Provider Network Errors**: HTTP failures and non-2xx status codes are converted to `ProviderError::HttpError` or `ProviderError::ApiError`.
4. **Max Iterations Exceeded**: An `AgentError::MaxIterationsExceeded` error is returned when the guardrail triggers.

---

## 13. Testing & Verification Strategy

- **Unit Tests**:
  - Serialization/deserialization tests for `Role`, `Message`, `ToolCall`, `ToolDefinition`.
  - In-memory `ToolRegistry` lookup and execution.
- **Mock Provider Tests**:
  - Deterministic testing using a `MockProvider` returning pre-arranged tool calls and text responses to verify loop behavior without external API requests.
- **Integration Tests**:
  - Live tests against OpenAI or local Ollama endpoints.

---

## 14. Future Roadmap

- **Token & Context Window Pruning**: Sliding window algorithms to discard older conversation turns while retaining system directives.
- **Asynchronous & Streaming Pipeline**: SSE (Server-Sent Events) streaming for token-by-token output and tool call chunk reassembly.
- **State Persistence**: Serialization of `Agent` context to SQLite or disk files.
- **Sandboxed Execution**: Subprocess/Wasm isolation for dangerous tools.

---

## 15. Living Changelog & Evolution Ledger

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
  - Perform codebase-wide review, style harmonization, edge-case analysis, and test suite hardening.

---

### [2026-09-25] - Codebase Review, Refactoring & Test Suite Hardening
- **Objective**: Conduct a comprehensive codebase review, restyle and comment all modules, expand test coverage for all edge cases across domain models, tool registry, HTTP provider, agent execution engine, and built-in tools.
- **Changes Made**:
  - Enhanced [`src/core/types.rs`](file:///home/nana/dev/harness/src/core/types.rs):
    - Added `Message::assistant_with_tool_calls(content, tool_calls)` for assistant messages containing thoughts/text alongside tool calls.
    - Added role inspection helper methods: `is_system()`, `is_user()`, `is_assistant()`, `is_tool()`.
    - Added unit tests for role checking and assistant turns with simultaneous content and tool calls.
  - Enhanced [`src/core/tool.rs`](file:///home/nana/dev/harness/src/core/tool.rs):
    - Added `contains(&self, name: &str) -> bool` and `unregister(&mut self, name: &str) -> Option<Box<dyn Tool>>` methods.
    - Added unit tests for tool existence checks, unregistration, and whitespace/empty argument parsing.
  - Enhanced [`src/core/provider.rs`](file:///home/nana/dev/harness/src/core/provider.rs):
    - Added unit tests for parsing multiple parallel tool calls in a single response and fallback handling for null content.
  - Enhanced [`src/core/agent.rs`](file:///home/nana/dev/harness/src/core/agent.rs):
    - Added unit tests for multi-tool calls in a single turn, multi-turn conversation memory persistence, `clear_history()`, and `config_mut()` / `registry_mut()` accessors.
  - Enhanced [`src/tools/calculator.rs`](file:///home/nana/dev/harness/src/tools/calculator.rs) and [`src/tools/echo.rs`](file:///home/nana/dev/harness/src/tools/echo.rs):
    - Added edge case tests for missing parameters, negative numbers, division by zero, and invalid data types (e.g. non-numeric operands).
- **Architectural Decisions**:
  - Retained strict zero-dependency bloat while maximizing API ergonomics and hardening against edge cases.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` expanded from 23 to 31 tests with 31/31 passing (100% success rate).
- **Next Steps**:
  - Plan Milestone 2 and open atomic issues #6 through #10 tagged 'MS2'.

---

### [2026-09-25] - STEP 2: Milestone 2 Architectural Specification & Issue Planning
- **Objective**: Author exhaustive specifications for Milestone 2 (Observability, Context Management & Filesystem Capabilities) in `DOCUMENTATION.md`, partition into 5 atomic issues, and prepare issue tickets.
- **Changes Made**:
  - Updated [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md):
    - Added Section 5: "STEP 2: Milestone 2 — Observability, Memory Pruning & Filesystem Capabilities".
    - Detailed Part 1: Token Usage Tracking (`Usage` type, OpenAI payload extraction, session metrics on `Agent`).
    - Detailed Part 2: Agent Event Hooks (`AgentHook` trait and lifecycle observer callbacks for tracing/logging/UI).
    - Detailed Part 3: Sandboxed Filesystem Tools (`ReadFileTool`, `WriteFileTool` with root-jail traversal protections).
    - Detailed Part 4: Context Retention & Pruning (`ContextPolicy` sliding window preserving initial `Role::System` directive).
    - Detailed Part 5: CLI REPL Observability & Diagnostics (Terminal hook output, `/stats`, `/tokens`, `/help`, and filesystem tools integration).
  - Synchronized [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) with Milestone 2 roadmap.
  - Opened 5 atomic GitHub issues for Milestone 2 tagged `MS2` and `agent-ready` (#6, #7, #8, #9, #10).
- **Architectural Decisions**:
  - Maintained zero runtime dependency bloat (pure standard library + `serde` + `serde_json` + `ureq`).
  - Separated concerns cleanly: observability decoupled via Observer pattern (`AgentHook`), token accounting separated into `Usage`, and security bounds enforced in `fs` tools.
- **Verification**:
  - Verified Markdown layout, table of contents links, and schema definitions.
  - Successfully created issues #6, #7, #8, #9, #10 via GitHub MCP tools.
- **Next Steps**:
  - Author Quickstart & Usage guide at top of documentation and begin Milestone 2 implementation.

---

### [2026-09-25] - Documentation: Comprehensive Quickstart & Usage Guide
- **Objective**: Author a prominent, exhaustive Quickstart & Usage guide at the top of `DOCUMENTATION.md` detailing build steps, CLI REPL launch configurations (Cloud & Local LLMs), environment variable options, REPL commands, and programmatic library usage examples.
- **Changes Made**:
  - Added Section 1: "Quickstart & Usage Guide" to [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md):
    - **1.1 Prerequisites & Compilation**: Build steps (`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`).
    - **1.2 Launching the Interactive CLI REPL**: Cloud OpenAI setup and local open-source LLM setups (Ollama, vLLM, Groq, DeepSeek, Mistral).
    - **1.3 Environment Variables Reference**: Complete reference table with descriptions and default fallbacks.
    - **1.4 REPL Commands & Example Workflows**: Interactive command reference (`history`, `clear`/`reset`, `exit`/`quit`) and realistic conversational transcript.
    - **1.5 Using HarnessMe as a Library (Rust API)**: Complete standalone Rust snippet showing custom tool definition, registration, provider initialization, and agent loop execution.
  - Updated Table of Contents and renumbered all sections across the entire document (1 through 15).
- **Verification**:
  - Verified all Markdown links, table formatting, code block syntax, and headers.
- **Next Steps**:
  - Proceed with implementing Milestone 2 Issue #6: Token Usage Tracking & Provider Metadata.

---

### [2026-09-25] - Implementation of Issue #11: Google Antigravity (AGY) Provider Support
- **Objective**: Implement first-class support for Google Antigravity (AGY) as an LLM provider in HarnessMe, enabling local language server discovery and custom CSRF/source header management.
- **Changes Made**:
  - Created [`AntigravityProvider`](file:///home/nana/dev/harness/src/core/provider.rs) in `src/core/provider.rs`:
    - Auto-discovery constructor `from_env()` resolving `ANTIGRAVITY_BASE_URL`, `AGY_BASE_URL`, or `http://{ANTIGRAVITY_LS_ADDRESS}/v1`.
    - Support for `ANTIGRAVITY_CSRF_TOKEN` forwarded via `X-Antigravity-CSRF-Token` header.
    - Support for `ANTIGRAVITY_SOURCE_METADATA` forwarded via `X-Antigravity-Source` header.
    - Default model preset `gemini-2.5-flash` with support for `gemini-2.5-pro` and `agy-pro`.
    - Added blanket `impl<P: Provider + ?Sized> Provider for Box<P>` to allow dynamic provider dispatch.
    - Added 2 comprehensive unit tests for builder methods and environment default resolution.
  - Updated [`src/core/mod.rs`](file:///home/nana/dev/harness/src/core/mod.rs) and [`src/lib.rs`](file:///home/nana/dev/harness/src/lib.rs) re-exports.
  - Updated [`src/main.rs`](file:///home/nana/dev/harness/src/main.rs) CLI REPL with dynamic provider autodetection (`HARNESS_PROVIDER="antigravity"`).
  - Updated [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md) (Quickstart Option D, Provider Subsystem, Environment Reference, Living Changelog) and synchronized [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md).
- **Architectural Decisions**:
  - Maintained strict zero-dependency bloat relying only on `ureq` + `serde`/`serde_json` + Rust standard library.
  - Enabled seamless compatibility with both local Antigravity Language Servers and cloud endpoints.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` expanded to 33 tests with 33/33 passing (100% success rate).
- **Next Steps**:
  - Proceed with Milestone 2 Issue #6: Token Usage Tracking & Provider Metadata.

---

### [2026-09-25] - Implementation of Issue #12: REPL Command Parsing & /agy Antigravity Configuration
- **Objective**: Implement a command parsing system in the CLI REPL (`src/main.rs`) and dynamic provider mutation on `Agent` (`src/core/agent.rs`), introducing `/agy` to inspect and configure the Antigravity provider in real time.
- **Changes Made**:
  - Enhanced [`Agent`](file:///home/nana/dev/harness/src/core/agent.rs) with `set_provider` and `set_boxed_provider` for dynamic runtime provider updates without loss of conversation history or tool registrations.
  - Implemented strongly typed command parser in [`src/main.rs`](file:///home/nana/dev/harness/src/main.rs):
    - `Command`: `Exit`, `Clear`, `History`, `Help`, `ProviderInfo`, `Agy(AgySubcommand)`, `UserPrompt(String)`.
    - `AgySubcommand`: `Status`, `Model(String)`, `Url(String)`, `Port(u16)`, `Csrf(Option<String>)`, `Key(Option<String>)`, `Temperature(f32)`, `Timeout(u64)`, `Reset`, `Switch`, `Help`.
  - Added formatted outputs for `/help`, `/agy status`, `/agy help`, `/provider`, and `/history`.
  - Added unit tests in `src/main.rs` for command parsing and in `src/core/agent.rs` for `set_provider` and `set_boxed_provider`.
  - Updated [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md) Section 1.4 and [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) Phase 12.
- **Architectural Decisions**:
  - Maintained zero dependency bloat: standard library pattern matching and token splitting without external CLI frameworks.
  - Supported both modern slash commands (`/agy`, `/help`, `/clear`) and legacy bare words for backwards compatibility.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 37 tests (34 library + 3 binary unit tests) with 37/37 passing (100% success rate).
- **Next Steps**:
  - Proceed with Milestone 2 Issue #13: Google Account Linking & Interactive Model Selection to /agy.

---

### [2026-09-25] - Implementation of Issue #13: Google Account Linking & Interactive Model Selection in `/agy`
- **Objective**: Implement interactive choices in the `/agy` command suite, allowing users to link their Google account / OAuth tokens and select models from a curated Gemini / Antigravity catalogue with numeric shortcuts.
- **Changes Made**:
  - Enhanced [`AntigravityProvider`](file:///home/nana/dev/harness/src/core/provider.rs):
    - Added `account_email: Option<String>` with builder methods `with_account(email)` and `with_optional_account(email)` and getter `account_email()`.
    - Auto-discovered `ANTIGRAVITY_ACCOUNT`, `GOOGLE_ACCOUNT`, and `AGY_ACCOUNT` in `from_env()`.
    - Added `SUPPORTED_MODELS` catalogue (`gemini-2.5-flash`, `gemini-2.5-pro`, `gemini-1.5-pro`, `gemini-1.5-flash`, `agy-pro`).
    - Added `resolve_model_name(input)` supporting numeric shortcuts (`1`, `2`, `3`, `4`, `5`) and aliases (`flash`, `pro`).
    - Added comprehensive unit tests for account builder, environment loading, and model resolution.
  - Enhanced [`src/main.rs`](file:///home/nana/dev/harness/src/main.rs):
    - Added `AgySubcommand::Menu`, `AgySubcommand::Link(Option<String>)`, `AgySubcommand::Account(String)`, `AgySubcommand::ModelList`.
    - Updated `parse_command` to route `/agy` or `/agy menu` to interactive selection menu, `/agy 1` to account linking, `/agy 2` to model selection, `/agy 3` to status, and `/agy 4` to provider switch.
    - Implemented `print_agy_menu` and `print_agy_models` presenting choices.
    - Added CLI tests covering all new subcommands, numeric shortcuts, and alias resolutions.
  - Updated [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md) (Section 1.4 & Section 15) and [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) (Phase 13).
- **Architectural Decisions**:
  - Maintained zero dependency bloat using Rust standard library pattern matching and formatted terminal output.
  - Interactive selection menu provides immediate feedback and straightforward numeric selection for terminal users.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 38 tests (35 library + 3 binary unit tests) with 38/38 passing (100% success rate).
- **Next Steps**:
  - Proceed with Milestone 2 Issue #14: Exclusive Google AI Provider & Browser-Based Google Sign-In in `/agy`.

---

### [2026-09-25] - Implementation of Issue #14: Exclusive Google AI Provider & Browser-Based Google Sign-In in `/agy`
- **Objective**: Configure HarnessMe CLI to exclusively use Google AI / Antigravity as the sole provider, eliminate OpenAI default fallbacks and warnings, implement cross-platform browser launch for Google Sign-In on `/agy`, and auto-discover local Google credentials (`~/.gemini/google_accounts.json` and `~/.gemini/oauth_creds.json`).
- **Changes Made**:
  - Enhanced [`AntigravityProvider`](file:///home/nana/dev/harness/src/core/provider.rs):
    - Added helper `read_local_google_account()` reading `~/.gemini/google_accounts.json` (`active` email).
    - Added helper `read_local_oauth_token()` reading `~/.gemini/oauth_creds.json` (`access_token`).
    - Updated `AntigravityProvider::from_env()` to automatically discover local account and bearer token credentials when environment variables are unset.
    - Added unit test `test_local_google_credentials_discovery`.
  - Refactored [`src/main.rs`](file:///home/nana/dev/harness/src/main.rs):
    - Added cross-platform browser launcher `open_browser(url: &str) -> io::Result<()>` using `xdg-open` (Linux), `open` (macOS), and `cmd /C start` (Windows).
    - Added `AgySubcommand::Login(Option<String>)` supporting `/agy`, `/agy login`, `/agy 1`.
    - Implemented `handle_google_signin`: displays branded Google Sign-In banner, launches default system browser, auto-discovers credentials, reconfigures the provider, and confirms readiness.
    - Configured CLI binary entrypoint exclusively for Google AI / Antigravity: removed OpenAI default fallback, removed OpenAI-missing warnings, updated status displays.
    - Updated test suite covering `/agy`, `/agy login`, `/agy 1`, `/agy menu`, and all subcommands.
  - Updated [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md) (Sections 1.2, 1.3, 1.4, and Section 15) and [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) (Phase 14).
- **Architectural Decisions**:
  - Maintained zero dependency bloat using Rust's standard library `std::process::Command` for cross-platform browser launching and standard file parsing for credential discovery.
  - Dedicated the interactive REPL binary to Google AI / Antigravity while preserving the generic `Provider` trait and `OpenAiCompatibleProvider` in the core library for modular multi-provider crate consumers.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 39 tests (36 library + 3 binary unit tests) with 39/39 passing (100% success rate).
  - Executed interactive CLI with `/provider`, `/agy menu`, `/agy 3`, and `/agy` demonstrating successful browser launch, automatic detection of `mavepith762@gmail.com`, and prompt readiness.
- **Next Steps**:
  - Proceed with Milestone 2 Issue #6: Token Usage Tracking & Provider Metadata.
