---
description: "Task list for 018-recalculate-action (amended model)"
---

# Tasks: Recalculate Amounts on an Existing List (Amended Model)

**Input**: Design documents from `/specs/018-recalculate-action/` (amended 2026-09-20)

**Prerequisites**: plan.md (re-planned, required), spec.md (amended, required)

> **Re-plan note**: The original tasks (T001–T013, all marked done) targeted
> the pre-amendment model and are superseded by this regenerated list.
> The amended model: manual quantities are authoritative, recalculation
> is explicit-only with three scopes (R / Ctrl+R / Ctrl+Shift+R).

**Tests**: Included per story (the spec's SC-001..SC-005 demand coverage).

**Organization**: Composes with 012 (resume-restore) — land together or 012 first.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Scoped Recalculation (US3)

**Goal**: The state layer recalculates by scope: current item, current category, everything.

**Independent Test**: With three categories and a manual value in each: R changes only the cursor's item; Ctrl+R the cursor's category; Ctrl+Shift+R everything; fixed items never change (SC-003).

- [X] T001 [US3] Add `RecalculateScope { Item(usize), Category(String), All }` and `recalculate_quantities_scoped(calc, scope)` to `AppDrawState` in `src/ui/state.rs` (plan D1): items outside the scope pass as `Fixed(current)` so they are untouched; fixed-mode items inside the scope also pass as `Fixed` (FR-004); the existing all-items method delegates to `Scope::All`
- [X] T002 [US3] Add scope tests in `src/ui/tests/state.rs`: item scope changes exactly one item; category scope exactly the category's automatic items; all scope everything automatic; fixed items unchanged in every scope (FR-004); idempotent per scope (FR-005); packed/selection/collapse untouched (FR-006)

**Checkpoint**: The three scopes exist and are correct.

---

## Phase 2: Manual Authority (US1, US2)

**Goal**: No automatic trigger overwrites a quantity; only the three keys recalculate.

**Independent Test**: Set a manual quantity; change every parameter, save, add/edit other items, apply the raw editor — the value never changes (SC-001/SC-002); press a recalculate key — the scope updates.

- [X] T003 [US2] Remove the automatic recalculation triggers (plan D4, verified inventory): the 4 welcome-parameter sites and the resume-decline site in `src/ui/handlers/welcome.rs`; the 2 modifier sites in `src/ui/handlers/config.rs`; the raw-editor-apply site in `src/ui/handlers/editor.rs`; the add/edit sites in `src/ui/handlers/dialogs.rs` (`update_quantities_or_report` calls); the trip-apply site in `src/ui/input.rs`
- [X] T004 [US2] Keep exactly the two load-time calculations: the fresh-list initial calculation in `App::new` (FR-008) — and remove the resume-path recalculation (with 012 landed, resume restores saved quantities; without 012, resume keeps the restored values without recalculating)
- [X] T005 [US1] Add manual-authority tests in `src/ui/tests/`: a manually set quantity survives parameter changes, config saves, add/edit of other items, and a raw-editor apply (SC-001); no non-recalculate event changes any quantity (SC-002)
- [X] T006 [US2] Edit/add save semantics (FR-009, EC-003/EC-004): `save_edited_item` applies exactly the dialog's values with no recalculation; `save_new_item` (post-021: with the dialog's mode) applies exactly the dialog's values — verify and test

**Checkpoint**: The calculation is a tool, not a background authority.

---

## Phase 3: Bindings and Discovery (US3, US4)

**Goal**: The three keys work from both list views, including category headers, and are documented.

**Independent Test**: All three scopes from both panels and from a header (EC-001/EC-002); the hints and help list all three keys (SC-004).

- [X] T007 [US3] Wire the handler arms in `src/ui/handlers/welcome.rs` (main screen, normal mode): plain `r`/`R` → item scope (on a header: the header's category, EC-001); `Ctrl+R` → category scope; `Ctrl+Shift+R` → all scope (plan D5: match modifiers before the plain arm); both panels use the cursor's item (EC-002)
- [X] T008 [US4] Update hints (`src/ui/hints.rs`: both list panels), help (`src/ui/help.rs`: the three keys with scopes), and the extraction tests (`help_binding_tokens` + the handler-binding list gain `Ctrl+R`, `Ctrl+Shift+R`); entries must fit the minimum width
- [X] T009 [US4] Update `TUI_USAGE.md` with the three keys and the manual-authority model
- [X] T010 [US5] Persistence tests in `src/ui/tests/config_io.rs`: manual and recalculated quantities survive exit/resume exactly as saved (SC-001's restart clause; with 012, the trip file carries them)

**Checkpoint**: Discoverable, documented, drift-guarded.

---

## Phase 4: Verification

- [X] T011 Final gate: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all exit 0; the quickstart's manual-authority, scope, and resume checks pass; PTY screen-diff shows no unintended behavior change on the untouched flows

---

## Dependencies & Execution Order

- T001–T002 (scoped state layer) before T007 (the handler arms call it).
- T003–T006 (trigger removal + authority) independent of T001 but same files as T007 — sequence T003, then T007.
- T008–T010 after T007 (they document what exists).
- **Cross-spec**: with 012 landed, T004's resume path restores the file's quantities (012's T003); without 012, land 012 first or together. Post-021, T006 covers the add dialog's mode.

## Notes

- The 42 existing tests are the contract; the tests asserting auto-recalculation side effects (if any) are updated with the trigger removal — the amended spec is the authority.
- The old tasks.md (T001–T013, done) is superseded; this list is the amended model's complete task set.
