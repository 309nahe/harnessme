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
- [ ] Create a minimal `main.rs` binary.
- [ ] Read API configuration (API key, base URL, model name) from environment variables (e.g., `OPENAI_API_KEY`, `OPENAI_BASE_URL`).
- [ ] Provide an interactive REPL in the terminal to converse with the agent and verify autonomous tool usage.
- [ ] Validate edge cases: invalid tool arguments, tool failures, model hallucinated tool names.

---

## 5. Potential Future Enhancements (Post-MVP)

- **Streaming Support**: Stream token generation and tool call chunks.
- **Context Pruning**: Sliding window or summarization strategy when conversation approaches token limits.
- **Pluggable Persistence**: Save and restore conversation states across sessions.
- **Sandboxed Execution**: Safe execution boundaries for command-line or filesystem tools.
