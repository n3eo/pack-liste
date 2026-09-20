# Implementation Plan: Warning-Free Codebase via Static Analysis

**Branch**: `009-clippy-clean-codebase` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/009-clippy-clean-codebase/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Bring the codebase to a warning-free state under `cargo check` and `cargo clippy` and guard that state. The current inventory is 12 compiler warnings and 37 clippy warnings (normal and test builds). The bulk are dead code from the superseded packing-list-management design (services, config model, error enums, unused model methods) — these are removed per FR-004. The rest are mechanical/idiomatic fixes (collapsible ifs, manual clamps, needless mut/parens, map-value iteration, while-let loop) applied in place. Two `too_many_arguments` warnings on the popup draw functions are resolved by grouping the popup fields into a small struct rather than suppressing. A workspace lint table (`[lints]` with warnings denied) enforces the state going forward, and the verification commands are documented in AGENTS.md.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29, serde/serde_yaml, anyhow, thiserror, clap, dirs, chrono (unchanged — no new dependencies)

**Storage**: YAML config and trip files (unchanged)

**Testing**: `cargo test` (11 existing tests must keep passing); verification via `cargo check`, `cargo clippy`, `cargo clippy --tests`

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: N/A — behavior-preserving cleanup; no runtime changes

**Constraints**: Behavior-preserving; no new dependencies; no `unsafe`; fixes must not silence lints except scoped+documented suppressions (target: zero suppressions)

**Scale/Scope**: ~3,900 lines of own source across src/main.rs, src/app.rs, src/ui/, src/config/, src/models/, src/quantity/, src/services/

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution (`.specify/memory/constitution.md`) is an unfilled template with no ratified principles; no gates are defined. The repository's AGENTS.md conventions apply instead and align with this feature: "Run `cargo fmt`; `cargo clippy` must pass with no warnings", "No `unwrap()` in non-test code without a justifying comment", "Don't introduce new dependencies". No violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/009-clippy-clean-codebase/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   └── verification.md  # The quality-gate contract this feature establishes
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point; gains the workspace lint table
├── app.rs               # Run loop (while-let fix)
├── config/
│   ├── mod.rs           # model module removed (dead)
│   ├── error.rs         # REMOVED (dead: ConfigError never used)
│   ├── modifiers.rs     # unchanged
│   └── trip_file.rs     # collapsible-if fix
├── models/
│   ├── item.rs          # toggle_packed/set_quantity removed or wired (dead methods)
│   ├── packing_list.rs  # dead helper methods removed
│   └── weekdays.rs      # unchanged
├── quantity/
│   ├── calculation.rs   # unchanged (tests stay green)
│   ├── error.rs         # REMOVED (dead: QuantityError never used)
│   └── model.rs        # unchanged
├── services/            # REMOVED entirely (dead: PackingService, ItemService)
└── ui/
    ├── app.rs           # Bulk of the fixes: collapsible ifs, clamps, popup arg struct
    └── error.rs         # REMOVED (dead: UiError never used)

tests/                   # empty directories from scaffolding; removed or left as-is
```

**Structure Decision**: Single-project structure (unchanged). The cleanup removes the dead `services/` module and three dead error enums rather than keeping them annotated; the module tree shrinks accordingly. No new modules are added; the only structural addition is a popup-state struct inside `src/ui/app.rs` to satisfy `too_many_arguments`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations. Not applicable.
