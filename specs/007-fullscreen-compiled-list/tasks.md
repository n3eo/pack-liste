---
description: "Task list for 007-fullscreen-compiled-list"
---

# Tasks: Fullscreen Compiled List View

**Input**: Design documents from `/specs/007-fullscreen-compiled-list/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included per story.

**Organization**: Independent of the data-model features; composes with 006 (either order).

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: The Toggle (US1)

**Goal**: A dedicated key switches between the two-panel view and a fullscreen compiled list.

**Independent Test**: With items selected, press the key → only the compiled list, full width; press again → the two-panel view with the same selections, packed states, and cursor (US1/AC1-2).

- [X] T001 [US1] Add `fullscreen: bool` (default false, not persisted) to `AppDrawState` in `src/ui/state.rs`
- [X] T002 [US1] Branch `draw_main_screen` in `src/ui/screens/main.rs`: fullscreen renders only `draw_selected_items_panel` at the full main-screen area (plan D1 — the renderer is area-parameterized; zero renderer changes)
- [X] T003 [US1] Wire the toggle key (`F`, plan D2's candidate) in the main-screen handler in `src/ui/handlers/welcome.rs`: flips the flag both directions; Tab is a no-op in fullscreen (plan D3)

**Checkpoint**: The view toggles; the data never changes.

---

## Phase 2: Working in Fullscreen (US2)

**Goal**: All compiled-list actions work in fullscreen.

**Independent Test**: Navigate, toggle packed, recalculate in fullscreen; the changes are reflected after returning (US2/AC1-3).

- [X] T004 [US2] Verify the actions in `src/ui/handlers/welcome.rs`: Up/Down navigate, Space toggles packed, the recalculate keys work (post-018: all three scopes) — the handler arms operate on the same state the panel renders; add a state test in `src/ui/tests/` that the flag does not affect any action's outcome
- [X] T005 [US2] Verify scrolling in `src/ui/screens/main.rs`: a list longer than the screen scrolls and follows the cursor in fullscreen (plan D4 — the panel's scroll logic is area-relative); the PTY check with a deep list

**Checkpoint**: Fullscreen is a workplace, not a viewer.

---

## Phase 3: Discovery and Verification (US3)

- [X] T006 [US3] Update `src/ui/hints.rs` (both panels' lines gain the fullscreen key), `src/ui/help.rs` (the main screen's actions), and the extraction tests in `src/ui/tests/hints.rs`; entries must fit the minimum width
- [X] T007 [US1] Verify the exit path in `src/ui/input.rs` (`App::run`'s save): quitting from fullscreen saves the trip exactly as from the main view (US1/AC3) — the save path is screen-independent; the PTY check
- [X] T008 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's toggle/actions/exit checks pass

---

## Dependencies & Execution Order

- T001 → T002 → T003 → T004/T005 → T006/T007 → T008.
- **Cross-spec**: composes with 006 (the stacked layout — fullscreen is one panel either way); the recalculate keys in T004 are 018's three scopes once 018 lands.

## Notes

- The 42 existing tests are the contract; the flag is presentation-only (US1/AC3's data-unchanged guarantee).
