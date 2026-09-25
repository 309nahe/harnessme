# Project Plan: Minimal Agentic AI Harness in Rust

## 1. Overview & Objectives

The goal of this project is to build a lightweight, robust, and extensible AI agent harness from scratch in Rust, prioritizing **minimal dependencies**, clean design, and clarity.

The harness provides the runtime loop necessary for an LLM to act as an agent: taking user input, maintaining conversation context, deciding whether to call tools, executing those tools in a controlled environment, and returning the final output.

### Core Principles
- **Minimal Dependencies**: Rely on standard library wherever possible. Limit external crates to essential serialization (`serde`, `serde_json`) and a lightweight HTTP client.
- **Trait-Based Modularity**: Decouple the agent runner from specific LLM providers and tool implementations via clean traits.
- **Safety & Guardrails**: Enforce strict loop limits (max steps/iterations) and robust error handling to prevent runaway loops.
- **Simplicity First**: Start with a synchronous or lean async architecture before adding complex features like streaming or multi-agent orchestration.

---

## 2. Dependency Strategy

To keep compile times fast and binary sizes small, we keep external crates to the absolute minimum:

| Crate | Purpose | Rationale |
|---|---|---|
| `serde`, `serde_json` | Serialization & Deserialization | Universal format for LLM function/tool schemas and HTTP requests |
| `ureq` (or `reqwest` if async) | HTTP Client | Lightweight, synchronous HTTP client with minimal dependencies (no mandatory Tokio runtime if sync) |
| Standard Library (`std`) | Threads, I/O, Error Handling, Memory | `std::sync::Arc`, `std::collections::HashMap`, `std::error::Error` |

*(Optional future additions: `tokio` only if async concurrency is required, `thiserror` for cleaner error enums).*

---

## 3. Architecture & Core Concepts

```
┌────────────────────────────────────────────────────────┐
│                      Agent Loop                        │
│                                                        │
│  ┌──────────────┐     Prompt + History     ┌────────┐  │
│  │ Conversation │ ───────────────────────> │  LLM   │  │
│  │   Context    │ <─────────────────────── │ Client │  │
│  └──────┬───────┘   Response / Tool Call   └────────┘  │
│         ▲                                       │      │
│         │                                       ▼      │
│         │            Tool Execution       ┌──────────┐ │
│         └─────────────────────────────────┤   Tool   │ │
│                   Result / Output         │ Registry │ │
│                                           └──────────┘ │
└────────────────────────────────────────────────────────┘
```

### Key Modules

1. **`core::types`**:
   - `Role`: `System`, `User`, `Assistant`, `Tool`.
   - `Message`: Role, content, optional tool calls, and tool call IDs.
   - `ToolCall`: ID, function name, raw JSON arguments.
   - `ToolResult`: Tool call ID, output text, or error message.

2. **`core::provider`**:
   - `trait Provider`: Abstract interface for LLM completions.
   - Default implementation: `OpenAiCompatibleProvider` (works with OpenAI, LocalAI, vLLM, Ollama, Groq, etc.).

3. **`core::tool`**:
   - `trait Tool`:
     - `name(&self) -> &str`
     - `description(&self) -> &str`
     - `parameters_schema(&self) -> serde_json::Value`
     - `execute(&self, args: serde_json::Value) -> Result<String, ToolError>`
   - `ToolRegistry`: Container to register, look up, and serialize tool definitions for the provider.

4. **`core::agent`**:
   - `AgentConfig`: Model name, temperature, system prompt, max iterations (guardrail).
   - `Agent`: Holds context history, provider reference, and `ToolRegistry`.
   - `Agent::step()`: Executes a single turn (LLM query -> tool execution if needed).
   - `Agent::run()`: Loops until the model produces a final answer or reaches max iterations.

---

## 4. Phased Implementation Roadmap

### Phase 1: Project Setup & Core Domain Types
- [x] Initialize `Cargo.toml` with minimal dependencies (`serde`, `serde_json`, chosen HTTP client).
- [x] Implement foundational domain data models (`Message`, `Role`, `ToolCall`, `ToolDefinition`, `ToolResult`).
- [x] Implement serialization/deserialization tests for tool definitions and messages.

### Phase 2: Tool Abstraction & In-Memory Registry
- [x] Define the `Tool` trait and `ToolError` type.
- [x] Implement `ToolRegistry` for registering and querying tools by name.
- [x] Write 1-2 standard test tools (e.g., `CalculatorTool`, `CurrentTimeTool`, or `EchoTool`).
- [x] Verify argument parsing and error propagation within tool executions.

### Phase 3: Provider Abstraction & OpenAI-Compatible Client
- [x] Define the `Provider` trait (input: list of messages + tool definitions; output: `ProviderResponse`).
- [x] Implement a minimal OpenAI-compatible HTTP client using `ureq` (or minimal async).
- [x] Handle completion payload formatting, header authorization, and error code parsing.
- [x] Add integration/mock tests for provider responses (both plain text response and tool-call response).

### Phase 4: The Agent Execution Loop
- [x] Implement `Agent` state and memory (message history list).
- [x] Implement the execution loop (`Agent::run`):
  1. Append user prompt to history.
  2. Send history + registered tools to provider.
  3. Inspect provider response:
     - If response has tool calls: execute each tool via `ToolRegistry`, record tool outputs as `Role::Tool` messages, repeat loop.
     - If response is text only: append assistant message and return final response.
  4. Enforce `max_iterations` counter to prevent infinite tool-calling loops.
