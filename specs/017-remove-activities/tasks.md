---

description: "Task list for Remove Activity Options feature implementation"

---

# Tasks: Remove Activity Options

**Input**: Design documents from `/specs/017-remove-activities/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md (the 23-touchpoint inventory, refreshed post-018), data-model.md, contracts/bindings-and-formats.md, quickstart.md

**Tests**: REQUIRED (constitution Principle III): the removal ships with updated tests — activity-specific tests are removed with their behavior, legacy-tolerance and direct-rule tests are added, and the extraction tests are updated in the same change as the binding removal.

**Organization**: Three user stories. US2 (calculation) lands first — it is the deepest cut and everything else is mechanical against research.md's inventory; US1 (UI removal) follows; US3 (file tolerance) is verified by tests throughout. The hints/help/test-contract update (US1's tail) lands last so it states the final binding set once. Line numbers are verified against the current source (post-018).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root; tests in `#[cfg(test)]` modules next to the code under test

---

## Phase 1: Setup

**Purpose**: Baseline verification before the removal

- [X] T001 Verify the quality gate is clean before starting: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test` (39 tests, the post-018 baseline), `cargo fmt --check` — all pass, zero warnings; record the test count as the baseline

---

## Phase 2: User Story 2 — The calculation uses weather directly (lands first: the deepest cut)

**Goal**: `calculate` applies `style × weather` directly; the activity-priority branch, its input fields, and its tests are gone; the direct rule is covered.

**Independent Test**: `cargo test quantity` passes with the direct-rule tests (7 × 1.0 × 1.5 = 11); no activity-priority test remains; the notes text mentions weekdays, style, weather only.

- [X] T002 [US2] Delete the activity-priority branch in `calculate` in `src/quantity/calculation.rs` (~25-33): `combined` becomes `style_multiplier * weather_multiplier` directly; delete the activities notes variant (~46-61) so notes always read "Calculated for {n} weekdays, {style} style, {weather} weather"
- [X] T003 [US2] [P] Delete `activities` and `activity_multipliers` from `QuantityCalculationInput` in `src/quantity/model.rs` (~26-27)
- [X] T004 [US2] Delete the activity tests in `src/quantity/calculation.rs` (`activities_replace_weather_multiplier`, `unknown_activities_contribute_neutral_multiplier`, ~113-143) and drop the `activities` parameter from the test helper `input()` (~82-98); keep/verify the direct-rule tests (`base_quantity_is_weekdays_times_multipliers` covers 7 × 1.5 = 11)
- [X] T005 [US2] Delete the activity multiplier resolution in `AppDrawState::recalculate_quantities` in `src/ui/app.rs` (~1494-1523): remove the `activity_multipliers` construction and the two input fields from the `QuantityCalculationInput` literal (the spec 018 state-layer method; its `R`-related behavior is untouched)
- [X] T006 [US2] Run the full gate; the quantity tests pass, zero warnings (the `[lints]` table surfaces any orphaned reference)

**Checkpoint**: The calculation is the pre-activity rule; the app compiles with activities still present in the UI (unused state is removed in Phase 3).

---

## Phase 3: User Story 1 — No activity input anywhere

**Goal**: The welcome field, config section, handlers, state, hints, and help entries are gone; the focus and section cycles cover exactly the remaining members.

**Independent Test**: The welcome focus cycle is file → weekdays → style → weather → file; the config section cycle is travel styles ↔ weather; `grep -rni activit src/` returns nothing outside git history.

- [X] T007 [US1] Delete `Focus::Activities` and its `next()`/`prev()` arms in `src/ui/app.rs` (~53, 62-63, 69, 73); the cycle becomes file → weekdays → style → weather → file
- [X] T008 [US1] Delete `ConfigSection::Activities` and its `title()`/`next()`/`prev()` arms in `src/ui/app.rs` (~1234-1258); the cycle becomes TravelStyles ↔ Weather
- [X] T009 [US1] Delete the activity state fields `selected_activities` and `activity_cursor` from `AppDrawState` in `src/ui/app.rs` (~1191, 1200) and from every construction site (test helper ~263-272, App::new ~1770-1779, and the remaining initializers)
- [X] T010 [US1] Delete `toggle_activity` in `src/ui/app.rs` (~1991-2004) and its two handler arms: the Space dispatch on the welcome screen (~2467-2470) and the Left/Right activity-cursor arm (~2181-2187)
- [X] T011 [US1] Delete the welcome screen's activities block rendering in `src/ui/app.rs` (~3509-3557) and the fifth `params_layout` constraint (~3401-3411); the four remaining fields keep their `Length(3)` rows
- [X] T012 [US1] Delete the config screen's Activities arms: the tab rendering spans (~4047-4052), the `config_section_modifiers`/`config_section_modifiers_mut` match arms (~2619, 2630), and the `reconcile_selections` activity retain (~2953-2959)
- [X] T013 [US1] [P] Delete `ModifiersConfig.activities`, `default_activities()`, and `activity_multiplier()` in `src/config/modifiers.rs` (~20-21, 49-54, 83-92)
- [X] T014 [US1] Update the hint and help surfaces in `src/ui/app.rs`: remove `("Space", "Toggle activity")` from the Welcome hint entries (~3672) and from the extraction test's expected Welcome set (~781); drop the activity clauses from the help entries (~3856, 3860, 3863, 3907) — "Move focus between fields (file, weekdays, style, weather)", "Adjust the focused value (weekdays, cycle style/weather)", the Space entry removed, "Switch section (travel styles / weather)". The spec 018 `R` entries in the same surfaces are untouched
- [X] T015 [US1] Update the extraction tests in `src/ui/app.rs`: `help_binding_tokens` and `help_screen_text_covers_all_handler_bindings` lose the activity-only tokens/assertions (Space stays — it is bound on the main screen and the dialog); `welcome_focus_cycles_with_up_down` (~327-338) expects the wrap to land on `Focus::Weather`
- [X] T016 [US1] Run the full gate; zero warnings; `grep -rni "activit" src/` returns nothing

**Checkpoint**: No activity surface remains anywhere in the code.

---

## Phase 4: User Story 3 — Existing files load without activities

**Goal**: Legacy config/trip files with activity fields parse without error; new saves contain no activity keys; both properties are pinned by tests.

**Independent Test**: `cargo test` passes including the new legacy-tolerance and clean-save tests; loading the user's real config (which carries `selected_activities` and `modifiers.activities`) works.

- [X] T017 [US3] Delete `selected_activities` from `AppConfig` and `TripParams` in `src/ui/app.rs` (~147, 3206) and from every construction/copy site (load_config ~3243, create_default_config ~3289/3321, save_config ~1831, App::new ~1715-1770, resume ~1926, apply trip params ~2769); serde's unknown-field tolerance makes legacy files load
- [X] T018 [US3] [P] Delete `selected_activities` from `TripFile` in `src/config/trip_file.rs` (~18-19) and its construction site in `save_trip_file` in `src/ui/app.rs` (~1867)
- [X] T019 [US3] Update the round-trip tests in `src/ui/app.rs`: drop the activity assertions from `old_config_without_trip_params_loads` (~1036-1039), `full_config_round_trips` (~1063-1072), `auto_quantities_are_not_persisted` (~1101), `trip_file_round_trips_and_reconciles` (~1122), and `multi_category_reconcile_matches_any_membership` (~1002); remove the field from every test literal (including the spec 018 recalculate test's state construction, which uses an empty list)
- [X] T020 [US3] Add the legacy-tolerance and clean-save tests in `src/ui/app.rs` tests: (a) a config YAML literal containing `selected_activities` and `modifiers.activities` parses into `AppConfig` without error; (b) a trip YAML literal containing `selected_activities` parses into `TripFile` without error; (c) a serialized `AppConfig` and `TripFile` contain no `selected_activities`/`activities` key (SC-004)
- [X] T021 [US3] Run the full gate; the new tests pass, zero warnings

**Checkpoint**: Files load clean and save clean, pinned by tests.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T022 Update `TUI_USAGE.md`: remove the Activities field from the welcome screen's fields and navigation tables, remove the activity clauses from the workflow examples, and the config screen's section list becomes travel styles / weather
- [X] T023 Run all four quickstart.md validation scenarios and record the results (Scenario 1 and the TUI parts of 2 via `--force-tui`; Scenarios 3 and 4 via `cargo test`)
- [X] T024 Final gate run: `cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo test`, `cargo fmt --check` all clean; `grep -rni "activit" src/ TUI_USAGE.md` returns nothing
- [X] T025 Add the alignment notes: spec 002 and spec 005 (activities removed; weather is the sole condition multiplier), and a note in spec 015 that its US3 (unknown-activity fallback) is moot with the dimension

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **US2 (Phase 2)**: First — the calculation cut unblocks everything; the UI still compiles with unused activity state until Phase 3 removes it
- **US1 (Phase 3)**: Depends on US2 (the input fields the handlers feed are gone); the bulk of the removal
- **US3 (Phase 4)**: Depends on US1 (the state fields it deletes are the ones US1's inventory enumerates)
- **Polish (Phase 5)**: Depends on all stories

### Within Each User Story

- Full gate after every phase; the `[lints]` table is the orphan-detection net
- US1's hint/help/test-contract update (T014/T015) lands with the handler removal, never after it

### Parallel Opportunities

- T003 is parallelizable with T002 (different files)
- T013 and T018 are parallelizable with their same-file predecessors' tails (different files from app.rs)
- Everything else is sequential: the removal is one connected cut through `src/ui/app.rs`

---

## Implementation Strategy

### MVP First (US2 + US1)

1. Complete Phase 1: baseline
2. Complete US2 (calculation) — the behavioral core
3. Complete US1 (UI + state + hints/help) — the visible removal
4. **STOP and VALIDATE**: quickstart Scenarios 1 and 2 — no activity surface, direct weather rule

### Incremental Delivery

1. US2 → US1 → validate (MVP)
2. US3 → validate (files)
3. Polish (docs, scenarios, final gate, alignment notes)

---

## Notes

- [P] tasks = different files, no dependencies; most of this feature is one connected cut through `src/ui/app.rs`, so tasks are sequential
- The constitution governs: removal not annotation (no `#[allow]`, no `#[serde(skip)]` placeholders), warning-free gate after every phase, tests updated with the behavior
- research.md's 23-touchpoint inventory is the checklist; the `[lints]` table guarantees nothing dangles
- The user's real config (~/.config/pack-liste/config.yaml) carries activity fields; it must load (D2) and is rewritten clean on the next save (EC-005)
- Post-018: the `R` (Recalculate) binding and its test are disjoint from the removal; verify after T016 that `cargo test recalculate` still passes
- Commit after each task or logical group (Conventional Commits per AGENTS.md)
