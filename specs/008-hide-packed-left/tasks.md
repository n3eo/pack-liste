---
description: "Task list for 008-hide-packed-left"
---

# Tasks: No Packed State on the Left Panel

**Input**: Design documents from `/specs/008-hide-packed-left/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included (the spec's independent test is a render-identity assertion).

**Organization**: Independent; lands in any order.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Remove the Indicator (US1)

**Goal**: The left panel shows no packed state; the right panel is untouched.

**Independent Test**: Pack and unpack items on the right; the left panel's rendering of every item is identical in both cases (US1/AC1-3).

- [X] T001 [US1] Remove the packed indicator from `draw_all_items_panel`'s item row in `src/ui/screens/main.rs` (plan D1: the `item_symbol` computation and its two spans, lines ~146-165): the row becomes selection symbol + name + quantity; `draw_selected_items_panel` (the right panel) is untouched (US2, plan D2)
- [X] T002 [US1] Add the render-identity test in `src/ui/tests/`: an item's left-panel row content is identical packed and unpacked (the spec's independent test, as a state-level assertion on the row construction)

**Checkpoint**: Packed state lives only where it is managed.

---

## Phase 2: Verification (US2)

- [X] T003 [US2] Verify `src/ui/screens/main.rs`'s `draw_selected_items_panel` is unchanged: its per-item indicators, category all-packed markers, and progress summary update exactly as before (US2/AC1) — the existing packed-toggle tests in `src/ui/tests/state.rs` pass unmodified
- [X] T004 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's pack/unpack identity check passes

---

## Dependencies & Execution Order

- T001 → T002 → T003 → T004. No cross-spec dependencies.

## Notes

- The 42 existing tests are the contract; `packed_toggles_only_on_right_side` and the packed-derivation tests must pass unmodified (they test the right panel and the state, not the left row's spans).
- Display only: `Item.packed` and its persistence are untouched.
