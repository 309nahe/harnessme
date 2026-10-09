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

### Rule 5: Document Produced Code Properly
- All produced code must be properly documented at the source level:
  - Every public item (struct, enum, trait, function, method, module) must carry a `///` doc comment explaining its purpose, invariants, and usage.
  - Modules must have a `//!` header comment describing their responsibility within the architecture.
  - Documentation must describe behavior, not implementation trivia, and must be kept in sync when the code changes.

### Rule 6: Unit Testing & Regression Prevention
- All produced code must ship with unit tests to prevent regressions:
  - Every new module or public API must include `#[cfg(test)]` unit tests covering nominal cases and key edge cases (empty input, malformed data, boundary values).
  - Serialization-bound types must include round-trip tests (`struct -> JSON -> struct`) to guarantee wire-format stability.
  - `cargo test` must pass before any change is considered complete; a change that breaks an existing test must fix the test or justify the behavioral change in the changelog entry.
  - Never delete a failing test to make a suite pass; fix the underlying defect or document the intentional behavior change.

### Rule 7: GitHub Operations via `gh`
- **Always use the `gh` CLI** for any GitHub interaction (issues, pull requests, releases, comments, labels, repo metadata), never the REST/GraphQL API through ad-hoc HTTP calls or third-party connectors.
- Use `gh issue list --repo 309nahe/harnessme --state open` to inspect open work, and `gh issue view <number> --repo 309nahe/harnessme` before implementing an issue so the acceptance criteria are known.
- Close issues with `gh issue close <number> --repo 309nahe/harnessme` only after the work is verified (tests pass) and this changelog is updated.

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
| **Phase 1** | Project setup & core domain types (`Role`, `Message`, `ToolCall`, `ToolResult`) | Complete (Issues #15, #16, #17 closed) | [PLAN.md:L80-84](file:///home/nana/dev/harness/PLAN.md#L80-L84) |
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
  - Begin Phase 1 implementation (`Cargo.toml` and `src/core/types.rs`) on `dev` branch.

---

### [2026-10-09] - Mistral Vibe (Phase 1: Issues #15 & #16)
- **Objective**: Add mandatory documentation and unit-testing directives to this manual, then implement GitHub issues #15 (Cargo project initialization) and #16 (core domain types).
- **Changes Made**:
  - `AGENTS.md`: Added Rule 5 (Document Produced Code Properly) and Rule 6 (Unit Testing & Regression Prevention); updated Phase 1 roadmap status.
  - `Cargo.toml` (new): Initialized with minimal dependencies only (`serde` with `derive`, `serde_json`, `ureq`), per Rule 1.
  - `src/lib.rs` (new): Library root exposing `pub mod core`.
  - `src/core/mod.rs` (new): Core module root exposing `types`.
  - `src/core/types.rs` (new): `Role`, `Message`, `ToolCall`, `FunctionCall`, `ToolDefinition`, `FunctionDefinition`, `ToolResult`, plus `Message` role constructors.
  - `src/main.rs` (new): Minimal CLI placeholder until Phase 5.
  - `DOCUMENTATION.md`: Added `ToolResult` specification and `Message` constructor reference to Section 3.
  - `PLAN.md`: Checked off Phase 1 milestones for issues #15 and #16.
- **Architectural Decisions**:
  - `ToolDefinition.kind` uses `#[serde(rename = "type")]` (instead of `r#type`) for a raw identifier-free API surface while keeping the OpenAI wire format.
  - `FunctionCall::arguments` stays a raw JSON string, parsed lazily at tool-execution time, preserving the exact model output.
  - Optional `Message` fields use `skip_serializing_if` to keep wire payloads clean.
  - `ToolResult::content` carries either output or an error message, so failures feed back to the LLM as `Role::Tool` messages per Rule 3.
- **Verification**:
  - `cargo test`: 8 unit tests passed (role wire-format, message/ToolCall/ToolResult round-trips, OpenAI tool-definition schema shape, constructor roles).
  - Note: build required `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk` because the default `MacOSX27.0.sdk` tbd files are rejected by the installed Rust 1.89 linker (tapi "malformed file" error).
- **Next Steps**:
  - Issue #17: Add serde round-trip tests for tool definitions and messages (basic coverage exists; extend with edge cases and regression tests).
  - Issues #18-19 (Phase 2): `Tool` trait, `ToolError`, and in-memory `ToolRegistry`.
  - Consider documenting the SDKROOT workaround for macOS agents in this manual.

---

### [2026-10-09] - Mistral Vibe (macOS toolchain fix: tapi "malformed file" linker error)
- **Objective**: Diagnose and fix why no Rust crate could link on this MacBook (`ld: tapi error: malformed file ... unknown architecture arm64e.x1`).
- **Root Cause**:
  - `xcode-select` pointed at `/Applications/Xcode.app` (26.3), whose MacOS platform SDKs are missing entirely (`Platforms/MacOS.platform/Developer/SDKs/` does not exist), so clang fell back to the CommandLineTools `MacOSX27.0.sdk`.
  - The 27.0 SDK's `.tbd` files use architectures (e.g. `arm64e.x1`) that Xcode 26.3's older `ld`/libtapi cannot parse, while the matching CommandLineTools toolchain (CLT 27.0, clang 21) understands them fine.
  - Rust 1.89 itself was never at fault; the same failure occurs with any rustc invoking `cc` against that SDK/toolchain mismatch.
- **Changes Made**:
  - Ran `sudo xcode-select -s /Library/Developer/CommandLineTools` so `cc`/`ld` and the default SDK now come from the self-consistent CLT 27.0 toolchain.
  - No repository files modified; the previous `SDKROOT=...MacOSX26.5.sdk` workaround is no longer needed and can be removed from any shell profile or CI env.
- **Verification**:
  - `cargo clean && cargo build`: succeeds with no environment overrides.
  - `cargo test`: 8 passed, 0 failed.
  - Standalone `rustc` hello-world outside the repo: compiles and runs.
- **Next Steps**:
  - If full Xcode is needed later, reinstall/repair Xcode (its MacOS platform is currently broken) and switch back with `sudo xcode-select -s /Applications/Xcode.app`.
  - Optionally `rustup update stable` (1.89 -> 1.99 available) for general hygiene; not required for this fix.

---

### [2026-10-09] - Mistral Vibe (Add Rule 7: GitHub operations via `gh`)
- **Objective**: Mandate the `gh` CLI as the sole interface for GitHub operations in this repository, and list all open issues to establish the remaining roadmap.
- **Changes Made**:
  - `AGENTS.md`: Added Rule 7 (GitHub Operations via `gh`) to Section 2.
- **Architectural Decisions**:
  - `gh` is authenticated, scoped, and consistent across agents; ad-hoc API calls or connector tools risk auth drift and inconsistent repo targeting.
  - Canonical repository is `309nahe/harnessme` (origin remote).
- **Verification**:
  - `gh issue list --repo 309nahe/harnessme --state open` executed successfully; 5 open issues found (#15-#19).
  - Noted discrepancy: issues #15 and #16 are still OPEN on GitHub despite being implemented and logged in the changelog (2026-10-09 entry); they still need to be closed via `gh issue close`.
- **Next Steps**:
  - Close issues #15 and #16 via `gh` once confirmed complete.
  - Issue #17: Extend serde round-trip tests.
  - Issues #18-19 (Phase 2): `Tool` trait, `ToolError`, and in-memory `ToolRegistry`.

---

### [2026-10-09] - Mistral Vibe (Issue #17: serde round-trip tests; dev branch reset)
- **Objective**: Complete issue #17 (serialization/deserialization tests for messages and tool definitions), and re-establish the `dev` branch from the current local Phase 1 baseline.
- **Changes Made**:
  - `AGENTS.md`: Added Rule 7 (GitHub operations via `gh`); recorded that issues #15/#16 were closed via `gh`; reset roadmap Phase 1 to Complete.
  - Deleted the stale remote `dev` branch (an unrelated, far-ahead history from a previous iteration) and recreated it from the current local baseline; old tip archived locally as `archive/old-dev` (660f080).
  - Committed the Phase 1 work (issues #15, #16) on `dev` and pushed it as the new remote baseline.
  - `src/core/types.rs`: Added `PartialEq`/`Eq` derives to all data structs (no wire-format impact; enables whole-value round-trip assertions). Added 10 new tests: all-roles JSON-string round-trip, assistant message with content + tool calls, assistant message omitting both optional fields, multiple tool calls ordering, standalone `ToolCall` exact wire fields, canonical OpenAI `ToolDefinition` JSON parse/re-serialize, empty-schema `ToolDefinition` round-trip, malformed-JSON rejection, empty/unicode payloads, full conversation-history round-trip.
  - `PLAN.md`: Checked off Phase 1 item 3 (issue #17).
  - `DOCUMENTATION.md`: Expanded Section 10 (Testing & Verification Strategy) with the issue #17 coverage inventory.
- **Architectural Decisions**:
  - Round-trip tests compare whole values via `assert_eq!` after adding `PartialEq`/`Eq` derives, rather than field-by-field checks, to keep failures readable and exhaustive.
  - Tests assert exact JSON field names against the OpenAI wire format (including `type` rename on `ToolDefinition.kind` and raw-string preservation of `FunctionCall::arguments`).
  - `dev` is now a clean continuation of `main` plus Phase 1; the old remote history is preserved locally in `archive/old-dev` for reference and can be deleted at will.
- **Verification**:
  - `cargo test`: 18 passed, 0 failed (8 pre-existing + 10 new).
  - `gh issue list --state open`: #15, #16 closed via `gh issue close` with verification comments.
- **Next Steps**:
  - Close issue #17 via `gh issue close` once this change is pushed.
  - Issues #18-19 (Phase 2): `Tool` trait, `ToolError`, and in-memory `ToolRegistry`.
  - Delete `archive/old-dev` once the old history is confirmed unneeded.

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
