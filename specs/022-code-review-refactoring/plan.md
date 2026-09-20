# Implementation Plan: Code Review Remediation

**Branch**: `022-code-review-refactoring` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/022-code-review-refactoring/spec.md`

## Summary

Behavior-preserving remediation of the nine verified code-review findings
(F1–F9 in the spec): split the 4,917-line `src/ui/app.rs` monolith into a
concern-separated module tree, decompose the ~473-line input handler into a
two-level dispatcher with per-screen handlers, consolidate four duplication
sites, remove five dead model fields plus the `chrono`, `tracing`, and
`mockall` dependencies, rename the misleading `editable` field and tighten
visibility, surface the six swallowed save/recalculate errors via a status
line, and rewrite the stale quantity-calculation documentation. Every step
lands with the full quality gate green and all 39 existing tests passing
unmodified. The per-frame state clone is kept as a documented trade-off
(research R8).

## Technical Context

**Language/Version**: Rust 1.98 (edition 2024)

**Primary Dependencies**: ratatui 0.30.2, crossterm 0.29, serde/serde_yaml,
anyhow, thiserror, clap 4, dirs 7. Removed by this feature: chrono,
tracing, tracing-subscriber, mockall (dev).

**Storage**: YAML files — input config (`config.yaml`, via `AppConfig`/
`SavedItem`) and trip file (via `TripFile`/`TripItem`, human-editable,
reconciled on load). Neither format contains the dead fields or the
`editable` field, so removals/renames are format-neutral (research R1, R2).

**Testing**: `cargo test` — 39 tests (33 in `src/ui/app.rs`, 6 in
`src/quantity/calculation.rs`). The help/hint contract tests encode the
spec 016 binding contract and guard the split. Manual smoke pass per step
(research R12).

**Target Platform**: Linux terminal (TUI binary `pack-liste`)

**Project Type**: CLI/TUI desktop application (single binary crate)

**Performance Goals**: No change — the per-frame `AppDrawState` clone is
retained as a documented trade-off (FR-012, research R8).

**Constraints**: Behavior-preserving (FR-011): all 39 tests pass
unmodified; persisted YAML formats remain loadable; the spec 016
binding contract holds. Warning-free gate (constitution Principle II).
`Cargo.lock` never hand-edited.

**Scale/Scope**: One file of 4,917 lines split into ~20 modules; six user
stories; no new functionality.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. No Unused Code | **Violated today, fixed by this feature** | The dead fields (F5), unused `chrono`/`tracing`/`mockall` dependencies, and stale docs are exactly what US3/US5/US6 remove. The feature is itself the remediation. |
| II. Zero Warnings and Errors | **Pass** | Gate is green today; every step must keep it green. |
| III. Documented and Tested | **Violated today, fixed by this feature** | Stale `docs/quantity-calculation.md` (F8) is rewritten by US6; tests move adjacent to their modules (FR-010). |

Post-Phase-1 re-check: the design adds no unused code (the status line
is exercised by new tests; `HandlerOutcome` replaces the boolean return),
introduces no warnings, and ships with tests + updated docs. **Gate
passes.**

## Project Structure

### Documentation (this feature)

```text
specs/022-code-review-refactoring/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output — 12 verified decisions (R1–R12)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── module-structure.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

Target structure after US1 (research R3):

```text
src/
├── main.rs             # entry: CLI parse, single error report (US5)
├── app.rs              # thin entry shim (unchanged role)
├── config/
│   ├── mod.rs
│   ├── modifiers.rs
│   └── trip_file.rs
├── models/
│   ├── mod.rs
│   ├── item.rs         # minus id/base_quantity; editable → selected (US3/US4)
│   ├── packing_list.rs  # minus id/name/file_path; chrono removed (US3)
│   └── weekdays.rs
├── quantity/
│   ├── mod.rs
│   ├── calculation.rs
│   └── model.rs
└── ui/
    ├── mod.rs          # re-exports App; declares submodules
    ├── state.rs        # AppDrawState, Screen, Focus, FocusSide, InputField,
    │                   # InputMode, AppState, StatusMessage, cursor ops
    ├── input.rs        # two-level key dispatcher (US2)
    ├── handlers/       # per-screen key handlers
    │   ├── mod.rs
    │   ├── welcome.rs
    │   ├── main.rs
    │   ├── dialogs.rs
    │   ├── config.rs
    │   └── editor.rs
    ├── draw.rs         # App::draw dispatch + shared draw helpers
    ├── screens/         # per-screen rendering
    │   ├── mod.rs
    │   ├── welcome.rs
    │   ├── main.rs
    │   ├── help.rs
    │   ├── config.rs
    │   └── editor.rs
    ├── hints.rs        # hint entries, help tokens
    ├── help.rs         # help screen content
    ├── config_io.rs    # AppConfig, SavedItem, load/save/default
    └── tests/           # 33 tests split alongside the code they test
        ├── mod.rs
        ├── state.rs
        ├── input.rs
        ├── hints.rs
        └── config_io.rs
```

**Structure Decision**: Concern-separated modules under the existing
`src/ui/` tree, mirroring the internal layout the monolith already has
(state at top, handlers middle, drawing bottom). `handlers/` and
`screens/` sub-trees mirror each other for name-based navigation. No new
top-level architecture; `src/app.rs` stays the thin shim. Tests move into
`src/ui/tests/` adjacent to their subjects (FR-010).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None — the constitution violations found by the review are *removed* by this feature, not introduced. The per-frame state clone (F4) is not a constitution violation; it is documented as an accepted trade-off in research R8 per FR-012. | | |
