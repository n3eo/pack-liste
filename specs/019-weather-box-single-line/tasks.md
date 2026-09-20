---
description: "Task list for 019-weather-box-single-line"
---

# Tasks: Weather Box Single Line

**Input**: Design documents from `/specs/019-weather-box-single-line/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Visual verification (the quickstart's multi-size check); no state to test.

**Organization**: Independent; one constraint change.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: The Fix (US1)

**Goal**: The weather box matches the other fields' height at every size.

**Independent Test**: The welcome screen at 80x24, 120x40, and a tall size — the weather box's height equals the other three fields' at every size; no field touches the bottom or the hint footer (US1/AC1).

- [X] T001 [US1] Change the fourth constraint in `draw_welcome_screen`'s `params_layout` in `src/ui/screens/welcome.rs` from `Constraint::Min(3)` to `Constraint::Length(3)` (plan D1 — the activities field's old flexible role, left stretched since spec 017)

**Checkpoint**: The four fields are uniform compact blocks.

---

## Phase 2: Verification

- [X] T002 Verify `src/ui/screens/welcome.rs` at multiple sizes (the quickstart's PTY size checks): 80x24, 120x40, tall — the weather box matches the other fields; the hint footer fully visible; the resume prompt unaffected (plan D2)
- [X] T003 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0

---

## Dependencies & Execution Order

- T001 → T002 → T003. No cross-spec dependencies.

## Notes

- The 42 existing tests are the contract and are untouched by a layout constraint change.
