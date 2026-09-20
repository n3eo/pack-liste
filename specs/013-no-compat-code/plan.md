# Implementation Plan: Remove Version-Compatibility Code

**Branch**: `013-no-compat-code` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Remove every compatibility mechanism from the load paths: optional
persisted fields with fallback defaults, the legacy single-`category`
lifting, and the tests that pin old formats. Both data files (input
config, trip file) load exactly the current format — a file that does
not match fails with a clear error naming the file, and a rejected file
is never modified. First-run creation of a fresh config stays (EC-001:
that is not compatibility). The affected specs' superseded
compatibility requirements get supersession notes (US4).

**Sequencing note (critical)**: This feature re-baselines the file
formats. It MUST land **after** 010 (per-item modifiers) and 012
(trip-file quantities) — both add fields whose pre-feature loading
relies on the default-substitution this feature removes. Order:
010 → 012 → 013. After 013, the formats are: config = the full
current schema (all fields required), trip file = with quantities
(required). Old files are regenerated or hand-edited per the
documented format (US3), never migrated.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: serde/serde_yaml (unchanged — `deny_unknown_fields`
and required fields are serde-native)

**Storage**: Both files become strict: every field required as written;
unknown fields rejected (`#[serde(deny_unknown_fields)]`, EC-002);
no `#[serde(default)]` on persisted structs; no legacy-key lifting.

**Testing**: `cargo test` — the compatibility tests are removed with
the code they pin (EC-004: `old_config_without_trip_params_loads`,
`legacy_files_with_activity_fields_load_and_save_clean`, the legacy
category-lifting paths). New tests: a current-format file loads; a
file missing a field fails with a clear error; a file with an unknown
field fails; a rejected file is byte-identical afterward.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; a load failure must never produce
silently-substituted running state (EC-003); a rejected file is never
written (US2); first-run creation stays (EC-001); no new
dependencies.

**Scale/Scope**: `src/ui/config_io.rs` (AppConfig/SavedItem strict),
`src/config/trip_file.rs` (TripFile/TripItem strict), the load error
paths in `src/ui/input.rs`, test removals + additions, spec
supersession notes, docs updated to the current format only.

## Constitution Check

- **Principle I**: satisfied — this feature *is* dead-code removal
  (the compat machinery and its tests).
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — the format documentation describes
  only the current format (US3/AC2); the strict-load contract gets
  tests.
- No violations.

## Project Structure

```text
src/
├── ui/
│   ├── config_io.rs     # AppConfig/SavedItem: required fields,
│   │                    #   deny_unknown_fields, custom Deserialize
│   │                    #   impl removed (the legacy lifting dies with it)
│   ├── input.rs         # load errors surface (status line / exit);
│   │                    #   a rejected file is never written
│   └── tests/
│       └── config_io.rs # compat tests removed; strict-load tests added
├── config/
│   └── trip_file.rs     # TripFile/TripItem: required fields,
│                        #   deny_unknown_fields
└── docs/
    └── (format docs)    # current format only, no version history
```

**Structure Decision**: The custom `Deserialize` impl on `SavedItem`
exists solely for legacy-category lifting — it is removed; the derive
handles the strict format. `deny_unknown_fields` goes on all four
persisted structs.

## Phase 0 Research

- **D1 — Compat inventory (verified in code)**: `AppConfig` has five
  `#[serde(default...)]` attributes (`config_io.rs:16-25`); `SavedItem`
  has a hand-written `Deserialize` that lifts a legacy single
  `category` key (`config_io.rs:34,75,86`); `TripFile.items` and
  `TripItem.packed` have `#[serde(default)]` (`trip_file.rs`); the
  tests `old_config_without_trip_params_loads` and
  `legacy_files_with_activity_fields_load_and_save_clean` pin old
  formats. All are removed.
- **D2 — First-run vs compat (EC-001)**: `App::new` creates a default
  config when the file does not exist — kept. The removal targets
  existing-but-nonmatching files only.
- **D3 — Failure surface (EC-003)**: a config load failure at startup
  exits with the error (anyhow propagates from `App::new`); a trip
  file passed as the argument that fails to load exits with the error
  rather than silently starting a new trip that would overwrite the
  file on exit (US2/AC3). The raw editor's apply path already reports
  parse errors in its footer — unchanged.
- **D4 — Supersession notes (US4)**: specs 004 (FR-006's
  recalculate-on-load), 010 (FR-008 pre-feature files), 012 (EC-001
  pre-feature trip files), and any other spec demanding old-file
  loading get a note: "Superseded by spec 013 for format
  compatibility; the program loads only the current format."
- **D5 — serde strictness**: `#[serde(deny_unknown_fields)]` rejects
  unknown keys (EC-002); removing `#[serde(default)]` makes missing
  fields load errors. Both produce serde errors that name the file
  via the existing `with_context` wrappers.

## Phase 1 Design

### Data model

Unchanged shapes; strictness only:

```text
AppConfig  { categories, items, weekdays, travel_style, weather, modifiers, last_trip_file }  // all required, deny unknown
SavedItem  { name, categories, quantity?, auto_quantity }  // derive-based, deny unknown
TripFile   { weekdays, travel_style, weather, items }     // all required, deny unknown
TripItem   { name, category, packed, quantity }           // all required (post-012), deny unknown
```

(`SavedItem.quantity` stays optional-in-format because the *current*
format legitimately omits it for automatic items — that is a format
rule, not compatibility.)

### Contracts

- Load: current format only; failures name the file; no substitution.
- Save: unchanged (writes the current format).
- Rejected files: byte-identical on disk after a failed load.

### Quickstart

- A current-format config and trip file load cleanly.
- Remove a field from either file → load fails, error names the file,
  the file is unchanged on disk.
- Add an unknown field → load fails.
- Delete the config → first run creates a fresh one (EC-001).
- Old-format files: regenerate or hand-edit per the documented format.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
