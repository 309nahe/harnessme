# AGENTS.md: AI Agent System Manual & Living Changelog

> **Notice for AI Agents & Developers**: This document is the primary operational manual, architecture guide, verbose system handbook, and **mandatory living changelog** for the HarnessMe repository. Every agent interacting with this codebase must read, follow, and update this file.

---

## Table of Contents

1. [How to Use This Document](#1-how-to-use-this-document)
2. [Agent Operational Directives & Rules](#2-agent-operational-directives--rules)
3. [Architecture Blueprint & Deep Rationale](#3-architecture-blueprint--deep-rationale)
4. [Component Walkthrough](#4-component-walkthrough)
5. [Implementation Roadmap Sync (`PLAN.md`)](#5-implementation-roadmap-sync-planmd)
6. [Living Changelog & Evolution Ledger](#6-living-changelog--evolution-ledger)
7. [Operational Command Reference](#7-operational-command-reference)

---

## 1. How to Use This Document

This document serves multiple critical roles across the lifecycle of the HarnessMe project:

### 1.1 As a More Verbose README & System Handbook
While standard READMEs provide quick summaries for end users, `AGENTS.md` provides an exhaustive, low-level technical reference. It documents:
- The architectural philosophy and trade-offs behind every design decision.
- Module boundaries, type contracts, and runtime lifecycle details.
- Contextual information needed by an AI agent or engineer to immediately become productive in the codebase.

### 1.2 As a Living Changelog & Context Preservation Ledger
To avoid context drift between different AI sessions or contributors:
- **Mandatory Logging**: Every time an agent or developer implements a feature, refactors a module, fixes a bug, or completes a phase from `PLAN.md`, they **must append an entry to [Section 6: Living Changelog & Evolution Ledger](#6-living-changelog--evolution-ledger)**.
- Each entry must include the date, author/agent model, summary of changes, rationale/decisions, files modified, and next steps.

### 1.3 As the Primary Documentation Source
- Whenever you make changes to core data types, traits, configuration, or tools, you must update both [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md) and `AGENTS.md`.
- Ensure that everything is documented: public APIs, trait methods, error types, environment variables, and testing procedures.

---

## 2. Agent Operational Directives & Rules

All AI agents operating on this repository must adhere to the following non-negotiable rules:

### Rule 1: Zero Dependency Bloat
- Keep external crates strictly minimal. Core runtime relies only on:
  - `serde`, `serde_json` (for message and schema serialization).
  - A lean HTTP client (`ureq` for synchronous or lightweight `reqwest` if async).
- Do **NOT** pull in large frameworks, heavy async runtimes (unless explicitly approved for async scaling), or unneeded utility crates. Use Rust's standard library (`std::sync`, `std::collections`, `std::time`, `std::io`) wherever possible.

### Rule 2: Trait-First Decoupling
- Never bind the Agent runner directly to concrete provider clients or specific tool implementations.
- Always program against the `Provider` and `Tool` traits located in `src/core/`.

### Rule 3: Error Handling & Safety
- **No `unwrap()` or `expect()` in production pathways.**
- All fallible operations must return explicit `Result<T, E>` with domain-specific error enums (`ToolError`, `ProviderError`, `AgentError`).
- All tool execution failures must be safely caught and mapped back into conversation context as `Role::Tool` messages so the LLM can self-correct.

### Rule 4: Documentation Synchronization
- Any code addition or API modification must be accompanied by updates in:
  1. [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md) (Technical specifications & user guides).
  2. [AGENTS.md](file:///home/nana/dev/harness/AGENTS.md) (Living changelog & architectural notes).
  3. [PLAN.md](file:///home/nana/dev/harness/PLAN.md) (Check off completed milestones).

---

## 3. Architecture Blueprint & Deep Rationale

### 3.1 Why Rust?
Rust provides memory safety without garbage collection, zero-cost abstractions, first-class concurrency primitives, and deterministic resource cleanup. For an AI agent harness, Rust ensures ultra-low overhead, minimal latency during tool dispatch, and high resilience during long-running tasks.

### 3.2 Synchronous Core vs. Async Extensibility
- **Decision**: Start with a synchronous core using blocking I/O (`ureq`).
- **Rationale**: An agent execution loop is intrinsically sequential (LLM Think $\rightarrow$ Tool Call $\rightarrow$ Tool Result $\rightarrow$ LLM Think). A synchronous architecture eliminates the cognitive and runtime overhead of async executors, runtime pinning, and future cancellation issues for single-agent loops.
- **Future path**: An async provider adapter can be layered in when parallel tool dispatch or web server integration is needed.

### 3.3 Universal Message Protocol
The harness mirrors the standard OpenAI tool-calling protocol as the universal intermediate representation (IR), as it is supported by almost all modern inference engines (OpenAI, Ollama, vLLM, Groq, Anthropic proxies).

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
Defines the message envelope and tool structures. All structs derive `Serialize` and `Deserialize` using `serde`.

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
| **Phase 2** | Tool abstraction & in-memory `ToolRegistry` with sample tools | Planned | [PLAN.md:L85-90](file:///home/nana/dev/harness/PLAN.md#L85-L90) |
| **Phase 3** | Provider abstraction & OpenAI-compatible client | Planned | [PLAN.md:L91-96](file:///home/nana/dev/harness/PLAN.md#L91-L96) |
| **Phase 4** | The Agent execution loop & guardrails | Planned | [PLAN.md:L97-107](file:///home/nana/dev/harness/PLAN.md#L97-L107) |
| **Phase 5** | CLI demo, environment parsing & interactive REPL | Planned | [PLAN.md:L108-114](file:///home/nana/dev/harness/PLAN.md#L108-L114) |

---

## 6. Living Changelog & Evolution Ledger

> **Agent Instruction**: Whenever you modify the codebase, append a new subsection below under this section using the standard template.

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
- **Objective**: Establish the repository foundation, generate `DOCUMENTATION.md`, create `AGENTS.md` as a verbose handbook and living changelog, and prepare GitHub repository `harnessme`.
- **Changes Made**:
  - Analyzed [PLAN.md](file:///home/nana/dev/harness/PLAN.md) specifications for the minimal Rust agent harness.
  - Created [DOCUMENTATION.md](file:///home/nana/dev/harness/DOCUMENTATION.md): Complete technical reference detailing system architecture, core domain models, tool subsystem, provider interface, agent execution loop, configuration, and testing strategy.
  - Created [AGENTS.md](file:///home/nana/dev/harness/AGENTS.md): Established agent operating directives, documentation protocols, architectural deep dives, and living changelog.
- **Architectural Decisions**:
  - Standardized on zero-bloat dependencies (`serde`, `serde_json`, `ureq`).
  - Formalized `AGENTS.md` as both a verbose README and the mandatory append-only session changelog.
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

## 7. Operational Command Reference

### Build & Check
```bash
cargo check
cargo build
```

### Run Tests
```bash
cargo test
```

### Run CLI Interactive REPL
```bash
OPENAI_API_KEY="your-api-key" cargo run
```
