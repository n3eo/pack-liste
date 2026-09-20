# Implementation Plan: Remove Activity Options

**Branch**: `017-remove-activities` | **Date**: 2026-09-20 (refreshed after spec 018 landed) | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/017-remove-activities/spec.md`

## Summary

Remove the activity dimension entirely: the welcome screen's activities field, the config screen's third section, the activity multipliers and the activity-priority calculation rule, and all activity persistence (config file, trip file). The calculation reverts to spec 005's pre-activity rule — weather is the sole condition multiplier. Legacy files carrying activity fields load via serde's default unknown-field tolerance and are rewritten clean on the next save. Hints, help, and the extraction tests are updated in the same change so no stale activity reference survives.

**Refresh note (post-018)**: spec 018 has landed — the calculation body now lives in the state-layer method `AppDrawState::recalculate_quantities` (src/ui/app.rs:1484), and the `R` (Recalculate) binding exists in the hint/help/extraction surfaces. The removal edits the method's body (deleting the two activity input fields) and leaves every `R` entry untouched. All line references in research.md are re-verified against the current source.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29, serde/serde_yaml (unchanged — no new dependencies)

**Storage**: YAML config and trip files (fields removed; no format machinery added)

**Testing**: `cargo test` — the spec 015/016/018 state-layer and extraction tests are the net (39 tests at baseline); activity-specific tests are removed with their behavior, legacy-tolerance tests are added. The quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo fmt --check`) must stay warning-free per the `[lints]` table.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: None — a removal; the calculation gets simpler.

**Constraints**: Warning-free gate (constitution Principle II); removal not annotation (Principle I — no `#[allow]`, no `#[serde(skip)]` placeholders); hints/help/extraction tests updated in the same change as the binding removal (Principle III, spec 016's drift-guard contract).

**Scale/Scope**: 23 verified activity touchpoints across `src/ui/app.rs` (~93 references), `src/config/modifiers.rs`, `src/quantity/model.rs`, `src/quantity/calculation.rs`, `src/config/trip_file.rs` — the full inventory is in [research.md](research.md).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0 (`.specify/memory/constitution.md`):

- **Principle I (No Unused Code)**: satisfied and strengthened — the feature *is* removal; every deleted touchpoint is enumerated in research.md. No dormant fields, no skip-annotated placeholders, no kept-for-later code.
- **Principle II (Zero Warnings and Errors)**: satisfied — the `[lints]` table denies warnings, so any orphaned reference fails the gate; every task ends with the gate clean.
- **Principle III (Documented and Tested)**: satisfied — hints/help/TUI_USAGE.md updated in the same change; activity tests removed with their behavior, legacy-tolerance and direct-rule tests added; the extraction tests are updated so a stale activity entry is a test failure.
- No violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/017-remove-activities/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output: the 23-touchpoint inventory
├── data-model.md                  # Phase 1 output: removed fields, surviving shapes
├── quickstart.md                  # Phase 1 output: validation scenarios
├── contracts/
│   └── bindings-and-formats.md    # Phase 1 output: post-removal bindings + file formats
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── ui/
│   └── app.rs            # Focus::Activities, ConfigSection::Activities,
│                         #   selected_activities/activity_cursor state,
│                         #   toggle_activity, handler arms, welcome field
│                         #   rendering, config tab, hints/help entries,
│                         #   extraction-test contract copies, trip
│                         #   save/load/restore copies, and the activity
│                         #   fields inside recalculate_quantities
├── config/
│   ├── modifiers.rs      # ModifiersConfig.activities, default_activities(),
│   │                     #   activity_multiplier()
│   └── trip_file.rs      # TripFile.selected_activities
└── quantity/
    ├── model.rs          # QuantityCalculationInput.activities,
    │                     #   .activity_multipliers
    └── calculation.rs    # activity-priority branch, notes variant,
                          #   activity tests
```

**Structure Decision**: Single-project structure (unchanged). All changes land in the five files above; no new modules, no new dependencies. The removal is mechanical against the research.md inventory.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations. Not applicable.
