# Implementation Plan: Code Review Remediation

**Branch**: `022-code-review-refactoring` | **Date**: 2026-09-20 (updated post-convergence) | **Spec**: [spec.md](spec.md)

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
lands with the full quality gate green and all existing tests passing
unmodified. The per-frame state clone is kept as a documented trade-off
(research R8).

**Status**: US1–US6 implemented (T001–T039 complete); behavior verified
byte-identical to the pre-refactor binary via PTY screen-diff. The
convergence review (Phase 9, T040–T042) closes three remaining partial
gaps: config-screen status rendering, status clearing on transitions, and
doc comments on the remaining undocumented public items.

## Technical Context

**Language/Version**: Rust 1.98 (edition 2024)

**Primary Dependencies**: ratatui 0.30.2, crossterm 0.29, serde/serde_yaml,
anyhow, thiserror, clap 4, dirs 7. Removed by this feature: chrono,
tracing, tracing-subscriber, mockall (dev).

**Storage**: YAML files — input config (`config.yaml`, via `AppConfig`/
`SavedItem`) and trip file (via `TripFile`/`TripItem`, human-editable,
reconciled on load). Neither format contains the dead fields or the
`editable` field, so removals/renames are format-neutral (research R1, R2).

**Testing**: `cargo test` — 41 tests (33 original UI tests moved to
`src/ui/tests/` with assertions unmodified, 6 in
`src/quantity/calculation.rs`, 2 new status-line tests). The help/hint
contract tests encode the spec 016 binding contract and guard the split.
Manual smoke pass per step (research R12); PTY screen-diff against the
pre-refactor binary proves behavior preservation.

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
| I. No Unused Code | **Fixed by this feature** | Dead fields (F5), `chrono`/`tracing`/`mockall` dependencies, and stale docs removed (T018–T020, T030, T033). Verified: zero references remain. |
| II. Zero Warnings and Errors | **Pass** | Gate green throughout implementation and after convergence review. |
| III. Documented and Tested | **Partial — closed by Phase 9** | `docs/quantity-calculation.md` rewritten truthfully (T033); tests adjacent to their modules (T009). Remaining: 62 public items lack doc comments — T042 closes this. |

Post-Phase-1 re-check: the design adds no unused code (the status line
is exercised by new tests; `StatusKind::Info` was dropped rather than
kept unused), introduces no warnings, and ships with tests + updated
docs. **Gate passes once T042 lands.**

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
    │   ├── welcome.rs  # welcome screen, main screen, and dialog handlers
    │   ├── dialogs.rs  # add/edit dialog state operations (save, field cycle)
    │   ├── config.rs   # config screen + modifier editor
    │   └── editor.rs   # raw config editor
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
`screens/` sub-trees mirror each other for name-based navigation (the
welcome/main/dialog key handlers share `handlers/welcome.rs` because
they share the `App`-method plumbing; the per-screen *rendering* stays
one module per screen). No new top-level architecture; `src/app.rs` stays
the thin shim. Tests moved into `src/ui/tests/` adjacent to their
subjects (FR-010).

## Convergence Refinements (Phase 9)

The convergence review found three partial gaps; the design decisions
for closing them (all traceable to existing artifacts):

1. **Config-screen status rendering (T040)**: research R6 specified the
   status line renders "in the main/config screen footer area"; the
   implementation rendered only the main screen. Close by rendering
   `state.status` in `src/ui/screens/config.rs`'s footer, styled
   identically to the main screen (red/bold for `StatusKind::Error`).
   The config screen's layout already has an instructions area; the
   status line takes precedence over the hint line when set, matching
   the raw editor's existing error-footer pattern.
2. **Status clearing on transitions (T041)**: data-model V3 requires
   the status "does not persist across a mode change (cleared on
   transition)". Clear `status` in `handle_escape` (screen and mode
   exits) and where dialogs close (`input_mode = InputMode::Normal`).
   A successful save/recalculate already clears it
   (`save_config_or_report`/`update_quantities_or_report`).
3. **Doc comments (T042)**: Constitution III requires doc comments on
   public modules, types, and functions. 62 public items lack them
   (verified by scan). Add `///` doc comments stating purpose and
   behavior — no code changes, no behavior changes.

All three keep the gate green and the 41 tests passing; none change
user-visible behavior except making already-set status messages visible
on the config screen and clearing stale ones.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None — the constitution violations found by the review are *removed* by this feature, not introduced. The per-frame state clone (F4) is not a constitution violation; it is documented as an accepted trade-off in research R8 per FR-012. Principle III's remaining doc-comment gap is closed by T042, not justified. | | |
