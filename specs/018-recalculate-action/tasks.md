---

description: "Task list for Recalculate Amounts on an Existing List feature implementation"

---

# Tasks: Recalculate Amounts on an Existing List

**Input**: Design documents from `/specs/018-recalculate-action/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md (facts F1-F7, decisions D1-D5), data-model.md, contracts/recalculate-binding.md, quickstart.md

**Tests**: REQUIRED (constitution Principle III): the action's contract gets a state-layer test (auto update, fixed preserved, non-quantity state untouched, idempotent); the extraction tests are extended with the `R` binding in the same change.

**Organization**: Four user stories. US1+US3 (the action and its mode semantics) land together — they are one extraction plus one handler arm; US2 (hints/help/tokens) lands with them so the binding is never undocumented; US4 is a verification-only story (the existing exit-save flow already persists quantities). The state-layer extraction comes first so the handler and the test share the seam.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root; tests in `#[cfg(test)]` modules next to the code under test

---

## Phase 1: Setup

**Purpose**: Baseline verification before the change

- [X] T001 Verify the quality gate is clean before starting: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` — all pass, zero warnings; record the test count as the baseline. Note: if spec 017 (remove activities) has landed, the baseline reflects its revised test set; if not, the `R` handler and hints are written against the current activity-bearing surfaces and 017's plan already accounts for the rebase

---

## Phase 2: User Story 1 + US3 — The recalculate action, honoring item modes

**Goal**: `AppDrawState::recalculate_quantities` exists (extracted from `App::update_quantities`), the `R` handler calls it from the main screen in normal mode, and the action's contract is pinned by a state-layer test.

**Independent Test**: `cargo test recalculate` passes — auto items update to the current calculation, fixed items keep their values, packed/selection/collapse state is untouched, and a second run is a no-op.

### Tests first (they fail before the extraction)

- [X] T002 [US1] Write the failing state-layer test `recalculate_updates_auto_and_preserves_fixed` in `src/ui/app.rs` tests: construct a state with one auto and one fixed item (via the `test_state`/`test_item` helpers), set both quantities to stale bogus values, set packed/selection/collapse state, call the new `AppDrawState::recalculate_quantities(&QuantityCalculation::new())`, assert the auto item's quantity equals the current calculation (7 weekdays × 1.0 style × 1.0 weather = 7), the fixed item's quantity is unchanged, the packed/selection/collapse state is identical, and a second call produces identical quantities (SC-001/SC-002/SC-003/SC-005)

### Implementation

- [X] T003 [US1] Extract the state-layer method in `src/ui/app.rs`: move the body of `App::update_quantities` (~1907-1971) into `AppDrawState::recalculate_quantities(&mut self, calc: &QuantityCalculation) -> Result<()>` (multiplier resolution, input construction, calculate, quantity writeback); `App::update_quantities` becomes the one-line delegate `self.draw_state.recalculate_quantities(&self.quantity_calculation)` (the spec 015 state-layer pattern; D1)
- [X] T004 [US1] Add the `R` handler arm in `src/ui/app.rs` in the main-screen letter-handler block (~2443 area, next to the `C`/`A`/`E`/`D` arms): `KeyCode::Char('r') | KeyCode::Char('R')` guarded by `Screen::Main && InputMode::Normal` only — no `focus_side` guard, so the action works from both list views (EC-001) and is unbound on every other screen/mode (EC-002); the arm calls `self.update_quantities()?` (D2/D3)
- [X] T005 [US1] Run the full gate; the T002 test passes, all existing tests pass, zero warnings

**Checkpoint**: The action works and its contract is pinned.

---

## Phase 3: User Story 2 — Discoverable and documented

**Goal**: The `R` binding appears in both list panels' hint lines, the help screen, and all four extraction-test surfaces, in the same change.

**Independent Test**: `cargo test hint` passes with the extended surfaces; removing the `R` entry from any of them fails a test.

- [X] T006 [US2] Add `("R", "Recalculate")` to `hint_entries(HintSurface::LeftPanel)` and `hint_entries(HintSurface::RightPanel)` in `src/ui/app.rs` (~3598); the `hint_rows` footer reflows automatically (verified: at 80x24 the left footer grows one row, within the `Min(10)` list budget)
- [X] T007 [US2] Add the help entry in `help_content`'s MAIN SCREEN section in `src/ui/app.rs` (~3823-3835): `entry("R", "Recalculate quantities on the current list")` after the `D` entry
- [X] T008 [US2] Extend the extraction-test surfaces in `src/ui/app.rs`: add `"R"` to `help_binding_tokens` (~676) and to the handler list in `help_screen_text_covers_all_handler_bindings` (~660); add `("R", "Recalculate")` to both list panels' expected sets in `hint_entries_match_the_contract_binding_sets` (~743) — the both-directions contract pin makes a dropped or stale entry a test failure (US2/AC3, SC-004)
- [X] T009 [US2] Run the full gate; the extended extraction tests pass, zero warnings

**Checkpoint**: The binding cannot drift.

---

## Phase 4: User Story 4 — Persistence verification (no new code)

**Goal**: Confirm the recalculated amounts ride the existing exit-save flow and that the action itself writes nothing.

**Independent Test**: Recalculate, exit, resume; the quantities are the recalculated ones per the current persistence behavior; the trip file's mtime is unchanged by the key press alone.

- [X] T010 [US4] Verify by inspection and a manual run: `recalculate_quantities` performs no file I/O (FR-004/US4-AC2 — the method only writes `packing_list.items[i].quantity`); run the TUI, press `R`, confirm the trip file's mtime is unchanged, then exit and confirm the save flow writes the recalculated quantities per the existing behavior (US4-AC1)

**Checkpoint**: Persistence semantics confirmed; no code change required.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T011 Update `TUI_USAGE.md`: add `R | Recalculate quantities on the current list` to the main screen's navigation table and a step to the managing-your-packing-list workflow (press R to bring amounts up to date)
- [X] T012 Run all four quickstart.md validation scenarios and record the results (Scenario 1 via `cargo test recalculate` plus the TUI flow; Scenario 2 via `cargo test hint` plus reading the surfaces; Scenarios 3 and 4 in the TUI at 80x24)
- [X] T013 Final gate run: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all clean; live-render check at 80x24 confirms both panels' hint footers show every entry including R Recalculate with no overlap

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **US1+US3 (Phase 2)**: First — the extraction creates the seam the handler and the test share; the test (T002) is written before the extraction and fails until T003 lands
- **US2 (Phase 3)**: Depends on US1 (the hints document the final binding); lands immediately after so the binding is never undocumented
- **US4 (Phase 4)**: Verification only; any time after Phase 2
- **Polish (Phase 5)**: Depends on all stories

### Within Each User Story

- Test first where behavior changes (T002 before T003/T004)
- Full gate after every phase
- The hint/help/token updates (T006-T008) land together, never split

### Parallel Opportunities

- T006/T007/T008 touch different regions of the same file — sequential recommended to avoid conflicts
- T010 is parallelizable with Phase 3 (verification vs. code)
- Everything else is sequential: the feature is one connected change through `src/ui/app.rs`

---

## Implementation Strategy

### MVP First (US1 + US3 + US2)

1. Complete Phase 1: baseline
2. Complete Phase 2: the extraction, the handler, the test — the action works
3. Complete Phase 3: hints/help/tokens — the action is discoverable
4. **STOP and VALIDATE**: quickstart Scenarios 1 and 2

### Incremental Delivery

1. US1+US3 → US2 → validate (MVP)
2. US4 → verify persistence semantics
3. Polish (docs, scenarios, final gate, live render)

---

## Notes

- [P] tasks = different files, no dependencies; this feature is single-file, so tasks are sequential
- The constitution governs: warning-free gate after every phase, tests with the behavior, the binding documented in the same change it is added
- `R` is verified unbound today (research F1); the handler follows the established letter-handler pattern (F3)
- The action reuses the existing calculation path — no new calculation logic (the spec's own assumption)
- Interaction with spec 017: both features touch `update_quantities`'s input construction; whichever lands second rebases trivially (research R1) — 018 extracts the method, 017 edits its body
- Commit after each task or logical group (Conventional Commits per AGENTS.md)
