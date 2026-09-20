# Implementation Plan: Recalculate Amounts on an Existing List

**Branch**: `018-recalculate-action` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/018-recalculate-action/spec.md`

## Summary

Add an explicit, idempotent recalculate action to the main screen's list views: the `R` key (currently unbound) reruns the existing quantity calculation over the current list — automatic items update from the current trip parameters and per-item configuration, fixed items never change, and no packed/selection/collapse state or file is touched. The calculation body of `App::update_quantities` moves into a state-layer method `AppDrawState::recalculate_quantities` (the spec 015 pattern), the handler becomes a one-line delegate, and the binding lands in both list panels' hint lines, the help screen, and all four extraction-test surfaces in the same change so it cannot drift.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged — no new dependencies)

**Storage**: None — the action writes no file; quantities remain session state (spec 012 not implemented; US4 rides the existing exit-save flow)

**Testing**: `cargo test` — one new state-layer test (`recalculate_updates_auto_and_preserves_fixed`) covers the action's contract (auto update, fixed preserved, non-quantity state untouched, idempotent); the extraction tests are extended with the `R` binding. The quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo fmt --check`) must stay warning-free per the `[lints]` table.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Same responsiveness as a parameter-change recalculation (FR-006) — the action reuses the existing calculation path, no new work.

**Constraints**: Warning-free gate (constitution Principle II); the binding must appear in hints, help, and all extraction-test surfaces in the same change (Principle III, spec 016's drift-guard contract); no new calculation logic (the spec's own assumption — reuse `update_quantities`).

**Scale/Scope**: One new handler arm, one extracted state-layer method (~40 lines moved, not written), one test, and data updates across the hint/help/extraction surfaces — all in `src/ui/app.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0 (`.specify/memory/constitution.md`):

- **Principle I (No Unused Code)**: satisfied — the feature adds one reachable action and one method extracted from existing code; nothing is scaffolded for later features (spec 012's persistence, when implemented, will reuse the same path).
- **Principle II (Zero Warnings and Errors)**: satisfied — every task ends with the gate clean.
- **Principle III (Documented and Tested)**: satisfied — the binding ships in the hint lines, the help screen, and TUI_USAGE.md in the same change; the action's contract gets a state-layer test; the extraction tests pin the binding against drift.
- No violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/018-recalculate-action/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output: verified facts F1-F7, decisions D1-D5
├── data-model.md                  # Phase 1 output: the state-layer method, action contract, binding
├── quickstart.md                  # Phase 1 output: validation scenarios
├── contracts/
│   └── recalculate-binding.md     # Phase 1 output: the R binding amendment to spec 016's contract
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
└── ui/
    └── app.rs           # All changes:
                         #   US1/US3: extract AppDrawState::recalculate_quantities from
                         #            App::update_quantities (~1907); add the R handler arm
                         #            (~2443 area, the letter-handler block); state-layer test
                         #   US2: hint_entries LeftPanel/RightPanel gain ("R", "Recalculate")
                         #        (~3598); help_content MAIN SCREEN gains the R entry (~3823);
                         #        help_binding_tokens, help_screen_text_covers_all_handler_bindings,
                         #        hint_entries_match_the_contract_binding_sets gain "R"
```

**Structure Decision**: Single-project structure (unchanged). All changes land in `src/ui/app.rs`; no new modules, no new dependencies. The extraction is a move of existing code into the established state-layer pattern (spec 015).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations. Not applicable.
