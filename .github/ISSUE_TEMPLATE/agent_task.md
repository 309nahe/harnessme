---
name: Agent Task Specification
about: Structured issue for autonomous/coding agent execution
title: '[TASK]: '
labels: ['agent-ready']
assignees: ''
---

## 1. Goal & Context
<!-- Provide a concise description of the objective and why it is needed. -->
- **Objective:** 
- **Context/Background:** 

---

## 2. Relevant Scope & Files
<!-- Pinpoint the exact areas of the codebase to touch. Helps prevent agents from hallucinating or sprawling across unrelated modules. -->
- **Target Files/Directories:**
  - `path/to/file.rs`
- **Reference Implementations / Existing Patterns:**
  - `path/to/similar_pattern.rs`
- **Out of Scope (Do NOT Modify):**
  - `path/to/locked_module.rs`

---

## 3. Implementation Requirements
<!-- Specific design constraints, architectural requirements, or edge cases. -->
- [ ] Requirement 1: 
- [ ] Requirement 2: 
- [ ] Concurrency/Safety/Performance considerations: 

---

## 4. Verification & Definition of Done
<!-- Commands the agent must run locally to self-verify before submitting a PR. -->
The agent must verify that all the following pass cleanly:

```bash
# 1. Build & Lint checks
cargo check --all-targets
cargo clippy -- -D warnings
cargo fmt --check

# 2. Test execution
cargo test
```
