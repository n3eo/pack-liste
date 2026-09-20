# Implementation Plan: Implemented-Specs Compliance Review

**Branch**: `015-implemented-specs-compliance` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/015-implemented-specs-compliance/spec.md`

## Summary

Close the six gaps found by the claim-by-claim compliance review of the implemented specs (001–005): (1) a TUI toggle for the auto/fixed quantity mode in the item edit dialog, (2) a packed-state indicator on the compiled list's category headers, (3) the uniform 1.0 fallback for unknown modifier names (activities are currently silently dropped), (4) a complete help screen listing every active binding, (5) an input-handling test net at the handler layer, and (6) multi-category items with one shared selection/packed state. The work is concentrated in `src/ui/app.rs` (edit dialog, drawing, key handling, help), `src/quantity/calculation.rs` (fallback semantics), and `src/models/item.rs` (category membership); the test net lands next to the handlers it covers.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29, serde/serde_yaml, anyhow, thiserror, clap, dirs, chrono (unchanged — no new dependencies)

**Storage**: YAML config and trip files (config gains per-item category lists; format change is additive with defaults for existing single-category entries)

**Testing**: `cargo test`; the quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo fmt --check`) must stay warning-free per the `[lints]` table

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: N/A for UI stories (existing render loop); calculation stories keep the deterministic, unit-tested rule

**Constraints**: Warning-free gate (constitution Principle II); no new dependencies; behavior-preserving except the specified gap fixes; multi-category is a data-model change and must keep trip-file reconciliation working

**Scale/Scope**: ~3,900 lines of own source; 6 user stories; heaviest story (multi-category) touches the item model, tree grouping, selection toggles, edit dialog, and reconciliation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0 (`.specify/memory/constitution.md`):

- **Principle I (No Unused Code)**: satisfied — all stories add reachable behavior; no scaffolding.
- **Principle II (Zero Warnings and Errors)**: satisfied — the `[lints]` table denies warnings; every task ends with the gate commands clean.
- **Principle III (Documented and Tested)**: this feature is itself largely *about* Principle III (help completeness = documentation, input tests = testing). Every story ships with its tests (FR-005) and the help screen is updated in the same change as any binding change (FR-004).
- Quality standards table and workflow gates apply as usual; no violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/015-implemented-specs-compliance/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   └── input-and-help.md  # Key-binding and help-screen contract
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # unchanged
├── app.rs               # unchanged
├── config/
│   ├── modifiers.rs     # unchanged (fallback fix is at the call site)
│   └── trip_file.rs      # reconcile: match items by name across category lists
├── models/
└── item.rs              # category: String -> categories: Vec<String> (US6)
└── ui/
    └── app.rs           # US1 edit-dialog mode toggle; US2 category indicator;
                         # US3 unknown-activity fallback; US4 help screen;
                         # US5 input tests; US6 tree grouping + selection semantics
```

**Structure Decision**: Single-project structure (unchanged). All six stories land in the existing modules; no new modules. The test net (US5) lives in `#[cfg(test)] mod tests` inside `src/ui/app.rs` next to the handlers, per the existing convention.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations. Not applicable.
