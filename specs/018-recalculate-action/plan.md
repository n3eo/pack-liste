# Implementation Plan: Recalculate Amounts on an Existing List (Amended Model)

**Branch**: `018-recalculate-action` | **Date**: 2026-09-20 (re-planned after amendment) | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/018-recalculate-action/spec.md` (amended 2026-09-20)

> **Re-plan note**: The original plan targeted the pre-amendment model
> (a single `R` action on top of always-recalculate) and the pre-022
> monolith (`src/ui/app.rs`). The amendment reverses the model — manual
> quantities are authoritative, recalculation is explicit-only with
> three scopes — and spec 022 split the codebase into modules. This plan
> supersedes the old one entirely.

## Summary

Reverse the recalculation model per the amended spec: manual quantity
edits become authoritative (no automatic trigger overwrites them), the
twelve automatic `update_quantities()` call sites are removed, and the
recalculate action gains three scopes — **R** (current item),
**Ctrl+R** (current item's category), **Ctrl+Shift+R** (everything) —
with fixed items never changing in any scope. The existing
`AppDrawState::recalculate_quantities` (all items) is joined by
scope-filtered variants in `src/ui/state.rs`; the handler arms land in
`src/ui/handlers/welcome.rs` (main screen); the three bindings ship in
the hint lines, help screen, and extraction tests in the same change.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None — the actions write no file (FR-006). Quantities
persist per the existing exit-save flow; spec 012 (persist quantities in
the trip file) is planned separately and composes with this one.

**Testing**: `cargo test` — 42 existing tests (the behavioral
contract); new state-layer tests cover the three scopes (item,
category, everything; fixed preserved; idempotent; non-quantity state
untouched) and the manual-authority rule (a manual value survives
parameter changes and saves). The extraction tests pin the three
bindings against drift.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Same as today — the scoped actions reuse the
existing calculation path over a filtered item set.

**Constraints**: Warning-free gate (constitution Principle II); the
three bindings appear in hints, help, and extraction tests in the same
change (spec 016's drift-guard contract); no new calculation logic —
the existing `QuantityCalculation` is reused with scope filtering.

**Scale/Scope**: One state-layer method family (scope-filtered
recalculate), removal of ~12 automatic trigger call sites, three
handler arms, hint/help/extraction updates, and tests. All in the
post-022 module tree.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0:

- **Principle I (No Unused Code)**: satisfied — removing the automatic
  triggers deletes code; the scoped actions are reachable and tested.
- **Principle II (Zero Warnings and Errors)**: satisfied — every task
  ends with the gate clean.
- **Principle III (Documented and Tested)**: satisfied — the three
  bindings ship in hints, help, and TUI_USAGE.md in the same change;
  the scope contracts get state-layer tests; the extraction tests pin
  the bindings.
- No violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/018-recalculate-action/
├── plan.md              # This file (re-planned)
├── research.md          # Phase 0 (original; still accurate on bindings)
├── data-model.md        # Phase 1 (updated below for the amended model)
├── quickstart.md        # Phase 1 (updated below)
├── contracts/           # Phase 1 (updated below)
├── checklists/
└── tasks.md             # Phase 2 (/speckit-tasks — regenerate)
```

### Source Code (repository root, post-022 module tree)

```text
src/
├── ui/
│   ├── state.rs          # AppDrawState::recalculate_quantities gains scope
│   │                     # filtering: RecalculateScope::{Item,Category,All}
│   ├── input.rs          # App::update_quantities (kept for the initial
│   │                     # load calculation only); trigger removal
│   ├── handlers/
│   │   ├── welcome.rs    # main-screen handler: R / Ctrl+R / Ctrl+Shift+R
│   │   ├── dialogs.rs    # remove the add/edit recalculate triggers
│   │   ├── config.rs     # remove the modifier add/delete triggers
│   │   └── editor.rs     # remove the raw-editor-apply trigger
│   ├── hints.rs          # the three bindings in both list panels
│   ├── help.rs           # the three bindings with their scopes
│   └── tests/            # new scope + manual-authority tests
└── quantity/
    └── calculation.rs    # unchanged (reused as-is)
```

