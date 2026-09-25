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

HarnessMe can be embedded directly into any Rust application. Add it to your `Cargo.toml`:

```toml
[dependencies]
harnessme = { path = "../path/to/harnessme" }
```

```rust
use harnessme::{
    Agent, AgentConfig, CalculatorTool, EchoTool, AntigravityProvider, ToolRegistry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize the Google AI provider
    let provider = AntigravityProvider::from_env();

    // 2. Register tools
    let mut registry = ToolRegistry::new();
    registry.register(EchoTool);
    registry.register(CalculatorTool);

    // 3. Configure the agent guardrails
    let config = AgentConfig::new()
        .with_system_prompt("You are a helpful calculation assistant.")
        .with_max_iterations(5);

    // 4. Instantiate the Agent
    let mut agent = Agent::with_config(provider, registry, config);

    // 5. Run the autonomous loop
    let response = agent.run("Calculate (45 * 12) + 180")?;
    println!("Agent Result: {response}");

    Ok(())
}
```

---

## 2. Introduction & Architectural Goals

**HarnessMe** is an AI agent harness written in Rust. It provides the core runtime engine that allows Large Language Models (LLMs) to function as autonomous agents: accepting user prompts, managing conversation history, reasoning about available tools, executing tool calls in a secure/controlled environment, and iterating until the task is resolved.

### Core Design Principles

- **Minimal Dependencies**: Fast compilation and small binary footprints. Core runtime relies strictly on standard library primitives, `serde`/`serde_json` for serialization, and a lightweight HTTP client (`ureq`).
- **Trait-Based Modularity**: Providers (`Provider`) and Tools (`Tool`) are defined as decoupled traits. The harness is completely agnostic to the underlying LLM provider.
- **Strict Guardrails & Safety**: Enforces maximum execution iterations (`max_iterations`) to prevent runaway loops and runaway token consumption.
- **Explicit Error Propagation**: Uses strongly typed errors with no unhandled panics or unwraps in production pathways.

---

## 3. High-Level Architecture

```
                                  ┌────────────────────────┐
                                  │       User Input       │
                                  └───────────┬────────────┘
                                              │
                                              ▼
                                 ┌──────────────────────────┐
                                 │   Agent Execution Loop   │
                                 │   (Context & Iterations) │
                                 └──────┬────────────▲──────┘
                                        │            │
            ┌───────────────────────────┴─┐        ┌─┴───────────────────────────┐
            │                             │        │                             │
            ▼                             ▼        ▼                             │
  ┌───────────────────┐         ┌─────────────────────┐                          │
  │  Provider Trait   │         │  ToolRegistry &     │                          │
  │  (HTTP / LLM API) │         │  Tool Trait         │                          │
  └─────────┬─────────┘         └──────────┬──────────┘                          │
            │                              │                                     │
            ▼                              ▼                                     │
┌───────────────────────┐       ┌─────────────────────┐                          │
│  Gemini / Antigravity │       │ Calculator, Echo,   │                          │
│  Local LS / Remote    │       │ Filesystem Tools... │──────────────────────────┘
└───────────────────────┘       └─────────────────────┘
```

---

## 4. STEP 1: Milestone 1 — Minimal Working Agent Harness ("Make It Work First")

### 4.1 Philosophy: Make It Work, Make It Perfect Later
Build a lean, strictly sequential agent runtime loop with minimal dependencies, clean trait boundaries, and robust safety limits.

### 4.2 Part 1: Manifest & Minimal Dependencies (`Cargo.toml`)
- `serde`, `serde_json`
- `ureq` (with `json` feature)

### 4.3 Part 2: Core Domain Models & Message IR (`src/core/types.rs`)
- `Role`: `System`, `User`, `Assistant`, `Tool`
- `Message`: Envelope storing role, content, tool_calls, and tool_call_id
- `ToolCall`, `FunctionCall`, `ToolDefinition`, `FunctionDefinition`, `ToolResult`

### 4.4 Part 3: Tool Abstraction & Registry (`src/core/tool.rs`)
- `Tool` trait: `name()`, `description()`, `parameters_schema()`, and `execute()`
- `ToolRegistry`: In-memory storage, definition export, and execution dispatch

### 4.5 Part 4: Provider Abstraction & HTTP Client (`src/core/provider.rs`)
- `Provider` trait: Synchronous completion interface
- `ProviderResponse`: `Text` or `ToolCalls`
- `OpenAiCompatibleProvider` & `AntigravityProvider` implementations

### 4.6 Part 5: Agent Execution Loop & Safety Limits (`src/core/agent.rs`)
- `AgentConfig`: `system_prompt` and `max_iterations`
- `Agent`: Turn execution, tool dispatch, error formatting into conversation history

### 4.7 Part 6: CLI Interactive Demo & REPL (`src/main.rs`)
- Terminal loop with Google AI exclusivity, browser sign-in, and real-time configuration

