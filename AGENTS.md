# AGENTS.md: AI Agent Operational Manual & System Guide

> **Notice for AI Agents & Contributors**: This document is the primary operational manual, architecture guide, and behavioral rulebook for autonomous and coding agents working on the HarnessMe repository.

---

## Table of Contents

1. [Role of This Document vs. DOCUMENTATION.md](#1-role-of-this-document-vs-documentationmd)
2. [Agent Operational Directives & Rules](#2-agent-operational-directives--rules)
3. [Architecture Blueprint & Deep Rationale](#3-architecture-blueprint--deep-rationale)
4. [Component Walkthrough](#4-component-walkthrough)
5. [Implementation Roadmap Sync (`PLAN.md`)](#5-implementation-roadmap-sync-planmd)
6. [Operational Command Reference](#6-operational-command-reference)
7. [GitHub Operations via MCP Protocol (Mandatory)](#7-github-operations-via-mcp-protocol-mandatory)

---

## 1. Role of This Document vs. DOCUMENTATION.md

### 1.1 `AGENTS.md`: The Agent System Manual & Instructions
- Provides AI agents with instructions on how to interact with this repository.
- Outlines architectural philosophies, design trade-offs, coding rules, and safety boundaries.

### 1.2 `DOCUMENTATION.md`: Technical Docs & Official Living Changelog
- **Primary Source of Truth**: [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md) contains the complete technical reference and the **Living Changelog & Evolution Ledger** (Section 13).
- **Mandatory Agent Rule**: Whenever you implement a feature, refactor, fix a bug, or complete an issue, you **must append an entry directly into [DOCUMENTATION.md Section 13](file:///home/nana/dev/harness/DOCUMENTATION.md#13-living-changelog--evolution-ledger)** and update the respective technical sections in [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md).

---

## 2. Agent Operational Directives & Rules

All AI agents operating on this repository must strictly adhere to the following rules:

### Rule 1: Zero Dependency Bloat
- Keep external crates strictly minimal. Core runtime relies only on:
  - `serde`, `serde_json` (for message and schema serialization).
  - A lean HTTP client (`ureq` for synchronous or lightweight `reqwest` if async).
- Do **NOT** pull in large frameworks, heavy async runtimes (unless explicitly approved), or unneeded utility crates. Use Rust's standard library (`std::sync`, `std::collections`, `std::time`, `std::io`) wherever possible.

### Rule 2: Trait-First Decoupling
- Never bind the Agent runner directly to concrete provider clients or specific tool implementations.
- Always program against the `Provider` and `Tool` traits located in `src/core/`.

### Rule 3: Error Handling & Safety
- **No `unwrap()` or `expect()` in production pathways.**
- All fallible operations must return explicit `Result<T, E>` with domain-specific error enums (`ToolError`, `ProviderError`, `AgentError`).
- All tool execution failures must be safely caught and mapped back into conversation context as `Role::Tool` messages so the LLM can self-correct.

### Rule 4: Mandatory Changelog & Documentation Updates
- Every task, feature, or bugfix must be documented in:
  1. [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md) (Update relevant technical sections and append to Section 13: Living Changelog).
  2. [PLAN.md](file:///home/nana/dev/harness/PLAN.md) (Check off completed milestones).

### Rule 5: Exclusively Use MCP for GitHub Operations (Zero CLI `git`/`gh`)
- **NEVER** use CLI commands like `git push`, `git commit`, `git add`, `gh issue`, or `gh auth` for remote operations.
- **ALWAYS and ONLY** use `github-mcp-server` tools via `call_mcp_tool` for all remote repository interactions:
  - **Pushing / Committing Files**: Use `push_files` or `create_or_update_file` (with blob SHA obtained via `get_file_contents`).
  - **Inspecting Remote State**: Use `get_file_contents`, `list_branches`, `list_commits`, `get_commit`.
  - **Issues & Tracking**: Use `issue_write`, `add_issue_comment`, `list_issues`, `issue_read`.
- Local shell execution is strictly reserved for local Rust builds and testing (`cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`).

---

## 3. Architecture Blueprint & Deep Rationale

### 3.1 Why Rust?
Rust provides memory safety without garbage collection, zero-cost abstractions, first-class concurrency primitives, and deterministic resource cleanup. For an AI agent harness, Rust ensures ultra-low overhead, minimal latency during tool dispatch, and high resilience during long-running tasks.

### 3.2 Synchronous Core vs. Async Extensibility
- **Decision**: Start with a synchronous core using blocking I/O (`ureq`).
- **Rationale**: An agent execution loop is intrinsically sequential (LLM Think $\rightarrow$ Tool Call $\rightarrow$ Tool Result $\rightarrow$ LLM Think). A synchronous architecture eliminates the cognitive and runtime overhead of async executors, runtime pinning, and future cancellation issues for single-agent loops.
- **Future path**: An async provider adapter can be layered in when parallel tool dispatch or web server integration is needed.

### 3.3 Universal Message Protocol
The harness mirrors the standard OpenAI tool-calling protocol as the universal intermediate representation (IR), supported by OpenAI, Ollama, vLLM, Groq, Anthropic proxies, and LocalAI.

```
   ┌─────────────────────────────────────────────────────────┐
   │                  Message IR (types.rs)                  │
   ├─────────────────────────────────────────────────────────┤
   │ Role: System | User | Assistant | Tool                  │
   │ Content: Option<String>                                 │
   │ ToolCalls: Option<Vec<ToolCall>>                        │
   │ ToolCallId: Option<String>                              │
   └─────────────────────────────────────────────────────────┘
```

---

## 4. Component Walkthrough

### 4.1 `core::types`
Defines the message envelope and tool structures (`Role`, `Message`, `ToolCall`, `ToolDefinition`, `ToolResult`).

### 4.2 `core::tool`
- Defines `trait Tool`: Requires `name()`, `description()`, `parameters_schema()`, and `execute()`.
- Implements `ToolRegistry`: Manages tool registration, dynamic schema generation, and argument parsing.

### 4.3 `core::provider`
- Defines `trait Provider`: Abstracts LLM inference endpoints.
- Implements `OpenAiCompatibleProvider`: HTTP transport for OpenAI-compatible REST endpoints.

### 4.4 `core::agent`
- `AgentConfig`: Controls model parameters, temperature, system prompt, and iteration limits (`max_iterations`).
- `Agent`: Manages history state, loops through turns, executes tools, and terminates on final answers or limit exhaustion.

---

## 5. Implementation Roadmap Sync (`PLAN.md`)

| Phase | Description | Status | Reference |
|---|---|---|---|
| **Phase 1** | Project setup & core domain types (`Role`, `Message`, `ToolCall`, `ToolResult`) | Completed | [PLAN.md:L80-84](file:///home/nana/dev/harness/PLAN.md#L80-L84) |
| **Phase 2** | Tool abstraction & in-memory `ToolRegistry` with sample tools | Completed | [PLAN.md:L85-90](file:///home/nana/dev/harness/PLAN.md#L85-L90) |
| **Phase 3** | Provider abstraction & OpenAI-compatible client | Completed | [PLAN.md:L91-96](file:///home/nana/dev/harness/PLAN.md#L91-L96) |
| **Phase 4** | The Agent execution loop & guardrails | Completed | [PLAN.md:L97-107](file:///home/nana/dev/harness/PLAN.md#L97-L107) |
| **Phase 5** | CLI demo, environment parsing & interactive REPL | Completed | [PLAN.md:L108-114](file:///home/nana/dev/harness/PLAN.md#L108-L114) |

---

## 6. Operational Command Reference

> [!IMPORTANT]
> **Local Shell vs. GitHub Operations**: Terminal commands (`run_command`) are permitted **ONLY** for local Rust compilation, linting, formatting, and test execution. **NEVER** run `git push`, `git commit`, `git add`, or `gh` commands via terminal.

### Build & Check
```bash
cargo check --all-targets
cargo clippy -- -D warnings
cargo fmt --check
```

### Run Tests
```bash
cargo test
```

### Run CLI Interactive REPL
```bash
OPENAI_API_KEY="your-api-key" cargo run
```

---

## 7. GitHub Operations via MCP Protocol (Mandatory)

All remote GitHub operations (pushing code, updating files, managing branches, commenting on issues, closing issues) **MUST be performed exclusively through the `github-mcp-server` MCP tools** via `call_mcp_tool`.

### MCP Tool Usage Guide

| Task | MCP Tool (`ServerName: "github-mcp-server"`) | Parameters / Strategy |
|---|---|---|
| **Push multiple files / Commit** | `push_files` | `owner: "309nahe"`, `repo: "harnessme"`, `branch: "dev"`, `message: "feat: ..."` , `files: [{ path, content }]` |
| **Create or update single file** | `create_or_update_file` | Retrieve current blob SHA with `get_file_contents` first, then pass `sha`, `path`, `content`, `branch: "dev"`, `message: "..."` |
| **Read remote file content & SHA** | `get_file_contents` | `owner: "309nahe"`, `repo: "harnessme"`, `path: "...", ref: "dev"` |
| **List remote branches** | `list_branches` | `owner: "309nahe"`, `repo: "harnessme"` |
| **List & inspect issues** | `list_issues` / `issue_read` | `owner: "309nahe"`, `repo: "harnessme"`, `state: "open"` |
| **Add issue comments** | `add_issue_comment` | `owner: "309nahe"`, `repo: "harnessme"`, `issue_number: N`, `body: "..."` |
| **Update / Close issues** | `issue_write` | `owner: "309nahe"`, `repo: "harnessme"`, `method: "update"`, `issue_number: N`, `state: "closed"`, `state_reason: "completed"` |
