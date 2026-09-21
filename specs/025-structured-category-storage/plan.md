# Implementation Plan: Label-Based Item Storage (revision 3 — singletons and chips)

**Branch**: `025-structured-category-storage` | **Date**: 2026-09-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/025-structured-category-storage/spec.md` (revision 3)

## Summary

Replace the category model with labels: items carry a `labels` list (the old "X: Y" splits into the labels "X" and "Y"), the list is deduplicated by (name, labels), and the UI groups flat — one header per label, an item appearing under every label it carries. A multi-label item can be a "singleton": its selection and packed toggles synchronize across all its labels (one state each); non-singletons toggle per occurrence. Every item renders its labels as grey chips behind its name — dark grey (`DarkGray`) when the occurrence's state matches the item's, light grey (`Gray`) when it differs. The old format is rejected (strict single format); the migration is one-time and external.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30 (Color::Gray / Color::DarkGray both exist — verified), crossterm 0.29, serde/serde_yaml (unchanged)

**Storage**: The configuration file (`AppConfig`/`SavedItem`) and the trip file (`TripItem`) change schema: `labels: Vec<String>` replaces `categories`; `singleton: bool` joins the item (optional-in-format with a default, see research R4). Strict per spec 013: `deny_unknown_fields` stays.

**Testing**: `cargo test` — the schema tests are rewritten (strict load, dedup on load, round trip, old-format rejection); the state tests gain the singleton toggle semantics and the chip rendering model; PTY checks for the flat grouping, the synchronized toggles, the chips, and scrolling at both extremes.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; the old format MUST be rejected (FR-004); the collapse/scroll behaviors are the contract (FR-008); the singleton semantics are exact (FR-011/FR-012); the migration is one-time and external (FR-010).

**Scale/Scope**: The model (`Item`), the two schemas, the flat row model, the occurrence-state layer, the chip rendering, the dialog's label assignment + singleton toggle, the reconcile, the dedup, and the tests. ~15 files, no new modules.

## Constitution Check

- **Principle I** (no unused code): satisfied — `category_layers`, `primary_category`, `in_category`, the group/sub-category row variants, and the multi-membership machinery all die with the category model; nothing is left annotated.
- **Principle II** (zero warnings): satisfied — gate green per task.
- **Principle III** (documented and tested): satisfied — the label schema, the singleton semantics, and the chip rendering are documented and pinned by tests; `TUI_USAGE.md` covers the label format and the singleton.
- No violations.

## Project Structure

### Documentation (this feature)

```text
specs/025-structured-category-storage/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (the label schema + occurrence state)
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (the yaml formats + the singleton semantics)
└── tasks.md             # /speckit-tasks output
```

### Source Code (repository root)

```text
src/
├── models/
│   └── item.rs          # Item: labels: Vec<String> + singleton: bool;
│                        #   the category helpers removed
├── config/
│   └── trip_file.rs     # TripItem: labels + singleton; reconcile by name+labels
├── ui/
│   ├── config_io.rs     # SavedItem: labels + singleton; AppConfig: no
│   │                    #   top-level categories; dedup on load
│   ├── state.rs         # the flat label row model; the occurrence
│   │                    #   state for non-singletons; the synchronized
│   │                    #   toggles; collapse/select/recalc per label
│   ├── screens/main.rs  # the flat grouping; the grey chips behind
│   │                    #   the name; the existing scroll contract
│   ├── handlers/
│   │   └── dialogs.rs   # the label assignment + the singleton toggle
│   └── tests/           # schema, dedup, singleton, chip, and behavior tests
└── TUI_USAGE.md         # the label yaml + the singleton + the chips
```

**Structure Decision**: The labels live on `Item` (the in-memory truth), mirrored into `SavedItem` and `TripItem`. The occurrence state (per-label toggles for non-singletons) is a session-side map in `AppDrawState` — the persisted files store the item's own state only (research R3). The chips render in both panels' item rows, sharing one helper.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| The occurrence-state map (a per-session (item, label) → toggle map) | Non-singleton multi-label items need independent per-occurrence toggles (FR-012); the persisted format stores one item state | Storing per-label state in the yaml — rejected: it complicates the human-editable format for a session-only concern; the load normalizes to the item's state |
