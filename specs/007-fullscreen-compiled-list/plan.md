# Implementation Plan: Fullscreen Compiled List View

**Branch**: `007-fullscreen-compiled-list` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Add a fullscreen toggle on the main screen: a dedicated key switches
to a view showing only the compiled packing list (the right panel's
content at full width); the same key switches back. The trip's data
is unaffected — purely presentation. All compiled-list actions
(navigate, toggle packed, recalculate) work identically in
fullscreen; the view scrolls and follows the cursor.

**Sequencing note**: Independent of the data-model features; touches
the main screen's layout and one handler arm. The binding joins the
hint/help/extraction surfaces in the same change. If spec 006
(stacked small-terminal layout) lands first, the fullscreen toggle
applies in both layouts (the compiled list is the same panel either
way).

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None — a view flag on the session state; not persisted
(a restarted app starts in the two-panel view).

**Testing**: `cargo test` — 42 existing tests; new state tests:
the toggle flips the flag and back; the compiled-list actions are
unaffected by the flag. The extraction tests gain the binding.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged — the fullscreen view reuses the
compiled panel's renderer at a wider area.

**Constraints**: Warning-free gate; the toggle is presentation-only
(no data change, US1/AC3); all compiled-list bindings work in
fullscreen (US2); the binding is documented and extraction-tested.

**Scale/Scope**: A `fullscreen: bool` on `AppDrawState`, one handler
arm (the toggle key), the main screen's layout branch (fullscreen →
render only the compiled panel at full size), hints/help/tests.

## Constitution Check

- **Principle I**: satisfied — the flag is read by the renderer and
  written by the handler.
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — the binding ships in hints/help and
  the extraction tests in the same change.
- No violations.

## Project Structure

```text
src/
└── ui/
    ├── state.rs          # AppDrawState gains fullscreen: bool (default
    │                     #   false); cursor/selection state unchanged
    ├── handlers/
    │   └── welcome.rs    # the main-screen handler gains the toggle key
    ├── screens/
    │   └── main.rs       # draw_main_screen branches: fullscreen renders
    │                     #   only draw_selected_items_panel at full size
    ├── hints.rs / help.rs    # the binding in both panels' hints + help
    └── tests/
        └── input.rs / state.rs   # toggle tests
```

**Structure Decision**: The fullscreen view reuses
`draw_selected_items_panel` — the same renderer, a different area.
No separate fullscreen module; the flag is a layout branch in
`draw_main_screen`.

## Phase 0 Research

- **D1 — The renderer is reusable (verified)**:
  `draw_selected_items_panel(f, area, state)` takes its area as a
  parameter and renders the compiled list with its title, hints,
  stats, and scrolling — passing the full main-screen area renders
  it fullscreen with zero renderer changes.
- **D2 — The binding**: a dedicated key; `F` (fullscreen) is free on
  the main screen (verified against the handler's arms and the help
  tokens). The exact key is a Phase 1 contract decision; `F` is the
  default candidate.
- **D3 — Actions in fullscreen (US2)**: the main-screen handler's
  arms (Up/Down, Space/packed, R/recalculate, Tab) operate on the
  same state the compiled panel renders — they work unchanged. Tab
  in fullscreen is a no-op (there is no second panel) or toggles
  back; the contract decision: Tab does nothing in fullscreen (the
  fullscreen key is the way back).
- **D4 — Cursor and scroll**: the compiled panel's existing
  cursor-following scroll logic is area-relative — it works at any
  width/height unchanged.

## Phase 1 Design

### Data model

```text
AppDrawState { ..., fullscreen: bool }   // default false; not persisted
```

### Contracts

- Binding: `F` on the main screen toggles fullscreen (both
  directions); documented in both panels' hints and the help
  screen; extraction tests gain the token.
- Fullscreen shows only the compiled list; all its actions work;
  the trip saves identically from either view (US1/AC3).

### Quickstart

- With items selected, press F → only the compiled list, full
  width; navigate, toggle packed, recalculate — all work.
- Press F again → the two-panel view with the same selections,
  packed states, and cursor.
- Exit from fullscreen → the trip saves exactly as from the main
  view.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
