# Implementation Plan: Persist Quantities in the Trip File

**Branch**: `012-trip-quantities-persisted` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Store each selected item's quantity in the trip file alongside its
name, category, and packed state, making the output file a complete,
self-contained snapshot of the trip. On resume, the saved quantities
are restored as-is — not recalculated — so the file is authoritative
for its own trip. Hand edits to quantities in the file are honored.
The input config file is untouched (fixed quantities only, as today).

**Sequencing note**: This plan composes with the amended spec 018
(manual authority, no auto-recalculation): 018's FR-008 already
requires "a resumed trip MUST restore saved quantities without
recalculating" — this feature provides the storage that makes that
possible. Land 012 **before or together with** 018; if 018 lands
first, its resume-restore requirement is satisfied by this feature.
Also sequenced **before** 013 (remove compat code): the optional
`quantity` field on load (EC-001) is compat machinery 013 will remove
when it re-baselines the format.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29, serde/serde_yaml
(unchanged)

**Storage**: The trip file (`TripItem` in `src/config/trip_file.rs`)
gains `quantity: Option<u32>` (serde default None, skipped when None on
write — pre-feature files load unchanged, EC-001). Save writes every
selected item's current quantity; load restores it when present and
leaves the item's quantity untouched (the load path stops
recalculating resumed items — see D1).

**Testing**: `cargo test` — 42 existing tests; new tests: trip-file
round-trip with quantities, resume-restores-saved-quantities (change a
modifier between save and resume — the saved numbers win),
hand-edit-honored, pre-feature-file-loads (no quantity key).

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; the input config's quantity
behavior is untouched (US4); reconciliation (spec 004) applies
unchanged to quantities (EC-002 — quantities survive for items that
reconcile); no new dependencies.

**Scale/Scope**: One field on `TripItem`, the save path writes it, the
load path restores it, the resume path stops recalculating; ~4 tests.

## Constitution Check

- **Principle I**: satisfied — the field is written and read.
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — `docs/quantity-calculation.md`'s
  storage section updated in the same change; tests ship with the
  feature.
- No violations.

## Project Structure

```text
src/
├── config/
│   └── trip_file.rs     # TripItem gains quantity: Option<u32>;
│                        #   save writes it; reconcile preserves it
├── ui/
│   ├── input.rs         # save_trip_file writes quantities;
│   │                    #   apply_trip_file_path / App::new restore
│   │                    #   them and stop recalculating on resume
│   └── tests/
│       └── config_io.rs # round-trip + resume tests
└── docs/
    └── quantity-calculation.md  # storage section updated
```

**Structure Decision**: The field lives on `TripItem` (the trip file's
item), not on the catalog's `SavedItem` — the input/compiled separation
(US4) is structural.

## Phase 0 Research

- **D1 — Resume must stop recalculating (verified)**: the current load
  path (`apply_trip_file_path` in `src/ui/input.rs`) restores packed
  state and then calls `update_quantities()`, which recalculates every
  automatic item — overwriting anything the file carried. With
  quantities stored, the resume path restores them and skips the
  recalculation; only a fresh list (no trip file) calculates on load
  (FR-008 of spec 018, US2/AC1 here).
- **D2 — Reconciliation composes**: `TripFile::reconcile` drops items
  gone from the catalog and remaps categories; it must also carry the
  quantity through for items that survive (EC-002). The quantity is
  part of the trip item's data, moved with it.
- **D3 — Option<u32> on the wire**: `quantity` is optional on load
  (pre-feature files, EC-001) and skipped when None on write. Once
  spec 013 lands, the optionality is revisited (013 makes the field
  required — files are regenerated, not migrated).
- **D4 — Where the quantity comes from at save time**: the in-memory
  `Item.quantity` (whatever the user sees — calculated, fixed, or
  manually set). Save is a snapshot; no calculation happens at save.

## Phase 1 Design

### Data model

```text
TripItem { name, category, packed, quantity: Option<u32> }
```

Save: `quantity: Some(item.quantity)` for every selected item. Load:
restore into the matching catalog item; automatic items keep the
restored value (no recalculation on resume). Reconcile: quantity
moves with the item.

### Contracts

- The trip file remains human-editable YAML; the quantity key is
  documented in `docs/quantity-calculation.md`'s storage section.
- The input config's quantity behavior is unchanged (US4) — verified
  by the existing config round-trip tests passing unmodified.

### Quickstart

- Compile a list, save, read the trip file — every item carries its
  quantity.
- Change a multiplier in the config, resume the trip — the saved
  quantities are shown, not recalculated.
- Hand-edit a quantity in the file, resume — the edited value appears
  and survives the next save.
- A pre-feature trip file (no quantity keys) loads and calculates as
  before.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
