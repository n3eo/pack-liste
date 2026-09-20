# Research: Remove Activity Options

**Feature**: Remove Activity Options
**Date**: 2026-09-20 (refreshed after spec 018 landed)
**Spec**: [spec.md](spec.md)

Code-level inventory of every activity touchpoint, verified against the
current source (post-018: the calculation body now lives in
`AppDrawState::recalculate_quantities`, and the `R` binding exists in
the hint/help/test surfaces). This is a removal feature: the plan's
job is to know exactly what gets deleted and what the survivors look
like.

## Where activities live today (verified, current line numbers)

| # | Touchpoint | Location | What it is |
|---|-----------|----------|------------|
| 1 | `Focus::Activities` variant | `src/ui/app.rs:53` + `next()`/`prev()` arms (62-63, 69, 73) | Welcome-screen focus cycle member |
| 2 | `AppDrawState.selected_activities` | `src/ui/app.rs:1191` | Selected activity names (runtime state) |
| 3 | `AppDrawState.activity_cursor` | `src/ui/app.rs:1200` | Cursor in the welcome activities list |
| 4 | `ConfigSection::Activities` | `src/ui/app.rs:1234` + `title()`/`next()`/`prev()` arms (1242-1258) | Config screen's third section |
| 5 | `toggle_activity` | `src/ui/app.rs:1991` | Space-toggle handler on the welcome screen |
| 6 | Activity multiplier resolution in `recalculate_quantities` | `src/ui/app.rs:1494-1523` (the spec 018 state-layer method) | Builds `activity_multipliers` (unknown → 1.0) and passes them into the calculation |
| 7 | Left/Right activity-cursor movement | `src/ui/app.rs:2181-2187` | Welcome screen handler arm |
| 8 | Space toggle dispatch | `src/ui/app.rs:2467-2470` | Welcome screen handler arm |
| 9 | `config_section_modifiers(_mut)` Activities arms | `src/ui/app.rs:2619, 2630` | Config screen data access |
| 10 | `reconcile_selections` activity retain | `src/ui/app.rs:2953-2959` | Keeps selected activities valid after modifier edits |
| 11 | `AppConfig.selected_activities` | `src/ui/app.rs:147` | Config-file field (serde) |
| 12 | `TripParams.selected_activities` | `src/ui/app.rs:3206` | Loaded trip parameters |
| 13 | Welcome activities block rendering | `src/ui/app.rs:3509-3557` | The field's draw code (block, list, scroll window) |
| 14 | `params_layout` 5th constraint | `src/ui/app.rs:3401-3411` | `Min(3)` row for the activities field |
| 15 | Welcome hint entry `("Space", "Toggle activity")` | `src/ui/app.rs:3672` (data) + `781` (test's contract copy) | Hint surface data |
| 16 | Help entries mentioning activities | `src/ui/app.rs:3856, 3860, 3863, 3907` | Help screen text |
| 17 | Config screen Activities tab rendering | `src/ui/app.rs:4047-4052, 4079` | Section tab + data match arm |
| 18 | `ModifiersConfig.activities` + `default_activities()` + `activity_multiplier()` | `src/config/modifiers.rs:20-21, 49-54, 83-92` | Modifier kind: field, lookup, defaults |
| 19 | `QuantityCalculationInput.activities` + `.activity_multipliers` | `src/quantity/model.rs:26-27` | Calculation input fields |
| 20 | Activity-priority rule in `calculate` | `src/quantity/calculation.rs:25-33, 46-61` | Activities' average replaces the weather multiplier; notes text |
| 21 | Activity tests | `src/quantity/calculation.rs:113-143` + app.rs tests (1036-1039, 1063-1072, 1101/1122, 337) | `activities_replace_weather_multiplier`, `unknown_activities_contribute_neutral_multiplier`, config/trip round-trip assertions, focus-cycle wrap |
| 22 | `TripFile.selected_activities` | `src/config/trip_file.rs:18-19` | Trip-file field (serde, `#[serde(default)]`) |
| 23 | Trip save/load/restore copies | `src/ui/app.rs:1715/1732/1770 (App::new), 1831 (save_config), 1867 (save_trip_file), 1926 (resume), 2769 (apply trip params)` | selected_activities config ↔ trip ↔ state |

Post-018 note: the `R` (Recalculate) entries added by spec 018 to the
hint data (3657/3664), help (3887), and the extraction tests (696, 760,
770, 3794) are **not** activity touchpoints — they stay untouched.

## Decisions

### D1: Calculation reverts to the direct weather rule
**Decision**: `calculate` applies `style_multiplier * weather_multiplier`
directly; the `effective_weather_multiplier` branch, the
`activity_multipliers`/`activities` input fields, and the
activities-priority notes variant are deleted. Notes always read
"Calculated for {n} weekdays, {style} style, {weather} weather".
**Rationale**: Spec FR-002/US2 — the pre-activity rule; the activity
path is removed with its input, not stubbed.
**Alternatives**: Keep the fields with empty lists (rejected — dormant
code, constitution Principle I).

### D2: Serde tolerance via field removal, not `#[serde(deny)]`
**Decision**: Delete `selected_activities` from `AppConfig`,
`TripParams`, `TripFile`, and `activities` from `ModifiersConfig`.
serde ignores unknown fields by default, so legacy files carrying
`selected_activities:` / `modifiers.activities:` load without error and
the data is dropped on the floor — exactly FR-003/FR-004/US3. New saves
cannot contain the fields because the structs no longer have them.
**Rationale**: Zero new code; the removal itself is the tolerance.
**Alternatives**: `#[serde(skip)]` fields kept for reading (rejected —
that is dormant state and version-compat machinery the spec explicitly
disclaims in its Assumptions).

### D3: Welcome layout shrinks to four fields
**Decision**: `params_layout` drops the fifth constraint; the
activities block rendering (touchpoint 13) is deleted. The remaining
four fields keep their `Length(3)` rows; the freed space goes to the
content area's `Min` constraint (weather becomes the last field).
**Rationale**: FR-006/EC-003 — exactly four fields, cycle
file → weekdays → style → weather → file.
**Alternatives**: Keep a fifth empty row (rejected — dead space).

### D4: Config screen becomes two sections
**Decision**: `ConfigSection::Activities` variant is deleted;
`next()`/`prev()` cycle TravelStyles ↔ Weather; the tab rendering and
`config_section_modifiers(_mut)` lose their Activities arms.
**Rationale**: FR-006/EC-004.
**Alternatives**: None considered — the spec is explicit.

### D5: Hints/help/test contract updated in the same change
**Decision**: The Welcome hint surface loses `("Space", "Toggle
activity")`; help entries at 3856/3860/3863/3907 drop their activity
clauses; the extraction test's expected binding sets (app.rs:743+) and
`help_binding_tokens` are updated in the same change. The
`hint_entries_match_the_contract_binding_sets` test is the guard that
makes a stale activity hint a test failure. The spec 018 `R` entries
in the same surfaces are untouched.
**Rationale**: FR-005/SC-001; spec 016's drift-guard contract.
**Alternatives**: None — the extraction test exists precisely for this.

### D6: Tests: remove activity-specific, extend weather
**Decision**: Delete `activities_replace_weather_multiplier` and
`unknown_activities_contribute_neutral_multiplier` (their subject is
gone); keep/extend the direct-rule tests
(`base_quantity_is_weekdays_times_multipliers` already covers 7×1.5=11).
App tests: the round-trip and `old_config_without_trip_params_loads`
tests drop their activity assertions; add tests asserting a legacy
config/trip YAML with activity fields still parses (US3/SC-003) and
that serialized output contains no activity keys (SC-004). The
focus-cycle test's wrap expectation changes from `Activities` to
`Weather`. The spec 018 recalculate test is untouched in substance (it
uses an empty activity list; after the removal its literals simply
lose the field).
**Rationale**: SC-002/SC-005; constitution Principle III (tests track
behavior) and Principle I (dead tests removed with dead behavior).
**Alternatives**: Keep activity tests against empty inputs (rejected —
they would test nothing real).

### D7: `capitalize_first` stays
**Decision**: The helper remains used by the travel-style and weather
displays (3484, 3502); only the activity call site (3544) goes.
**Rationale**: Verified two other call sites; not orphaned.

### D8: Spec alignment notes
**Decision**: Alignment notes are appended to specs 002 and 005
(activities removed; weather is the sole condition multiplier) and a
note in spec 015 that its US3 (unknown-activity fallback) is moot.
**Rationale**: Spec 017's own Assumptions require them; the project
convention (specs 001/003/005/014) is in-place alignment notes.
**Alternatives**: None.

## Risks and mitigations

- **R1**: Missed touchpoint leaves a dangling reference → compile
  error (the `[lints]` table denies warnings, so any unused import or
  dead code fails the gate). The 23-touchpoint inventory above is the
  checklist.
- **R2**: The user's real config (~/.config/pack-liste/config.yaml)
  carries `selected_activities` and `modifiers.activities` → loads
  fine (D2), and the next save rewrites it without the fields (EC-005).
- **R3**: Trip files saved before the change → same as R2; the resume
  path (app.rs:1926) simply stops copying the field.
- **R4 (post-018)**: The recalculate test and the `R` binding share
  surfaces with the removal — verified above that 018's additions are
  disjoint from the activity touchpoints; the only interaction is the
  `QuantityCalculationInput` literal inside `recalculate_quantities`
  (touchpoint 6), where the two activity fields are deleted.

## No open questions

All spec edge cases (EC-001..EC-005) map directly onto the inventory
above; nothing needs clarification before design.
