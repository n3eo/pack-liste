---

description: "Task list for Implemented-Specs Compliance Review feature implementation"

---

# Tasks: Implemented-Specs Compliance Review

**Input**: Design documents from `/specs/015-implemented-specs-compliance/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/input-and-help.md, quickstart.md

**Tests**: Tests are REQUIRED by this feature's own spec (FR-005, US5): every story ships with tests at the extracted state layer, plus the help-extraction test (SC-004) and the calculation-fallback tests (SC-003). The constitution (v1.0.0, Principle III) mandates tested features.

**Organization**: Tasks are grouped by user story. US5 (the test net) is implemented as the Foundational phase because every other story's tests depend on the state-layer extraction it performs. US4 (help) comes last among stories because it must list the final binding set after US1's dialog changes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root; tests live in `#[cfg(test)]` modules next to the code under test (existing convention)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Baseline verification before any change

- [ ] T001 Verify the quality gate is clean before starting: run `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test` (11 tests), `cargo fmt --check` — all must pass with zero warnings; record the test count as the baseline
- [ ] T002 Capture the current help screen content for the US4 comparison: run the app with `--force-tui`, press `?`, save the rendered text to `specs/015-implemented-specs-compliance/baseline-help.txt` (the "before" evidence for the missing-bindings finding)

---

## Phase 2: Foundational — State-Layer Extraction (US5, the test net)

**Purpose**: Extract the pure state transitions from `App` into terminal-free, unit-testable functions. This is US5's deliverable and the prerequisite for every other story's tests.

**⚠️ CRITICAL**: No user story implementation can begin until this phase is complete — their tests need the extracted layer.