- [x] Handle tool execution errors gracefully (feed errors back to LLM context for self-correction).


### Phase 5: Verification & CLI Demo
- [x] Create a minimal `main.rs` binary.
- [x] Read API configuration (API key, base URL, model name) from environment variables (e.g., `OPENAI_API_KEY`, `OPENAI_BASE_URL`).
- [x] Provide an interactive REPL in the terminal to converse with the agent and verify autonomous tool usage.
- [x] Validate edge cases: invalid tool arguments, tool failures, model hallucinated tool names.

---

## 5. Milestone 2: Observability, Memory Pruning & Filesystem Capabilities

### Phase 6 (MS2 - Issue #6): Token Usage Tracking & Provider Metadata
- [ ] Define `Usage` struct in `src/core/types.rs` (`prompt_tokens`, `completion_tokens`, `total_tokens`).
- [ ] Extract completion token usage in `OpenAiCompatibleProvider` and include in `ProviderResponse`.
- [ ] Aggregate lifetime and per-turn token metrics on `Agent`.
- [ ] Add unit tests for token usage parsing and agent aggregation.

### Phase 7 (MS2 - Issue #7): Agent Lifecycle Event Hooks
- [ ] Define `AgentHook` trait in `src/core/agent.rs` (`on_step_start`, `on_tool_call`, `on_tool_result`, `on_step_complete`, `on_error`).
- [ ] Add hook registration to `AgentConfig` and `Agent`.
- [ ] Safely dispatch lifecycle hooks during `Agent::step` and `Agent::run`.
- [ ] Add unit tests for hook invocation sequences during text and tool turns.

### Phase 8 (MS2 - Issue #8): Standard Sandboxed Filesystem Tools
- [ ] Implement `ReadFileTool` in `src/tools/fs.rs` with base directory jail validation.
- [ ] Implement `WriteFileTool` in `src/tools/fs.rs` with directory creation and root boundary checks.
- [ ] Re-export filesystem tools in `src/tools/mod.rs` and `src/lib.rs`.
- [ ] Add unit tests for reading, writing, missing files, and path traversal escape attempts (`../`).

### Phase 9 (MS2 - Issue #9): Conversation Context Pruning & History Retention
- [ ] Define `ContextPolicy` (`Unbounded`, `SlidingWindow { max_messages }`) in `src/core/agent.rs`.
- [ ] Implement history pruning algorithm preserving initial `Role::System` directive.
- [ ] Integrate automated pruning before provider calls in `Agent::step`.
- [ ] Add unit tests for sliding window pruning, system prompt retention, and history truncation.

### Phase 10 (MS2 - Issue #10): CLI REPL Observability & Diagnostic Commands
- [ ] Implement `CliObserverHook` in `src/main.rs` to print live tool execution and completion logs.
- [ ] Add `/stats` / `/tokens` REPL command to display cumulative token usage and step counts.
- [ ] Add `/tools` and `/help` REPL diagnostic commands.
- [ ] Register `ReadFileTool` and `WriteFileTool` in REPL workspace.

### Phase 11 (MS2 - Issue #11): Google Antigravity (AGY) Provider Support
- [x] Implement `AntigravityProvider` in `src/core/provider.rs` with automatic environment discovery (`ANTIGRAVITY_BASE_URL`, `ANTIGRAVITY_LS_ADDRESS`, `ANTIGRAVITY_API_KEY`, `ANTIGRAVITY_CSRF_TOKEN`).
- [x] Support header injection (`X-Antigravity-CSRF-Token`, `X-Antigravity-Source`).
- [x] Implement blanket `Provider` trait for `Box<dyn Provider>` / `Box<P>`.
- [x] Integrate Antigravity provider autodetection and dynamic loading in CLI REPL (`src/main.rs`).
- [x] Add unit tests for builder methods, environment defaults, and provider dispatch.
- [x] Re-export `AntigravityProvider` in `src/core/mod.rs` and `src/lib.rs`.

### Phase 12 (MS2 - Issue #12): REPL Command Parsing & /agy Configuration Command
- [x] Add `set_provider` and `set_boxed_provider` on `Agent` in `src/core/agent.rs` for dynamic runtime provider mutation.
- [x] Implement strongly typed `Command` and `AgySubcommand` parser in `src/main.rs`.
- [x] Support slash commands (`/help`, `/clear`, `/history`, `/exit`, `/quit`, `/provider`) and legacy bare commands.
- [x] Implement `/agy` subcommands: `/agy status`, `/agy model <name>`, `/agy url <url>`, `/agy port <port>`, `/agy csrf <token|none>`, `/agy key <key|none>`, `/agy temp <val>`, `/agy timeout <secs>`, `/agy reset`, `/agy switch`, `/agy help`.
- [x] Add unit tests in `src/main.rs` and `src/core/agent.rs` for command parsing and provider replacement.

---

## 6. Potential Future Enhancements (Post-Milestone 2)

- **Streaming Support**: Server-Sent Events (SSE) token streaming and delta chunk reassembly.
- **Pluggable Persistence**: Save and restore conversation states to SQLite or JSON files.
- **Subprocess Execution**: Sandboxed shell/command execution tools.
- **Multi-Agent Orchestration**: Router / Supervisor agent coordinating specialist sub-agents.
