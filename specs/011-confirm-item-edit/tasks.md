---
description: "Task list for 011-confirm-item-edit"
---

# Tasks: Confirm Before Saving Item Edits

**Input**: Design documents from `/specs/011-confirm-item-edit/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included per story.

**Organization**: Lands after 020/021 (the dialog's field set is final first).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: The Confirmation Mode (US1)

**Goal**: The edit dialog's save routes to a confirmation; confirm applies, cancel discards.

**Independent Test**: Edit, save the dialog, decline → the item and the config file are unchanged; accept → applied and written (US1/AC2-3).

- [X] T001 [US1] Add `PendingEdit { index, name, categories, quantity, auto_quantity }` (plus rate/buffer once 010 lands) and `InputMode::ConfirmEdit { pending: PendingEdit }` in `src/ui/state.rs` (plan D1's pattern: the dialog state is consumed into the snapshot)
- [X] T002 [US1] Route the edit dialog's save arm in `src/ui/handlers/welcome.rs` to `ConfirmEdit` instead of applying directly; Enter confirms (apply via `save_edited_item` + save), Esc/N cancels (drop the snapshot, return to Normal, nothing written — plan D3); Esc layering per plan D4 (one Esc from the prompt back to the list)

**Checkpoint**: The write is an explicit, deliberate act.

---

## Phase 2: The Prompt (US2)

**Goal**: The confirmation shows the pending change.

**Independent Test**: Edit name + quantity, save → the prompt shows both new values (US2/AC1).

- [X] T003 [US2] Add `draw_confirm_edit_popup` in `src/ui/draw.rs` (the delete popup's pattern: centered, `min()`-sized for small terminals, US2/AC3): the item's name and the new values (name, categories, quantity, mode; old values where space allows)
- [X] T004 [US2] Wire the renderer: `App::draw`'s `ConfirmEdit` arm renders the popup

**Checkpoint**: The confirmation is meaningful, not blind.

---

## Phase 3: Safety and Tests (US3)

**Goal**: Cancel at any stage loses nothing.

**Independent Test**: Cancel at the prompt → the item and the config file are byte-identical to before the dialog opened (US3/AC1).

- [X] T005 [US3] Add tests in `src/ui/tests/`: confirm applies + saves; cancel discards (item unchanged, no file write — US3/AC1); the snapshot carries the dialog's values; the add flow is NOT confirmed (scope note)
- [X] T006 [US3] Update `src/ui/hints.rs`/`src/ui/help.rs`: the confirmation's bindings (Enter/Esc, the delete popup's set); extend the extraction tests in `src/ui/tests/hints.rs`
- [X] T007 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's confirm/cancel/diff-the-file checks pass

---

## Dependencies & Execution Order

- T001 → T002 → T003/T004 → T005 → T006.
- **Cross-spec**: after 020/021 (the prompt shows the final field values); the snapshot struct carries 010's rate/buffer when 010 lands (extend `PendingEdit`).

## Notes

- The 42 existing tests are the contract; the edit-save tests are updated to route through the confirmation.
- The delete confirmation's flow and tests are untouched — this spec mirrors it, not merges with it.