- [ ] T003 Extract cursor navigation into `AppDrawState` methods in `src/ui/app.rs`: move `handle_up`/`handle_down`'s state logic (lines ~1464–1551: welcome focus prev/next, main-screen `selected_index` movement with `active_rows()`, add/edit category-index movement) into `AppDrawState::move_cursor_up/down`; `App::handle_up/down` become one-line delegates that then call `sync_left_view_to_right_cursor` where the current code does
- [ ] T004 Extract the selection/packed/collapse toggles into `AppDrawState` methods in `src/ui/app.rs`: `toggle_item_selection` (~2022), `toggle_category_selection` (~2030), `toggle_item_packed_right_side` (~2056), `toggle_category_collapse` (~2013), `set_category_collapsed` (~1989) — all operate only on `draw_state` fields; move them to `impl AppDrawState` and have `App` delegate
- [ ] T005 Extract the edit-dialog save logic into `AppDrawState::apply_item_edit` in `src/ui/app.rs`: move the field-application part of `save_edited_item` (~2152: name/category/quantity writes with the empty-field-keeps-old-value rule) to take `(&mut Item, name, category, quantity)`; `App::save_edited_item` keeps the `save_config`/`update_quantities` side effects and calls the extracted method
- [ ] T006 Write the input-handling test net in `src/ui/app.rs` `#[cfg(test)] mod tests`: construct `AppDrawState` values directly (items, categories, collapsed set, selected_index), dispatch `crossterm::event::KeyEvent` values through the extracted methods, and assert state — cover: cursor up/down on main (both focus sides, boundary at 0 and last row), welcome focus cycling, item selection toggle, category selection toggle, packed toggle, collapse toggle with cursor-to-header, edit-dialog save with empty-field-keeps-old-value; include one mutation-guard test (e.g., assert Up at index 0 stays at 0 — fails if the binding logic inverts)
- [ ] T007 Run the full gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check`): all existing 11 tests plus the new input tests pass, zero warnings; the extraction is behavior-preserving

**Checkpoint**: The state layer is testable without a TTY; the input net exists and fails on broken bindings. US5's independent test is met (quickstart.md Scenario 5).

---

## Phase 3: User Story 1 — Auto/Fixed Quantity Toggle in the Edit Dialog (Priority: P1) 🎯 MVP

**Goal**: The item edit dialog gains a Mode field; the auto/fixed flag is editable in the TUI; the silent auto-quantity overwrite is gone.

**Independent Test**: Edit an automatic item, toggle to fixed, save, change trip length — quantity stays; toggle back — it recalculates (quickstart.md Scenario 1).

### Tests for User Story 1

- [ ] T008 [P] [US1] Write failing state-layer tests in `src/ui/app.rs` tests: `apply_item_edit` with mode fixed→auto recalculates via the caller, auto→fixed freezes the current quantity (EC-003: "After toggling to fixed, the quantity equals the pre-toggle calculated value until edited"), and no fixed item can hold a quantity below 1 (EC-004); write the dialog-key tests (Tab cycles Name→Category→Quantity→Mode→Name; Space on Mode toggles) against the extracted dialog state

### Implementation for User Story 1

- [ ] T009 [US1] Add `Mode` to `InputField` and the Tab rotation in `src/ui/app.rs` (~79, ~1296–1300, ~1302–1308): cycle Name → Category → Quantity → Mode → Name in both AddingItem and EditingItem modes
- [ ] T010 [US1] Add `auto_quantity: bool` to `InputMode::EditingItem` in `src/ui/app.rs` (~96): `start_editing_item` (~2093) carries the item's current flag into the dialog; Space while the Mode field is focused toggles it in `handle_add_edit_input` (~1909)
- [ ] T011 [US1] Extend `save_edited_item` in `src/ui/app.rs` (~2152) and `AppDrawState::apply_item_edit` (from T005): write the dialog's `auto_quantity` back to the item; when the mode flipped to auto, call `update_quantities` after save so the quantity recalculates immediately (FR-001); the quantity edit applies per the mode shown — an auto item's quantity edit no longer silently sticks
- [ ] T012 [US1] Render the Mode field in `draw_edit_item_popup` in `src/ui/app.rs` (~3690): a fourth line "Mode: automatic|fixed" with the focused-field highlight, using the existing field style; extend `PopupFields` (~117) with `auto_quantity: bool`
- [ ] T013 [US1] Run the full gate; all US1 tests pass (T008 now green), zero warnings

**Checkpoint**: US1 complete and independently testable.

---

## Phase 4: User Story 2 — Category Packing Indicator (Priority: P1)

**Goal**: Compiled-list category headers show all-packed (`✓` green) vs. partially-packed (`○` red), derived at draw time.

**Independent Test**: Pack all items of one category, leave another partial; the headers show the correct markers and flip within one render (quickstart.md Scenario 2).

### Implementation for User Story 2

- [ ] T014 [US2] Derive the per-category packed state at draw time in `draw_selected_items_panel` in `src/ui/app.rs` (~3317): when rendering a `PanelRow::Category` header (~3413), compute `all_packed` over the category's selected items and prepend the marker — `✓` with `Color::Green` when all packed, `○` with `Color::Red` otherwise (EC-001: "The two states are visually distinct from each other and from the collapse arrow"); left panel headers unchanged (EC-002)
- [ ] T015 [US2] Add state-layer tests in `src/ui/app.rs` tests for the derivation helper: a category with all items packed → all-packed; one unpacked item → partial; empty category → all-packed (vacuous truth); expose the derivation as a small pure function (e.g., `fn category_all_packed(items: &[bool]) -> bool`) so it is testable without rendering
- [ ] T016 [US2] Run the full gate; US2 tests pass, zero warnings

**Checkpoint**: US2 complete and independently testable.

---

## Phase 5: User Story 3 — Uniform Unknown-Modifier Fallback (Priority: P2)

**Goal**: Unknown activity names resolve to 1.0 like travel styles and weather; the activity path stays in effect.

**Independent Test**: `cargo test quantity` — the unknown-activity case contributes 1.0 to the average (quickstart.md Scenario 3).

### Tests for User Story 3

- [ ] T017 [P] [US3] Write failing calculation tests in `src/quantity/calculation.rs` tests: an activity multiplier list containing 1.0 entries (the resolved form of unknown names) keeps the activity path in effect and averages them (EC-005: "With unknown activities selected, the note reflects the activity path, not the weather path"); a mix of known and unknown averages the unknown at 1.0

### Implementation for User Story 3

- [ ] T018 [US3] Change the activity multiplier collection in `update_quantities` in `src/ui/app.rs` (~853–858): replace `filter_map(|name| ...activity_multiplier(name))` with `map(|name| ...activity_multiplier(name).unwrap_or(1.0))` — matching the travel-style and weather fallbacks two lines above (~847, ~852) and spec 005 FR-010; known-activity calculations are bit-identical
- [ ] T019 [US3] Run the full gate; US3 tests pass (T017 green), the existing calculation tests unchanged and green, zero warnings

**Checkpoint**: US3 complete and independently testable.

---

## Phase 6: User Story 4 — Complete Help Screen (Priority: P2)

**Goal**: The help screen lists every active binding per the contract, with an extraction test enforcing 1:1 agreement.

**Independent Test**: `cargo test help` — the extraction test passes; manual `?` shows all groups (quickstart.md Scenario 4).

**Note**: This story lands after US1 so the help lists the final binding set (including the new Mode field keys) once.

### Tests for User Story 4

- [ ] T020 [P] [US4] Write the failing help-extraction test in `src/ui/app.rs` tests: extract the binding tokens from the help content (a `fn help_binding_tokens() -> Vec<&'static str>` next to `draw_help_screen`) and from a curated list of handler bindings; assert no handler binding is missing from help, no help entry is stale, and no entry is duplicated (SC-004)