### 4.8 Milestone 1 Acceptance Criteria
- 100% test suite pass rate
- Zero unhandled unwraps
- Complete self-correction on tool failure

---

## 5. STEP 2: Milestone 2 — Observability, Memory Pruning & Filesystem Capabilities

### 5.1 Architectural Scope & Core Drivers
Extend the harness with token usage accounting, lifecycle hooks, safe sandboxed filesystem operations, and sliding-window context pruning.

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

### `AntigravityProvider`
First-class provider implementation designed for Google Antigravity (AGY) endpoints, cloud gateways, and local Antigravity Language Servers.

---

## 9. Agent Execution Engine

### Configuration (`AgentConfig`)
```rust
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub system_prompt: Option<String>,
    pub max_iterations: usize, // Default: 10
}
```

---

## 10. Configuration & Environment Reference

| Environment Variable | Description | Default |
|---|---|---|
| `ANTIGRAVITY_BASE_URL` | Base endpoint URL for Antigravity (AGY) completions | `http://127.0.0.1:38035/v1` |
| `ANTIGRAVITY_MODEL` | Target model for Antigravity provider | `gemini-2.5-flash` |
| `ANTIGRAVITY_ACCOUNT` | Linked Google account email | Auto-detected from `~/.gemini/` |
| `ANTIGRAVITY_CSRF_TOKEN` | CSRF token for secure Antigravity Language Server RPC | Auto-detected |
| `ANTIGRAVITY_API_KEY` | Bearer authorization token | Auto-detected |
| `HARNESS_MAX_STEPS` | Maximum tool execution loop iterations | `10` |

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

1. **Malformed JSON Arguments**: Feeds parse failure back as `Role::Tool` message.
2. **Unknown Tool Invocations**: `ToolError::ToolNotFound` returned in context.
3. **Provider Network Errors**: Converted to `ProviderError::HttpError` or `ProviderError::ApiError`.
4. **Max Iterations Exceeded**: Guardrail terminates with `AgentError::MaxIterationsExceeded`.

---

## 13. Testing & Verification Strategy

- **Unit Tests**: Domain models, ToolRegistry, provider parsing, and agent loop.
- **Mock Provider Tests**: Deterministic testing with pre-arranged tool calls.
- **Integration Tests**: Verification against live or local endpoints.

---

## 14. Future Roadmap

- Token & Context Window Pruning
- Asynchronous & Streaming Pipeline
- State Persistence
- Sandboxed Execution

---

## 15. Living Changelog & Evolution Ledger

### [2026-09-25] - System Initialization & Comprehensive Documentation
- Initial project setup, architectural roadmap, and documentation.

---

### [2026-09-25] - Implementation of Phase 1 / Issue #1: Core Domain Types & Manifest
- Implemented `Cargo.toml`, `types.rs`, and initial test suite.

---

### [2026-09-25] - Implementation of Phase 2 / Issue #2: Tool Trait, Registry & Reference Tools
- Implemented `Tool` trait, `ToolRegistry`, `EchoTool`, and `CalculatorTool`.

---

### [2026-09-25] - AGENTS.md Operational Update: Exclusive MCP GitHub Operations
- Mandated exclusive use of `github-mcp-server` MCP tools for all GitHub repository operations.

---

### [2026-09-25] - Implementation of Phase 3 / Issue #3: Provider Trait & OpenAI-Compatible Client
- Implemented `Provider` trait, `ProviderResponse`, `ProviderError`, and `OpenAiCompatibleProvider`.

---

### [2026-09-25] - Implementation of Phase 4 / Issue #4: Agent Execution Loop & Safety Limits
- Implemented `Agent` loop, `AgentConfig`, safety guardrails, and self-correcting error handling.

---

### [2026-09-25] - Implementation of Phase 5 / Issue #5: CLI Interactive REPL & Environment Configuration
- Implemented `src/main.rs` terminal REPL and interactive commands (`exit`, `clear`, `history`).

---

### [2026-09-25] - Codebase Review, Refactoring & Test Suite Hardening
- Hardened test suite to 31 tests covering all edge cases.

---

### [2026-09-25] - STEP 2: Milestone 2 Architectural Specification & Issue Planning
- Planned Milestone 2 and created issues #6 through #10.

---

### [2026-09-25] - Documentation: Comprehensive Quickstart & Usage Guide
- Added Section 1: Quickstart & Usage Guide to `DOCUMENTATION.md`.

---

### [2026-09-25] - Implementation of Issue #11: Google Antigravity (AGY) Provider Support
- Implemented `AntigravityProvider` with local language server discovery and CSRF management.

---

### [2026-09-25] - Implementation of Issue #12: REPL Command Parsing & /agy Antigravity Configuration
- Implemented command parser and dynamic provider switching via `/agy`.

---

### [2026-09-25] - Implementation of Issue #13: Google Account Linking & Interactive Model Selection in `/agy`
- Added Google account linking, model catalog, and interactive menu to `/agy`.

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
