# Implementation Plan: Harmonized Navigation, UI, and UX Across Views

**Branch**: `016-harmonized-ui-navigation` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/016-harmonized-ui-navigation/spec.md`

## Summary

Harmonize navigation and hints across all views: implement the uniform arrow model on the main screen (remove Left/Right collapse/expand; `C` stays the toggle; add Left/Right category cycling in the add/edit dialog), give both list views an always-visible hint footer (the right panel has none today; the left panel's vanishes below 5 spare rows), unify all hint styling (" | " separators, ↑/↓/←/→ glyphs, fix the corrupted "Select fields.category" popup hint), unify focus/cursor indication across panels, and extend the help screen and its extraction test to cover the final binding set and the hint lines. All work lands in `src/ui/app.rs` on the state layer extracted in spec 015; no new modules, no new dependencies.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged — no new dependencies)

**Storage**: YAML config and trip files (unchanged)

**Testing**: `cargo test` — the spec 015 state-layer tests and the help extraction test are the net; new tests extend both. The quality gate (`cargo check`, `cargo clippy`, `cargo clippy --tests`, `cargo fmt --check`) must stay warning-free per the `[lints]` table.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Hint footers are static per-panel lines; no measurable render cost. No performance constraints beyond the existing loop.

**Constraints**: Warning-free gate (constitution Principle II); behavior-preserving except the specified binding changes; the extraction test must keep passing after every binding change; spec 006's minimum-size rule (80x24) applies to the new footers.

**Scale/Scope**: ~4,700 lines in `src/ui/app.rs`; 5 user stories; all changes in the draw functions, the key handler, the help content, and the test module.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0 (`.specify/memory/constitution.md`):

- **Principle I (No Unused Code)**: satisfied — removing the Left/Right collapse binding removes code; no scaffolding is added. The removed binding's help entries go with it.
- **Principle II (Zero Warnings and Errors)**: satisfied — the `[lints]` table denies warnings; every task ends with the gate clean.
- **Principle III (Documented and Tested)**: satisfied and strengthened — the hint system *is* documentation; the extraction test extended to hints makes drift a test failure (FR-006). Every binding change ships with its help/hint update in the same change.
- No violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/016-harmonized-ui-navigation/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   └── hints-and-bindings.md  # The harmonized hint + binding contract
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
└── ui/
    └── app.rs           # All changes:
                         #   US1: remove Left/Right collapse handler (~1811-1821);
                         #        add Left/Right category cycling in handle_add_edit_input (~2622)
                         #   US2: hint footers in draw_all_items_panel (~3799) and
                         #        draw_selected_items_panel (~4004) — fixed footer areas
                         #   US3: hint style unification across draw_welcome_screen (~3035),
                         #        draw_config_screen (~3493), draw_config_editor_screen (~3688),
                         #        draw_add_item_popup (~4228), draw_edit_item_popup (~4394),
                         #        draw_confirm_delete_popup (~4587); fix "Select fields.category"
                         #   US4: focus/cursor indication in both panels' titles and rows
                         #   US5: help_content (~3365) + help_binding_tokens (~3309) +
                         #        extraction tests (~558-630)
```

**Structure Decision**: Single-project structure (unchanged). All five stories land in `src/ui/app.rs` — the draw functions, the key handler, the help content, and the test module. No new modules; the hint-line construction is extracted into small helper functions next to `help_content` so the extraction test can read them.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations. Not applicable.