### Implementation for User Story 4

- [ ] T021 [US4] Rebuild the help content in `draw_help_screen` in `src/ui/app.rs` (~2630) grouped by context per `specs/015-implemented-specs-compliance/contracts/input-and-help.md`: Global (Ctrl+C, Ctrl+E, q, ?, O), Welcome (Up/Down, Left/Right, +/-, Space, Enter, F, Esc), Resume prompt (Y/Enter, N/Esc), Main (Up/Down, Tab/BackTab, Space, Ctrl+Space, C, A, E, D, Esc), Add/Edit dialog (Tab, Up/Down, Left/Right, Space, Enter, Esc — including the US1 Mode field), Config screen (Tab/BackTab, Up/Down, A, E, D, Esc), Modifier editor (Tab, Enter, Esc), Raw config editor (arrows, Home/End, Enter, Backspace, Ctrl+S, Esc); remove the duplicated "Enter - Continue" entry (~2663–2682)
- [ ] T022 [US4] Run the full gate; the extraction test passes (T020 green), zero warnings

**Checkpoint**: US4 complete; the help screen and the code are in verified 1:1 agreement.

---

## Phase 7: User Story 6 — Multi-Category Items (Priority: P3)

**Goal**: An item belongs to one or more categories; the tree shows it under each; selection and packed state are shared across occurrences.

**Independent Test**: Give an item two categories; it appears under both; deselecting under one deselects it everywhere; the compiled list shows it once (quickstart.md Scenario 6).

### Implementation for User Story 6

- [ ] T023 [US6] Change `Item.category: String` to `Item.categories: Vec<String>` in `src/models/item.rs` (constraint from data-model.md: "categories is non-empty (an item always belongs to at least one category...)" and "Duplicate entries within one item's categories are deduplicated on save"); update `Item::new` (~18) to take `Vec<String>`
- [ ] T024 [US6] Update `SavedItem` serialization in `src/ui/app.rs` (~147): write `categories: Vec<String>` per item; read with a serde default that lifts a legacy single `category: <string>` into a one-element list so existing config files load unchanged (additive format change per data-model.md)
- [ ] T025 [US6] Update the grouping and rendering sites in `src/ui/app.rs` to iterate memberships: `left_rows` (~441), `right_item_order` (~465), `draw_all_items_panel` grouping (~3144), `draw_selected_items_panel` grouping, `get_all_categories` (~781), `get_unique_categories` (~721) — an item appears once per membership in the tree; the compiled list shows it at most once (primary category first)
- [ ] T026 [US6] Verify selection semantics stay per-item in `src/ui/app.rs`: `toggle_item_selection` (~2022) and `toggle_category_selection` (~2030) flip the item's `editable` once; all occurrences reflect it on the next render (EC-006: "Category toggles and item toggles produce one consistent selection state per item")
- [ ] T027 [US6] Update `TripFile::reconcile` in `src/config/trip_file.rs` (~68): match a trip item by case-insensitive name against any of the catalog item's categories (prefer exact name+category as today); the trip file's `TripItem.category` stores the primary (first) category
- [ ] T028 [US6] Make the edit dialog's category field a multi-select in `src/ui/app.rs`: Space toggles membership of the highlighted category (per the contract's dialog binding), Up/Down move the highlight, Left/Right cycle; saving with zero memberships is rejected (data-model.md: "saving with zero memberships is rejected"); `start_editing_item` (~2093) seeds the membership set
- [ ] T029 [US6] Add state-layer tests in `src/ui/app.rs` tests: an item with two categories appears under both in `left_rows`; toggling selection through either occurrence flips the item once; the compiled list contains the item once; reconcile matches by name across memberships (extend the existing `trip_file_round_trips_and_reconciles` test, ~269)
- [ ] T030 [US6] Run the full gate; US6 tests pass, all existing tests green (the additive config default keeps the round-trip tests valid), zero warnings

