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
   - [5.1 Philosophy & Architectural Objectives](#51-philosophy--architectural-objectives)
   - [5.2 Part 1: Token Usage Tracking & Provider Metadata (`src/core/types.rs`, `src/core/provider.rs`)](#52-part-1-token-usage-tracking--provider-metadata-srccoretypesrs-srccoreproviderrs)
   - [5.3 Part 2: Agent Event Hooks & Lifecycle Observers (`src/core/agent.rs`)](#53-part-2-agent-event-hooks--lifecycle-observers-srccoreagentrs)
   - [5.4 Part 3: Standard Sandboxed Filesystem Tools (`src/tools/fs.rs`)](#54-part-3-standard-sandboxed-filesystem-tools-srctoolsfsrs)
   - [5.5 Part 4: Conversation Context Pruning & History Retention (`src/core/agent.rs`)](#55-part-4-conversation-context-pruning--history-retention-srccoreagentrs)
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

HarnessMe provides both an interactive terminal CLI (REPL) for hands-on agent experimentation and a decoupled Rust library crate to embed autonomous agents into your own systems and services.

---

### 1.1 Prerequisites & Compilation

#### Requirements
- **Rust Toolchain**: Rust 1.70+ (edition 2021) with `cargo` installed.
- Check your environment:
  ```bash
  rustc --version
  cargo --version
  ```

#### Building the Project
- **Debug build** (fast incremental compilation):
  ```bash
  cargo build
  ```
- **Optimized Release build**:
  ```bash
  cargo build --release
  ```
- **Run the Complete Test Suite**:
  ```bash
  cargo test
  ```
- **Linting & Code Formatting**:
  ```bash
  cargo clippy -- -D warnings
  cargo fmt --check
  ```

---

### 1.2 Launching the Interactive CLI REPL

The CLI entrypoint (`src/main.rs`) provides an interactive REPL with live tool execution, multi-turn conversation memory, and error self-correction, powered exclusively by **Google AI / Antigravity** using your **Google AI Pro subscription**:

1. **Automatic Discovery & Launch**:
   Launch HarnessMe directly:
   ```bash
   cargo run
   ```
   HarnessMe automatically connects to Google Cloud natively via the authenticated `agy` CLI binary using your Google account (`~/.gemini/google_accounts.json` and OAuth credentials). No local LLM server or fake localhost base URL is required!

2. **Browser-Based Google Sign-In**:
   If not yet signed in or if switching accounts, simply run `/agy` or `/agy login` inside the REPL. HarnessMe immediately opens your default system browser to authenticate with Google:
   ```text
   user > /agy
   ================ Google Sign-In (Google AI / Antigravity) ================
   Opening your web browser to authenticate with Google...
   If your browser does not open automatically, visit:
     https://accounts.google.com/

   system > Browser opened successfully.
   system > Detected local Google credentials!
   --------------------------------------------------------------------------
     Status       : SIGNED IN
     Account      : user@gmail.com (Google AI Pro Subscription: ACTIVE)
     Model        : gemini-3.1-pro-high
     Connection   : Google Cloud (Antigravity Native)
   system > Google AI (Antigravity) is active and ready to use.
   ==========================================================================
   ```

3. **Explicit Antigravity Configuration (Optional)**:
   ```bash
   export ANTIGRAVITY_MODEL="gemini-3.1-pro-high" # or gemini-3.8-flash-high, claude-sonnet-4-6
   export HARNESS_TEMPERATURE="0.7"
   export HARNESS_MAX_STEPS="10"
   cargo run
   ```

---

### 1.3 Environment Variables Reference

| Variable | Type | Default | Description |
|---|---|---|---|
| `ANTIGRAVITY_MODEL` / `HARNESS_MODEL` | String | `gemini-3.1-pro-high` | Model identifier for Antigravity provider (`gemini-3.1-pro-high`, `gemini-3.8-flash-high`, `claude-sonnet-4-6`). |
| `ANTIGRAVITY_ACCOUNT` / `GOOGLE_ACCOUNT` | String | Auto-Discovered | Linked Google account email address (discovered from `~/.gemini/google_accounts.json`). |
| `ANTIGRAVITY_BASE_URL` | URL | `Google Cloud (Antigravity Native)` | Connection mode; uses `agy` CLI pipe by default, or an explicit HTTP endpoint if configured. |
| `AGY_BIN` | Path | `which agy` | Explicit path to the `agy` CLI binary (defaults to `~/.local/bin/agy` or PATH). |
| `ANTIGRAVITY_API_KEY` / `AGY_API_KEY` | String | Auto-Discovered | Optional bearer token for Antigravity cloud gateway or auto-discovered OAuth token. |
| `ANTIGRAVITY_CSRF_TOKEN` | String | `None` | CSRF token passed via `X-Antigravity-CSRF-Token` header for secure language server communication. |
| `HARNESS_SYSTEM_PROMPT` | String | *"You are a helpful and concise AI assistant..."* | Custom system directives defining the agent's behavior and tone. |
| `HARNESS_MAX_STEPS` | Integer | `10` | Maximum iterative tool execution turns before halting (runaway loop guardrail). |
| `HARNESS_TEMPERATURE` | Float | `0.7` | Model sampling temperature (e.g. `0.0` for deterministic logic, `0.7` for general tasks). |

---

### 1.4 REPL Commands & Example Workflows

Once the REPL starts, you will see the interactive prompt:
```text
====================================================
      HarnessMe Agent Interactive CLI (Google AI)   
====================================================
 Provider     : Google AI / Antigravity
 Account      : user@gmail.com
 Model        : gemini-2.5-flash
 Base URL     : http://127.0.0.1:38035/v1
 Auth Key     : Configured (hidden)
 Max Steps    : 10
 Temperature  : 0.7
----------------------------------------------------
 Registered Tools: echo, calculator
 Commands: '/agy' to sign in with Google, '/help' for manual, '/exit' to quit.
====================================================

user > 
```

#### Interactive REPL Commands
HarnessMe supports intuitive slash commands as well as standard shell shorthand:

| Command | Shorthand | Description |
|---|---|---|
| `/agy` | `/agy login` | Prompt to sign in with Google & open default browser to authenticate |
| `/agy menu` | - | Display interactive menu with choices to sign in or change model |
| `/agy models` | `/agy model` | List supported Google Gemini and Antigravity models |
| `/agy model <name\|num>` | `/agy model 2` | Switch model by name or index (`1`=flash, `2`=pro, `3`=1.5-pro, `4`=1.5-flash, `5`=agy-pro) |
| `/agy status` | `/agy 3` | Inspect current Google AI / Antigravity configuration, linked account, and endpoints |
| `/provider` | `/providers` | Display active LLM provider configuration and status |
| `/history` | `history` | Display full multi-turn conversation memory with tool call identifiers |
| `/clear` | `clear`, `/reset` | Clear conversation memory and reset context |
| `/help` | `help`, `/?` | Display command summary and manual |
| `/exit` | `exit`, `/quit`, `quit` | Gracefully exit the interactive REPL session |

#### Antigravity Configuration (`/agy` Commands)
The `/agy` command suite presents an interactive choice menu and subcommands to sign in with Google via browser, select Gemini/Antigravity models, or mutate connection settings in real-time without restarting the process:

| Command | Shortcut | Example | Description |
|---|---|---|---|
| `/agy` / `/agy login` | `/agy 1` | `/agy` | Open system browser to sign in with Google and link credentials |
| `/agy menu` | - | `/agy menu` | Display interactive configuration menu |
| `/agy link [token\|email]` | - | `/agy link user@gmail.com` | Link Google account email or OAuth Bearer token (or `/agy link clear`) |
| `/agy account <email>` | - | `/agy account user@example.com` | Set Google account email (or `/agy account clear`) |
| `/agy models` / `/agy model` | `/agy 2` | `/agy models` | List supported Google Gemini and Antigravity models |
| `/agy model <name\|num>` | `/agy model 2` | `/agy model gemini-2.5-pro` | Switch model by name or index (`1`=flash, `2`=pro, `3`=1.5-pro, `4`=1.5-flash, `5`=agy-pro) |
| `/agy status` | `/agy 3` | `/agy status` | Inspect current Antigravity configuration, linked account, and active status |
| `/agy switch` | `/agy 4` | `/agy switch` | Confirm active provider is Google AI (Antigravity) |
| `/agy url <url>` | - | `/agy url http://127.0.0.1:38035/v1` | Update base endpoint URL |
| `/agy port <port>` | - | `/agy port 38035` | Shortcut to update base URL to `http://127.0.0.1:<port>/v1` |
| `/agy csrf <token\|none>` | - | `/agy csrf secret123` | Update or clear `X-Antigravity-CSRF-Token` header |
| `/agy key <key\|none>` | - | `/agy key token_abc` | Update or clear Bearer API key |
| `/agy temp <0.0 - 2.0>` | - | `/agy temp 0.2` | Update sampling temperature |
| `/agy timeout <secs>` | - | `/agy timeout 45` | Update HTTP request timeout |
| `/agy reset` | - | `/agy reset` | Reload all Antigravity settings from environment variables |
| `/agy help` | `/?` | `/agy help` | Display `/agy` command help manual |

#### Example Conversational Workflow
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
  Account  : user@gmail.com
  Model    : gemini-2.5-flash
  Endpoint : http://127.0.0.1:38035/v1
system > Google AI (Antigravity) is active and ready to use.
==========================================================================

user > /agy 2
---------------- Available Gemini & Antigravity Models ----------------
  [1] gemini-2.5-flash   : Gemini 2.5 Flash (Ultra-fast, multimodal reasoning & tool calling) (CURRENT)
  [2] gemini-2.5-pro     : Gemini 2.5 Pro (Advanced reasoning & deep code generation)
  [3] gemini-1.5-pro     : Gemini 1.5 Pro (Long 2M+ context window & complex analysis)
  [4] gemini-1.5-flash   : Gemini 1.5 Flash (Lightweight, ultra-low latency)
  [5] agy-pro            : Antigravity Pro Enterprise Model
-----------------------------------------------------------------------
To select a model, run: '/agy model <number|name>' (e.g. '/agy model 2' or '/agy model gemini-2.5-pro')

user > /agy model 2
system > Google AI model updated to 'gemini-2.5-pro'.

user > /agy status
---------------- Antigravity (AGY) Status ----------------
  Active on Agent : YES (Active)
  Google Account  : user@example.com
  Model           : gemini-2.5-pro
  Base URL        : http://127.0.0.1:38035/v1
  CSRF Token      : Configured
  API Key / Bearer: None
  Temperature     : 0.7
  Timeout         : 60s
----------------------------------------------------------

user > /agy model gemini-2.5-pro
system > Antigravity model updated to 'gemini-2.5-pro'.

user > Calculate (125 * 8.5) / 2 and tell me the result.
agent > The result of (125 * 8.5) / 2 is 531.25.

user > /history
system > Conversation history (4 turns):
  [0] system: You are a helpful and concise AI assistant equipped with tools...
  [1] user: Calculate (125 * 8.5) / 2 and tell me the result.
  [2] assistant (tool_calls: calculator): [No text content]
  [3] tool (call_id: call_xyz123): 531.25
  [4] assistant: The result of (125 * 8.5) / 2 is 531.25.
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
       {\n         \"model\": \"gpt-4o-mini\",\n         \"messages\": [...],\n         \"tools\": [...],\n         \"temperature\": 0.7\n       }
       ```
     - Note: If `tools` is empty, omit the `\"tools\"` field to support non-tool-calling models.
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
            - If execution fails (e.g. invalid arguments or runtime error), format error string: `\"Error: {err}\"`.
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
    pub fn new(provider: impl Provider + 'static, registry: ToolRegistry) -> Self { ... }
    pub fn with_config(provider: impl Provider + 'static, registry: ToolRegistry, config: AgentConfig) -> Self { ... }
    pub fn history(&self) -> &[Message] { ... }
    pub fn clear_history(&mut self) { ... }
    pub fn step(&mut self) -> Result<Option<String>, AgentError> { ... }
    pub fn run(&mut self, user_prompt: &str) -> Result<String, AgentError> { ... }
}
```

---

## 10. Configuration & Environment Reference

The following environment variables are supported by HarnessMe:

| Variable | Type | Default | Description |
|---|---|---|---|
| `ANTIGRAVITY_BASE_URL` | URL | `http://127.0.0.1:38035/v1` | Base endpoint URL for Google AI / Antigravity completions. |
| `ANTIGRAVITY_LS_ADDRESS` | Host:Port | `None` | Antigravity Language Server address (auto-sets base URL to `http://{LS_ADDRESS}/v1`). |
| `ANTIGRAVITY_CSRF_TOKEN` | String | `None` | CSRF token passed via `X-Antigravity-CSRF-Token` header. |
| `ANTIGRAVITY_SOURCE_METADATA` | String | `None` | Optional client provenance passed via `X-Antigravity-Source` header. |
| `ANTIGRAVITY_API_KEY` / `AGY_API_KEY` | String | `None` | Optional API key / bearer token for Antigravity cloud proxy. |
| `ANTIGRAVITY_MODEL` | String | `gemini-2.5-flash` | Default model when using Antigravity provider. |
| `OPENAI_API_KEY` | String | `None` | API key for OpenAI-compatible endpoints (`Bearer <key>`). |
| `OPENAI_BASE_URL` | URL | `https://api.openai.com/v1` | Base endpoint URL for OpenAI completions. |
| `HARNESS_MODEL` | String | `gpt-4o-mini` | Fallback model name if provider-specific model variable is unset. |
| `HARNESS_SYSTEM_PROMPT` | String | Default assistant prompt | Custom system instructions injected into conversation memory. |
| `HARNESS_MAX_STEPS` | Integer | `10` | Maximum iterative tool calls permitted in a single `Agent::run` call. |
| `HARNESS_TEMPERATURE` | Float | `0.7` | Sampling temperature (`0.0` = deterministic, `1.0` = creative). |

---

## 11. Extension & Integration Guide

### Creating a Custom Tool

To add a new tool to HarnessMe, implement the `Tool` trait and register it with the `ToolRegistry`:

```rust
use harnessme::core::tool::{Tool, ToolError};
use serde_json::json;

pub struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get current weather conditions for a given city."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "Name of the city, e.g. 'Tokyo' or 'Paris'"
                }
            },
            "required": ["city"]
        })
    }

    fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let city = args["city"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'city' argument".into()))?;

        // Replace with real API call or logic:
        Ok(format!("Weather in {city}: 22°C, Sunny"))
    }
}
```

Register and expose the tool:
```rust
let mut registry = ToolRegistry::new();
registry.register(WeatherTool);
```

### Implementing a Custom Provider

Any backend that produces text or tool calls can implement `Provider`:

```rust
use harnessme::core::provider::{Provider, ProviderError, ProviderResponse};
use harnessme::core::types::{Message, ToolDefinition};

pub struct MyCustomProvider;

impl Provider for MyCustomProvider {
    fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderResponse, ProviderError> {
        // Implement inference logic here
        Ok(ProviderResponse::Text("Hello from custom provider!".to_string()))
    }
}
```

---

## 12. Error Handling & Edge Cases

HarnessMe enforces robust error containment to ensure runaway loops, malformed model outputs, and external failures never crash the harness:

1. **Model Emits Invalid JSON Tool Arguments**:
   - Captured by `ToolRegistry::execute`.
   - Returns `ToolError::InvalidArguments(...)`.
   - Injected into context history as `Role::Tool` message (`"Error: Failed to parse tool arguments: ..."`).
   - Allows the LLM to inspect the syntax failure and emit corrected arguments on the next step.

2. **Model Requests Unregistered Tool**:
   - `ToolRegistry::execute` returns `ToolError::ToolNotFound(...)`.
   - Injected into context as `Role::Tool` message (`"Error: Tool '<name>' not found..."`).
   - The LLM receives immediate feedback that the tool does not exist and can choose an alternative strategy.

3. **Infinite Tool Loops (Runaway Condition)**:
   - Guarded by `config.max_iterations`.
   - If reached, `Agent::run` terminates immediately with `Err(AgentError::MaxIterationsExceeded)`.

4. **HTTP Transport & Provider Errors**:
   - Network timeouts, bad responses, and HTTP status codes (e.g. 401, 429, 500) are mapped into strongly typed `ProviderError::HttpError` or `ProviderError::ApiError`.
   - Propagated to caller as `AgentError::Provider(...)`.

---

## 13. Testing & Verification Strategy

The repository maintains strict test coverage across unit, integration, and CLI layers:

- **Type Serialization Tests** (`src/core/types.rs`):
  - Validates `Role` enum lowercase JSON serialization (`"system"`, `"user"`, `"assistant"`, `"tool"`).
  - Validates full roundtrip serialization of messages with and without tool calls.
  - Ensures optional fields omit `null` entries to match OpenAI protocol requirements.
- **Tool Registry Tests** (`src/core/tool.rs`):
  - Tests dynamic registration and schema generation.
  - Verifies tool lookup and error propagation on unregistered names.
  - Validates malformed argument handling.
- **Provider Tests** (`src/core/provider.rs`):
  - Tests request payload serialization.
  - Tests response parsing for both conversational text and single/multi tool calls.
  - Mock provider tests asserting deterministic multi-step agent behavior.
- **Agent Loop Tests** (`src/core/agent.rs`):
  - Direct text execution without tools.
  - Multi-turn tool calling loops with mock providers.
  - Verification of max iteration termination guardrails.
  - Self-correction verification upon tool failure.
- **Tool Implementations** (`src/tools/`):
  - `EchoTool`: Echoes payload cleanly.
  - `CalculatorTool`: Verifies addition, subtraction, multiplication, division, expression parsing, and division-by-zero protection.

### Running Tests
```bash
cargo test
```

---

## 14. Future Roadmap

- **Milestone 2: Observability & Sandboxing** (Underway):
  - Issue #6: Token Usage Tracking & Provider Metadata.
  - Issue #7: Agent Lifecycle Event Hooks (`AgentHook`).
  - Issue #8: Standard Sandboxed Filesystem Tools (`ReadFileTool`, `WriteFileTool`).
  - Issue #9: Conversation Context Pruning & History Retention (`ContextPolicy::SlidingWindow`).
  - Issue #10: CLI REPL Observability & Diagnostic Commands (`/stats`, `/tools`).
  - Issue #11: Google Antigravity (AGY) Provider Support.
  - Issue #12: REPL Command Parsing & `/agy` Configuration.
  - Issue #13: Google Account Linking & Interactive Model Selection in `/agy`.
  - Issue #14: Exclusive Google AI Provider & Browser-Based Sign-In.
  - Issue #15: Native Google AI Pro Subscription Integration via `agy`.
- **Milestone 3: Advanced Provider Ecosystem**:
  - Direct streaming SSE support (`StreamingProvider`).
  - Anthropic Native Messages API client.
  - Local model runtime integrations.
- **Milestone 4: Multi-Agent Orchestration & Persistence**:
  - Router / Dispatcher agent architecture.
  - Sub-agent delegation and state isolation.
  - SQLite session persistence.

---

## 15. Living Changelog & Evolution Ledger

### [2026-09-25] - Implementation of Issue #1: Project Setup & Core Domain Types
- **Objective**: Establish the foundation of the agent harness in Rust with minimal external dependencies.
- **Changes Made**:
  - Initialized `Cargo.toml` with `serde`, `serde_json`, and `ureq`.
  - Implemented `Role`, `Message`, `ToolCall`, `FunctionCall`, `ToolDefinition`, `FunctionDefinition`, and `ToolResult` in `src/core/types.rs`.
  - Added comprehensive unit tests for serialization, deserialization, and JSON schema compliance.
  - Exported core types in `src/core/mod.rs` and `src/lib.rs`.
- **Verification**:
  - `cargo check --all-targets` passed with 0 errors.
  - `cargo test` ran 6 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Phase 2: Tool Abstraction and `ToolRegistry`.

---

### [2026-09-25] - Implementation of Issue #2: Tool Abstraction & ToolRegistry
- **Objective**: Create decoupled tool abstractions and an in-memory registry to manage, validate, and execute tools.
- **Changes Made**:
  - Defined `Tool` trait and `ToolError` enum in `src/core/tool.rs`.
  - Implemented `ToolRegistry` supporting registration, schema export, and dynamic execution.
  - Implemented reference tools: `EchoTool` in `src/tools/echo.rs` and `CalculatorTool` in `src/tools/calculator.rs`.
  - Added unit tests for tools and registry.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 11 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Phase 3: Provider Abstraction & OpenAI Client.

---

### [2026-09-25] - Implementation of Issue #3: Provider Abstraction & OpenAI Client
- **Objective**: Create the inference abstraction layer and an OpenAI-compatible HTTP client using synchronous blocking I/O (`ureq`).
- **Changes Made**:
  - Defined `Provider` trait, `ProviderResponse`, and `ProviderError` in `src/core/provider.rs`.
  - Implemented `OpenAiCompatibleProvider` supporting custom base URLs, API keys, temperature, and request timeouts.
  - Added unit tests for request serialization, response parsing, and mock provider dispatch.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 20 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Phase 4: Agent Execution Loop & Guardrails.

---

### [2026-09-25] - Implementation of Issue #4: Agent Execution Loop & Guardrails
- **Objective**: Implement the recursive agent execution engine managing conversation memory, tool execution, and runaway loop prevention.
- **Changes Made**:
  - Implemented `AgentConfig`, `AgentError`, and `Agent` in `src/core/agent.rs`.
  - Implemented `step()` and `run()` execution loops.
  - Implemented self-correction error recovery by injecting tool failures into context memory as `Role::Tool` messages.
  - Added unit tests for direct text responses, multi-turn tool calling loops, max iteration limits, and tool error recovery.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 25 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Phase 5: CLI Demo & Interactive REPL.

---

### [2026-09-25] - Implementation of Issue #5: CLI Demo & Interactive REPL
- **Objective**: Provide an interactive terminal REPL for hands-on agent experimentation and tool verification.
- **Changes Made**:
  - Implemented `src/main.rs` with environment parsing (`OPENAI_API_KEY`, `OPENAI_BASE_URL`, `HARNESS_MODEL`, `HARNESS_SYSTEM_PROMPT`, `HARNESS_MAX_STEPS`).
  - Pre-registered `CalculatorTool` and `EchoTool`.
  - Added stdin command loop supporting interactive queries and `/exit`, `/quit`, `/history`, `/clear`.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 25 tests with 100% success rate.
  - Executed interactive CLI demo verifying tool invocation and conversation history.
- **Next Steps**:
  - Plan and execute Milestone 2 issues.

---

### [2026-09-25] - Implementation of Issue #11: Google Antigravity (AGY) Provider Support
- **Objective**: Add first-class support for Google Antigravity (AGY) language models and language servers.
- **Changes Made**:
  - Implemented `AntigravityProvider` in `src/core/provider.rs` with automatic discovery of `ANTIGRAVITY_LS_ADDRESS`, `ANTIGRAVITY_CSRF_TOKEN`, and `ANTIGRAVITY_BASE_URL`.
  - Implemented header injection for `X-Antigravity-CSRF-Token` and `X-Antigravity-Source`.
  - Added blanket `Provider` implementation for `Box<P>` / `Box<dyn Provider>`.
  - Updated CLI REPL in `src/main.rs` to autodetect and dynamically instantiate `AntigravityProvider`.
  - Added unit tests for builder methods and environment loading.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 28 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Milestone 2 Issue #12: REPL Command Parsing & /agy Configuration.

---

### [2026-09-25] - Implementation of Issue #12: REPL Command Parsing & `/agy` Configuration
- **Objective**: Add dynamic command parsing to the CLI REPL and implement the `/agy` configuration command suite.
- **Changes Made**:
  - Added `set_provider` and `set_boxed_provider` on `Agent` in `src/core/agent.rs`.
  - Implemented `Command` and `AgySubcommand` parser in `src/main.rs`.
  - Added `/agy` subcommands: `status`, `model`, `url`, `port`, `csrf`, `key`, `temp`, `timeout`, `reset`, `switch`, `help`.
  - Added unit tests for command parsing and provider replacement.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 37 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Milestone 2 Issue #13: Google Account Linking & Model Selection in /agy.

---

### [2026-09-25] - Implementation of Issue #13: Google Account Linking & Interactive Model Selection in `/agy`
- **Objective**: Implement interactive choices in `/agy`, allowing users to link Google accounts and select models with numeric shortcuts.
- **Changes Made**:
  - Enhanced `AntigravityProvider` with `account_email` metadata, builder methods, and getters.
  - Added `SUPPORTED_MODELS` catalogue and `resolve_model_name` for numeric shortcuts and aliases.
  - Added interactive selection menu in `src/main.rs` (`/agy menu`, `/agy 1`, `/agy 2`, `/agy 3`, `/agy 4`).
  - Added unit tests for account configuration and model shortcuts.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo test` ran 38 tests with 100% success rate.
- **Next Steps**:
  - Proceed with Milestone 2 Issue #14: Exclusive Google AI Provider & Browser-Based Sign-In.

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
  - Proceed with Milestone 2 Issue #15: Native Google AI Pro Subscription Integration via `agy`.

---

### [2026-09-25] - Implementation of Issue #15: Native Google AI Pro Subscription Integration via `agy`
- **Objective**: Connect HarnessMe directly to the user's active Google AI Pro subscription (`mavepith762@gmail.com`), eliminate misleading localhost endpoints, and execute Google Cloud models natively via the local authenticated `agy` CLI binary using `gemini-3.1-pro-high` as the default flagship model.
- **Changes Made**:
  - Refactored [`AntigravityProvider`](file:///home/nana/dev/harness/src/core/provider.rs):
    - Added `find_agy_binary()` to dynamically locate the system `agy` CLI executable (`AGY_BIN`, `~/.local/bin/agy`, `which agy`).
    - Added `build_prompt_with_tools(messages, tools)` formatting system directives, multi-turn history, and strict JSON tool schema definitions for Gemini.
    - Added `extract_tool_calls_from_text(text)` parsing markdown code blocks, JSON payloads, or embedded tool call schemas into `Vec<ToolCall>`.
    - Added `complete_agy_cli(messages, tools)` executing `agy --model <model> --output-format json --print <prompt>` using the active Google authentication keyring.
    - Updated `Provider::complete` to default to `complete_agy_cli` with cloud connectivity, preserving `complete_http` only for custom non-localhost HTTP endpoints.
    - Updated model catalogue and numeric shortcuts for Google AI Pro: `gemini-3.1-pro-high` (1 / pro), `gemini-3.8-flash-high` (2 / flash), `gemini-3.7-flash-high` (3), `gemini-3.6-flash-high` (4), `claude-sonnet-4-6` (5 / sonnet), `claude-opus-4-6-thinking` (6 / opus).
    - Added unit tests: `test_build_prompt_with_tools`, `test_extract_tool_calls_from_text`, `test_find_agy_binary`, and updated model catalogue assertions.
  - Updated [`src/main.rs`](file:///home/nana/dev/harness/src/main.rs):
    - Updated banner, `/agy status`, and `/agy menu` to reflect `Google AI Pro Subscription: ACTIVE`, Google Cloud connection, and `gemini-3.1-pro-high`.
    - Updated test suite covering model shortcuts and command parsing.
  - Updated [`DOCUMENTATION.md`](file:///home/nana/dev/harness/DOCUMENTATION.md) (Sections 1.2, 1.3, 15) and [`PLAN.md`](file:///home/nana/dev/harness/PLAN.md) (Phase 15).
- **Architectural Decisions**:
  - Zero external dependency bloat: utilizes Rust's `std::process::Command`, `std::fs`, and `serde_json`.
  - Truly connects the harness to the user's paid Google AI Pro cloud infrastructure without requiring local server daemons or mock endpoints.
- **Verification**:
  - `cargo check --all-targets` passed cleanly.
  - `cargo clippy -- -D warnings` passed with 0 warnings.
  - `cargo fmt --check` passed cleanly.
  - `cargo test` ran 42 tests (39 library + 3 binary unit tests) with 42/42 passing (100% success rate).
  - Executed end-to-end interactive CLI test with prompt `What is 15 * 8?`, verifying live Google AI Pro inference and accurate response generation (`agent > 15 * 8 is 120.`).
- **Next Steps**:
  - Proceed with Milestone 2 Issue #6: Token Usage Tracking & Provider Metadata.
