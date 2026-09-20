---
description: "Task list for 020-edit-dialog-navigation"
---

# Tasks: Edit Dialog Navigation

**Input**: Design documents from `/specs/020-edit-dialog-navigation/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included per story (US1–US3's scenarios are state-testable).

**Organization**: Composes with 021 (either order); the field cycle is generic so 010's fields join later.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Field Cycling (US1)

**Goal**: Up/Down cycle the dialog's fields (wrapping); the Category field keeps the dropdown behavior.

**Independent Test**: Down ×4 from Name wraps to Name; Up reverses; in Category, Up/Down move the dropdown (EC-001).

- [X] T001 [US1] Add `InputField::next()`/`prev()` (the cycle Name → Category → Quantity → Mode → Name, wrapping) in `src/ui/state.rs` — the single place later fields (010's Rate/Buffer) extend
- [X] T002 [US1] Rewire the dialog handler in `src/ui/handlers/welcome.rs`: Up/Down cycle fields via `next`/`prev` EXCEPT in the Category field, where they keep the dropdown-highlight behavior (EC-001); Tab remains the alias everywhere (US1/AC4)

**Checkpoint**: The dialog follows the uniform model.

---

## Phase 2: Quantity Stepping (US3)

**Goal**: Left/Right step the quantity (clamped at 1); typing still appends.

**Independent Test**: Quantity 3: Right → 4, Left → 2, Left at 1 → 1; a typed digit appends (US3/AC1-4).

- [X] T003 [US3] Add quantity stepping to the dialog handler in `src/ui/handlers/welcome.rs`: Left/Right in the Quantity field parse (empty → 1), ±1, clamp at 1, write back; in the Category field Left/Right keep the category cycling (US3/AC5)
- [X] T004 [US3] Add state tests in `src/ui/tests/input.rs`: stepping (3→4→3, clamp at 1), typing appends after stepping, the category field's Left/Right unchanged

**Checkpoint**: Horizontal = adjust the value, everywhere.

---

## Phase 3: Text Cursor (US2)

**Goal**: The focused free-text field shows a cursor at the insertion position.

**Independent Test**: Name focused with "Shirt" → a cursor after "Shirt"; Backspace removes and the cursor retreats; Category/Mode focused → no cursor (US2/AC1-4).

- [X] T005 [US2] Render the text cursor in `draw_item_dialog` (`src/ui/draw.rs`): a highlighted cell immediately after the focused field's text (Name, Quantity only — plan D2: the position is derived from the text length, no new state; the raw editor's cursor-cell pattern)
- [X] T006 [US2] Verify the cursor behavior in `src/ui/draw.rs` end-to-end (the quickstart's PTY check): visible in Name/Quantity when focused, absent in Category/Mode, advances on type, retreats on Backspace

**Checkpoint**: Typing is visually grounded.

---

## Phase 4: Discovery and Verification (US4)

- [X] T007 [US4] Update the AddEditPopup hint line in `src/ui/hints.rs` and the help screen's dialog section in `src/ui/help.rs` with the final binding set (Up/Down cycle, Left/Right adjust, Tab alias); extend the extraction tests in `src/ui/tests/hints.rs`
- [X] T008 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's cycling/stepping/cursor checks pass

---

## Dependencies & Execution Order

- T001 → T002 → T003 (the cycle exists before the handler uses it); T005 independent of T003 (different concerns, same renderer — sequence to avoid conflicts); T007 after T002/T003.
- **Cross-spec**: with 021 landed, the cycle covers both dialogs' shared field set; 010's Rate/Buffer join via `InputField::next`/`prev` (T001's design).

## Notes

- The 42 existing tests are the contract; the dialog-input tests (`dialog_category_dropdown_moves_within_bounds` etc.) must keep passing — the Category field's Up/Down behavior is preserved, not replaced.
