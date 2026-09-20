---
description: "Task list for 014-uniform-arrow-navigation (closure)"
---

# Tasks: Uniform Arrow-Key Navigation Model (Closure)

**Input**: Design documents from `/specs/014-uniform-arrow-navigation/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Verification-only (the plan's surface inventory); no code expected.

**Organization**: Runs after 020 lands (the one known deviation is 020's scope).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Closure Verification

**Goal**: Every surface follows the model; the spec records satisfaction.

**Independent Test**: The plan's surface inventory re-run — every row compliant; the extraction tests pin the binding set.

- [X] T01 Re-run the plan's surface inventory against the running application (`src/ui/handlers/welcome.rs`, `src/ui/handlers/config.rs`, `src/ui/handlers/editor.rs`, `src/ui/screens/`): Up/Down navigate, Left/Right adjust the focused value, on every surface — post-020, the dialog's row must read compliant
- [X] T02 Record the closure: update `specs/014-uniform-arrow-navigation/spec.md`'s status to reflect satisfaction (Implemented via 016 + 020), with the spec's existing alignment note as the record; any surprise deviation found in T001 becomes a task in the owning spec (not here)
- [X] T03 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0 (no code changes expected — the gate confirms the verification pass touched nothing)

---

## Dependencies & Execution Order

- **Cross-spec (critical)**: T001 requires 020 to have landed (the dialog's Up/Down field cycling is the one deviation). Run this closure after 020's implementation.

## Notes

- This is a bookkeeping spec: the model is implemented by 016; the closure verifies and records. No code changes are in scope here.