**Checkpoint**: US6 complete and independently testable.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all stories

- [ ] T031 Run all six quickstart.md validation scenarios and record the results (Scenarios 1, 2, 6 are manual TUI checks via `--force-tui`; 3, 4, 5 are `cargo test` runs)
- [ ] T032 Final gate run: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all clean; diff the help screen against `specs/015-implemented-specs-compliance/baseline-help.txt` (from T002) and confirm every previously missing binding (Ctrl+C, F, BackTab, config Up/Down, Home/End, modifier editor keys, Esc roles) is now listed
- [ ] T033 Update the affected implemented specs with alignment notes: spec 003 FR-002 (mode now TUI-editable), spec 003 input pt. 4 (indicator restored), spec 005 FR-010 (fallback now uniform), spec 001 input pt. 3 (multi-category restored) — one note per spec pointing to this feature

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2, US5)**: Depends on Setup; **BLOCKS all user stories** — their tests need the extracted state layer
- **US1 (Phase 3)**: Depends on Phase 2 (tests use the extracted layer)
- **US2 (Phase 4)**: Depends on Phase 2; independent of US1
- **US3 (Phase 5)**: Depends on Phase 2 only; fully independent (calculation-layer)
- **US4 (Phase 6)**: Depends on US1 (help must list the final binding set including the Mode field)
- **US6 (Phase 7)**: Depends on Phase 2; independent of US1–US4 but touches the same files — run sequentially after US1 to avoid conflicts in the edit dialog
- **Polish (Phase 8)**: Depends on all stories

### User Story Dependencies

- **US5 (P2, Foundational)**: The test net — first, everything depends on it
- **US1 (P1)**: After US5; MVP together with US5
- **US2 (P1)**: After US5; parallelizable with US1 in principle, but both edit `src/ui/app.rs` — sequential recommended
- **US3 (P2)**: After US5; touches `src/ui/app.rs` (one line) + `src/quantity/calculation.rs` (tests) — can run parallel with US2
- **US4 (P2)**: After US1 (final binding set)
- **US6 (P3)**: After US5 and US1 (edit-dialog conflicts); the largest ripple, last among stories

### Within Each User Story

- Tests first (they must fail before implementation)
- Model/entity changes before rendering changes
- Rendering before help (US4 last)
- Full gate after every story

### Parallel Opportunities

- T002 [P] parallel with T001
- T008 [US1 tests] and T017 [US3 tests] are parallelizable (different modules)
- T020 [US4 test] parallel with US2/US3 implementation
- US3's calculation tests (T017) touch a different file than US1/US2 — safe to parallelize

---

## Implementation Strategy

### MVP First (US5 + US1)

1. Complete Phase 1: Setup (baseline evidence)
2. Complete Phase 2: US5 state-layer extraction + input test net
3. Complete Phase 3: US1 auto/fixed toggle
4. **STOP and VALIDATE**: the input net passes, the mode toggle works end-to-end (quickstart Scenarios 1, 5)
5. The largest functional gap is closed on a tested foundation

### Incremental Delivery

1. Setup → US5 (test net) → validate
2. US1 (mode toggle) → validate → MVP
3. US2 (category indicator) → validate
4. US3 (fallback) → validate
5. US4 (help) → validate
6. US6 (multi-category) → validate
7. Polish (scenarios, final gate, spec alignment notes)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- The constitution governs: warning-free gate after every task, tests with every story, removal-not-annotation for any dead code exposed by the refactor
- The US5 extraction is behavior-preserving; if any existing test fails during it, the extraction is wrong — fix the extraction, not the test
- US6's config format change is additive (legacy `category:` lifts into a one-element list); spec 013 (no-compat, Draft) is NOT yet implemented — do not remove the legacy lift in this feature
- Commit after each task or logical group (Conventional Commits per AGENTS.md)
