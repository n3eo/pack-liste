---
description: "Task list for 021-add-dialog-quantity-mode"
---

# Tasks: Add Dialog Quantity and Mode

**Input**: Design documents from `/specs/021-add-dialog-quantity-mode/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included per story.

**Organization**: Composes with 020 (either order); with the amended 018 (no recalculation after add).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Dialog State (US1, US2)

**Goal**: The add dialog carries the Quantity and Mode fields' state.

**Independent Test**: Open the add dialog — the fields exist with the defaults (quantity empty, mode automatic).

- [X] T001 [US2] Add `auto_quantity: bool` to `InputMode::AddingItem` in `src/ui/state.rs` (default `true`, US2/AC1); `start_adding_item` in `src/ui/handlers/dialogs.rs` initializes it
- [X] T002 [US2] Generalize the mode toggle: `toggle_edit_mode` in `src/ui/state.rs` matches both `AddingItem` and `EditingItem` (plan D4); Space on the Mode field toggles in both dialogs

**Checkpoint**: The state exists; the toggle works in both dialogs.

---

## Phase 2: Rendering and Save (US1, US2, US3)

**Goal**: Both dialogs show the same four fields; the add save honors the typed quantity and mode.

**Independent Test**: Add with quantity 5 → the item shows 5 (US1/AC2); add empty → 1 (US1/AC3); add fixed 3 → survives recalculation (US2/AC2); add automatic 3 → recalculates on the next explicit recalculate (US2/AC3, EC-001).

- [X] T003 [US3] Pass the mode through the renderer: the add path in `src/ui/draw.rs` passes `auto_quantity: Some(mode)` to the shared `draw_item_dialog` (plan D2 — one-line; the Mode field renders for both dialogs, US3/AC1)
- [X] T004 [US1] Extend `save_new_item` in `src/ui/handlers/dialogs.rs` with `auto_quantity: bool`: the created item's mode is the dialog's value; quantity = the parsed typed value (empty → 1) (US1/AC2-3, US2/AC2); the dialog handler's save arm passes the dialog's fields
- [X] T005 [US1] Add tests in `src/ui/tests/`: add-quantity-5 → 5; add-empty → 1; add-fixed-3 survives a scoped recalculate; add-automatic-3 recalculates on the next explicit recalculate (EC-001); the parity invariant — both `InputMode` variants carry the same field set (US3, a state test)

**Checkpoint**: The two dialogs are one model with two entry points.

---

## Phase 3: Discovery and Verification

- [X] T006 Update `src/ui/hints.rs`/`src/ui/help.rs` if the add dialog's bindings differ from the documented set (they should not — the shared AddEditPopup surface already documents them); verify the extraction tests in `src/ui/tests/hints.rs` pass unchanged
- [X] T007 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's add-semantics checks pass

---

## Dependencies & Execution Order

- T001 → T002 → T003/T004 (state before rendering/save) → T005.
- **Cross-spec**: with 020 landed, the new fields join its Up/Down cycle automatically (the cycle is generic over `InputField`); with the amended 018, no recalculation follows the add (EC-004 there, EC-001 here).

## Notes

- The 42 existing tests are the contract; the add-flow tests (if any assert always-automatic) are updated per this spec.
- US3's parity is structural: one shared renderer, one shared field cycle.
