---

description: "Task list for Harmonized Navigation, UI, and UX feature implementation"

---

# Tasks: Harmonized Navigation, UI, and UX Across Views

**Input**: Design documents from `/specs/016-harmonized-ui-navigation/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/hints-and-bindings.md, quickstart.md

**Tests**: REQUIRED (constitution Principle III): the dialog-cycling behavior gets state-layer tests; the hint/help agreement is enforced by extending the spec 015 extraction test.

**Organization**: Five user stories. US3's hint helper lands first (US2's footers use it); US1's binding changes land before US5's help update so the help states the final set once.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root; tests in `#[cfg(test)]` modules next to the code under test

---

## Phase 1: Setup

**Purpose**: Baseline verification before any change

- [X] T001 Verify the quality gate is clean before starting: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test` (32 tests), `cargo fmt --check` — all pass, zero warnings; record the test count as the baseline

---

## Phase 2: User Story 3 — One hint style (lands first; US2 uses its helper)

**Goal**: A single `hint_line` constructor and per-surface `hint_entries` data; every screen's hints rendered through them.

**Independent Test**: Every hint line uses " | " separators, ↑/↓/←/→ glyphs, yellow-bold keys; the corrupted "Select fields.category" text is gone.

- [X] T002 [US3] Add the hint constructor and per-surface data in `src/ui/app.rs`: `fn hint_line(entries: &[(&'static str, &'static str)]) -> Line<'static>` (keys `Color::Yellow` bold, actions plain, " | " between entries) and `fn hint_entries(surface: HintSurface) -> &'static [(&'static str, &'static str)]` with the eight surfaces from data-model.md (left panel, right panel, welcome, config, add/edit popup, delete popup, modifier editor, raw editor); a small `enum HintSurface` keys the data
- [X] T003 [US3] Convert every screen's hint rendering in `src/ui/app.rs` to `hint_line(hint_entries(...))`: welcome screen footer (~3215), config screen footer (~3668), raw editor footer (~3770 area), add/edit popup hint blocks (~4326 and ~4480 area — replacing the corrupted "Select fields.category" with "Select category"), delete popup (~4610 area), modifier editor popup (~3560 area); convert the welcome screen's "Up/Down"/"Left/Right" words to ↑/↓/←/→ glyphs per EC-004
- [X] T004 [US3] Run the full gate; all 32 existing tests pass, zero warnings

**Checkpoint**: One hint style everywhere; the helper and data exist for US2 and US5.

---

## Phase 3: User Story 2 — Always-visible hint footers on both list views

**Goal**: Both list panels show their hint line at every supported size, as a fixed footer row.

**Independent Test**: At large and small (80x24) sizes, both panels show hints; a deep list shrinks the viewport, not the hints.

- [X] T005 [US2] Change the left panel's layout in `draw_all_items_panel` in `src/ui/app.rs` (~3799): split [title 1, list min 10, hints 1] instead of [title 1, list min 10]; render `hint_line(hint_entries(HintSurface::LeftPanel))` in the fixed hints area; delete the conditional `visible_rows + 5 < height` inline hint block (~3955–3993)
- [X] T006 [US2] Change the right panel's layout in `draw_selected_items_panel` in `src/ui/app.rs` (~4004): split [title 1, list min 10, hints 1, stats 5]; render `hint_line(hint_entries(HintSurface::RightPanel))` in the hints row; the stats block stays (EC-005)
- [X] T007 [US2] Verify the list viewport computation in both panels accounts for the one-row-smaller list area (scroll windows use the new list_area height); run the full gate

**Checkpoint**: Both list views always show hints; no overlap with list content.

---

## Phase 4: User Story 1 — Uniform arrow model

**Goal**: Left/Right never collapse/expand on the main screen; the dialog's category field cycles with Left/Right.

**Independent Test**: ←/→ do nothing on the main screen; `C` still collapses; the dialog's ←/→ cycle the category within bounds.

- [X] T008 [US1] Write failing state-layer tests in `src/ui/app.rs` tests: Left/Right cycle `category_index` within `category_list` bounds in the AddingItem and EditingItem modes when the Category field is focused (clamp at 0 and len-1, no wrap), and do nothing when another field is focused
- [X] T009 [US1] Delete the Left/Right collapse/expand handler block in `src/ui/app.rs` (~1811–1821, the `matches!(key.code, KeyCode::Left | KeyCode::Right)` block guarded by `Screen::Main` + `FocusSide::Left`); `C` and Ctrl+Space are untouched (EC-003)
- [X] T010 [US1] Add the Left/Right category cycling to `handle_add_edit_input` in `src/ui/app.rs` (~2622): a `KeyCode::Left | KeyCode::Right` arm in both AddingItem and EditingItem modes that steps `category_index` by ±1 within `category_list` bounds when the Category field is focused (same clamp logic as `move_cursor_up/down` at ~1178/~1218)
- [X] T011 [US1] Run the full gate; the T008 tests pass, the existing collapse tests (via `C`) still pass, zero warnings

**Checkpoint**: The arrow model is uniform; the binding set matches the contract.

---

## Phase 5: User Story 4 — Focus and cursor indication

**Goal**: One focus-indication convention across both panels; cursor highlight already uniform (verified).

**Independent Test**: Tab-switch focus; the focused title is highlighted per one convention on both panels.

- [X] T012 [US4] Unify the panel title focus indication in `src/ui/app.rs`: both panels' titles use the same mechanism (focused: bold + the panel's accent color; unfocused: plain white) — left panel (~3822–3833), right panel (~4022–4033); verify the cursor row highlight (Yellow on DarkGray) is identical on both panels (already verified in research; assert in a test if trivially assertable, else leave as-is)
- [X] T013 [US4] Run the full gate; zero warnings

**Checkpoint**: Focus indication is uniform.

---

## Phase 6: User Story 5 — Help and extraction test (last; states the final binding set)

**Goal**: The help screen matches the harmonized bindings; the extraction test covers hints and help.

**Independent Test**: `cargo test help` passes; breaking a hint line or the help content fails it.

- [X] T014 [US5] Update `help_content` and `help_binding_tokens` in `src/ui/app.rs` (~3309, ~3365): remove the main-screen "←/→ Collapse/expand" entry; add the dialog's "←/→ Cycle category" entry; convert the welcome entries to glyph style; keep every other entry per the contract `specs/016-harmonized-ui-navigation/contracts/hints-and-bindings.md`
- [X] T015 [US5] Extend the extraction test in `src/ui/app.rs` tests (~558–630): cross-check every `hint_entries(surface)` key against `help_binding_tokens` (every hint key is a help token; every surface's binding set is covered by its hint entries); the test fails on missing, stale, or duplicated entries in either direction
- [X] T016 [US5] Run the full gate; the extended extraction test passes, zero warnings

**Checkpoint**: Help and hints agree and cannot drift.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T017 Run all five quickstart.md validation scenarios and record the results (Scenarios 1–4 are TUI checks via `--force-tui` at two terminal sizes; Scenario 5 is `cargo test help`)
- [X] T018 Final gate run: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all clean; confirm the corrupted hint text is gone (`grep -rn "fields.category" src/` returns nothing) and no hint line uses long-form arrow words (`grep -rn '"Up/Down"' src/ui/app.rs` returns nothing outside test data)
- [X] T019 Add the alignment note to spec 014 (uniform arrows, Draft): its main-screen and dialog requirements are implemented by this feature; 014's remaining content (the model definition) is satisfied

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **US3 (Phase 2)**: First — US2's footers and US5's test consume its helper and data
- **US2 (Phase 3)**: Depends on US3 (uses `hint_line`/`hint_entries`)
- **US1 (Phase 4)**: Independent of US2/US3 in code, but lands after them to keep the hint data stable while bindings change
- **US4 (Phase 5)**: Independent; any time after US3
- **US5 (Phase 6)**: Depends on US1 (help states the final binding set) and US3 (test reads `hint_entries`)
- **Polish (Phase 7)**: Depends on all stories

### Within Each User Story

- Tests first where behavior changes (US1's T008 before T009/T010)
- Full gate after every story

### Parallel Opportunities

- T012 (US4) is parallelizable with US1's implementation (different regions of the same file — sequential recommended to avoid conflicts)
- T017/T018/T019 are sequential polish steps

---

## Implementation Strategy

### MVP First (US3 + US2 + US1)

1. Complete Phase 1: baseline
2. Complete US3 (hint system) — the foundation
3. Complete US2 (list-view footers) — the user-visible core request
4. Complete US1 (uniform arrows) — the behavior change
5. **STOP and VALIDATE**: hints on both panels at every size, arrows uniform, `C` still collapses (quickstart Scenarios 1–3)

### Incremental Delivery

1. US3 → US2 → US1 → validate (MVP)
2. US4 → validate
3. US5 → validate (help + drift guard)
4. Polish (scenarios, final gate, spec 014 note)

---

## Notes

- [P] tasks = different files, no dependencies; this feature is single-file, so most tasks are sequential
- The constitution governs: warning-free gate after every task, tests with every behavior change, removal-not-annotation (the deleted handler block goes completely)
- The extraction test is the drift guard: every binding change (T009, T010, T014) updates handler + hints + help + tokens together, and T015 makes forgetting a test failure
- The hint data intentionally still contains the welcome screen's activity entries; spec 017 (remove activities, Draft) removes them in its own change if implemented
- Commit after each task or logical group (Conventional Commits per AGENTS.md)

---

## Phase 8: Convergence

Appended by `/speckit-converge` (2026-09-20) after assessing the codebase
against spec.md, plan.md, tasks.md, and the constitution. Constitution
violations come first.

- [X] T020 CRITICAL: Update TUI_USAGE.md's key-binding documentation to the harmonized final binding set per contracts/hints-and-bindings.md — remove the nonexistent `X` (category toggle) and `Enter` (item selection) entries, document `Space`/`Ctrl+Space`/`E`/`?`/`O`/`Ctrl+E`, the add/edit dialog's ←/→ category cycling, and the always-visible hint footers, in the same change as the binding alteration (Constitution III) (missing)
- [X] T021 Make both list panels' hint lines fully readable at the validated minimum size (80x24): the left panel's 112-char line renders only "↑/↓ Navigate | Space Select |" in its 39-column panel (2 of 9 entries; full fit needs a ~226-column terminal) and the right panel clips "Switch | ? Help" — compact the per-surface entries or add a size-aware variant so every binding stays visible per FR-002/US2-AC3/SC-002 (partial)
- [X] T022 Extend the extraction test to the drop/stale direction: cross-check each surface's binding set both ways so removing or corrupting a hint entry (e.g. deleting ("C", "Collapse") from the left panel) fails the test per FR-006/US5-AC2 (partial)
- [X] T023 Remove the stray `test-config.yaml` from the repository root — a copy of the user's real config, referenced by no spec, plan, task, or code; risks committing personal data (unrequested)
