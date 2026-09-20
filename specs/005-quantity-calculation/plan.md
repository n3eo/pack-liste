# Implementation Plan: Quantity Calculation

**Branch**: `005-quantity-calculation` | **Date**: 2026-09-19 | **Spec**: [spec.md](../spec.md)

**Input**: Feature specification from `/specs/005-quantity-calculation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

> **Alignment note (2026-09-20)**: This plan predates several design changes
> made during implementation. The shipped application differs as follows:
> the `ui/views/` module split was not carried out (the TUI lives in
> `src/ui/app.rs`); packing-list management (naming, JSON export/import via
> `PackingService`) was superseded by the trip-file flow (see spec 004) and the
> corresponding service code is currently unused; quantities are persisted as
> YAML (config and trip files), not JSON. The quantity calculation itself
> (`src/quantity/`) matches this plan.

## Summary

The implementation will update the quantity calculation logic to consider input weekdays, travel style, weather input, and activities. The system will adjust item quantities dynamically based on these inputs, ensuring users pack appropriately for their trips. The implementation will be done in Rust using the Ratatui framework, with modular separation of views, quantity calculation, and error handling using `thiserror`.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**:
- Ratatui (TUI framework)
- Thiserror (error handling)
- Serde (serialization for config/export)
- Anyhow (for error handling in main functions)

**Storage**: Configuration and user data stored in `.config/pack-liste/` directory (JSON/YAML files).

**Testing**: Cargo test for unit and integration tests.

**Target Platform**: Terminal-based CLI/TUI application (Linux, macOS, Windows with WSL).

**Project Type**: CLI/TUI application.

**Performance Goals**: Real-time updates (<500ms) for quantity recalculations.

**Constraints**:
- Offline-capable (no external API dependencies).
- Memory usage <100MB.
- No unsafe Rust code.

**Scale/Scope**: Single-user application with support for multiple packing lists.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

No violations detected. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/005-quantity-calculation/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point for the CLI/TUI application
├── app.rs               # Main application state and lifecycle
├── config/              # Configuration loading and management
│   ├── mod.rs
│   ├── model.rs         # Configuration models (e.g., PackingListConfig)
│   └── error.rs         # Configuration-related errors
├── quantity/            # Quantity calculation module
│   ├── mod.rs
│   ├── model.rs         # Models for quantity calculation inputs
│   ├── calculation.rs   # Core quantity calculation logic
│   └── error.rs         # Quantity calculation errors
├── ui/                  # TUI views and interactions
│   ├── mod.rs
│   ├── app.rs           # Main TUI application
│   ├── views/           # Separate view modules
│   │   ├── input.rs     # Input views (weekdays, travel style, weather, activities)
│   │   ├── list.rs      # Packing list view
│   │   └── summary.rs   # Summary/packing progress view
│   └── error.rs         # UI-related errors
├── models/              # Core data models
│   ├── mod.rs
│   ├── packing_list.rs  # PackingList model
│   └── item.rs          # Item model
└── error.rs             # Global error handling and reporting

tests/
├── contract/            # Contract tests
├── integration/         # Integration tests
└── unit/                # Unit tests
```

**Structure Decision**: Single-project structure with modular separation for views, quantity calculation, and configuration. This aligns with the requirement to split files into sensible chunks and separate different concerns.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