**Structure Decision**: The scope filtering lives in the state layer
(`src/ui/state.rs`) beside the existing `recalculate_quantities`, per
the spec 015 pattern. The handler arms live in the main-screen handler
(`src/ui/handlers/welcome.rs`). Trigger removal touches the four
handler modules that currently call `update_quantities()` as a side
effect. No new modules; no new dependencies.

## Phase 0 Research (delta over the original research.md)

The original research (bindings, extraction-test surfaces, state-layer
pattern) remains accurate. New decisions for the amended model:

- **D1 — Scope representation**: a `RecalculateScope` enum
  (`Item(usize) | Category(String) | All`) passed to a single
  state-layer method `recalculate_quantities_scoped(&mut self, calc,
  scope)`. The existing all-items method delegates to it with
  `Scope::All`. One code path, three entry points.
- **D2 — Manual authority needs no new state**: the trip file already
  restores quantities (spec 004's load path keeps saved values for
  fixed items; automatic items recalculate on load — see D3). "Manual"
  is simply "not recalculated": once the automatic triggers are gone,
  any value the user set stays until an explicit recalculate key
  touches its scope. No `manual: bool` flag is added — the spec's
  authority model is achieved by removing the overwrite paths.
- **D3 — Load behavior**: the initial load of a fresh list calculates
  once (existing `App::new` behavior, kept). A resumed trip restores
  the trip file's quantities; the current load path recalculates
  automatic items on resume — this must change to restore-only (FR-008:
  "a resumed trip MUST restore saved quantities without
  recalculating"). The load path in `src/ui/input.rs`
  (`apply_trip_file_path` and `App::new`) stops calling
  `update_quantities` after a resume.
- **D4 — Trigger inventory (verified in code)**: the automatic
  `update_quantities()` call sites to remove: welcome-screen parameter
  changes (4 sites in `handlers/welcome.rs`), resume-prompt decline
  (`handlers/welcome.rs`), modifier add/delete (`handlers/config.rs`,
  2 sites), raw-editor apply (`handlers/editor.rs`), add/edit item
  saves (`handlers/dialogs.rs`, 2 sites via
  `update_quantities_or_report`), trip-file apply (`input.rs`), and the
  `App::new` initial calculation (kept — it is the fresh-list load
  calculation, FR-008).
- **D5 — Ctrl+Shift+R detection**: crossterm reports
  `KeyModifiers::CONTROL | KeyModifiers::SHIFT` with `KeyCode::Char('R')`
  (shift produces uppercase). The handler matches modifiers before the
  plain `r`/`R` arm.

## Phase 1 Design

### Data model (delta)

```text
RecalculateScope { Item(usize), Category(String), All }
```

`AppDrawState::recalculate_quantities_scoped(calc, scope)`:
builds the `QuantityCalculationInput` with `ItemQuantitySpec::Fixed`
for every item **outside** the scope (so the calculation leaves them
exactly as they are) and `Auto` for items inside it. Fixed-mode items
inside the scope are also passed as `Fixed` (FR-004). This reuses the
existing calculation verbatim — the scope is expressed entirely in the
item specs.

### Contracts (delta)

- Hints: LeftPanel and RightPanel gain `("R", "Recalc item")`,
  `("Ctrl+R", "Recalc category")`, `("Ctrl+Shift+R", "Recalc all")`
  (exact wording per the hint-width tests; entries must fit the
  minimum panel width).
- Help: the three keys documented under the main screen's actions with
  their scopes.
- Extraction tests: `help_binding_tokens` gains `"Ctrl+R"` and
  `"Ctrl+Shift+R"`; the handler-binding list gains all three.

### Quickstart (delta)

- Manual authority: set a quantity, change every parameter, save,
  add/edit other items — the value must not change.
- Scopes: with three categories and a manual value in each, R changes
  only the cursor's item; Ctrl+R the cursor's category; Ctrl+Shift+R
  everything (fixed items unchanged in all cases).
- Resume: save, exit, resume — quantities are exactly the saved ones.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. The amendment removes code (automatic triggers) and adds three reachable, tested actions. | | |
