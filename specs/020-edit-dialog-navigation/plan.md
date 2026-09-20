# Implementation Plan: Edit Dialog Navigation

**Branch**: `020-edit-dialog-navigation` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Bring the add/edit dialog into the uniform arrow model: **Up/Down
cycle the dialog's fields** (Name → Category → Quantity → Mode,
wrapping; in the Category field they keep the dropdown-highlight
behavior, EC-001), **Left/Right adjust the focused value** (quantity
±1 clamped at 1 in the Quantity field; category cycling unchanged),
and the **free-text fields show a visible text cursor** at the
insertion position. Tab stays as the field-cycling alias. The hint
line, help screen, and extraction tests state the final binding set.

**Sequencing note**: Composes with 021 (add dialog gains the same
fields — the navigation applies to both dialogs' shared field set)
and 010 (Rate/Buffer fields join the cycle when 010 lands). The
field-cycling helper is written generically over `InputField` so
later fields extend the cycle without new navigation code.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None — dialog state is session-only.

**Testing**: `cargo test` — 42 existing tests; new state-layer tests:
field cycling (order, wrapping, both directions), quantity stepping
(clamp at 1, typing still appends), the cursor position invariant
(after the last character; retreats on Backspace). The extraction
tests gain the dialog's new binding tokens.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; the Category field's Up/Down
dropdown behavior is preserved (EC-001); the cursor is visible only
in focused free-text fields (US2/AC4); hints/help/extraction updated
in the same change.

**Scale/Scope**: The dialog handler's Up/Down/Left/Right arms
(`src/ui/handlers/welcome.rs`), a cursor position on the dialog state,
the renderer's cursor cell (the raw editor's cursor pattern), and the
hint/help/test updates.

## Constitution Check

- **Principle I**: satisfied — all new code is reachable and tested.
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — hints/help document the model;
  extraction tests pin it; state tests cover the contracts.
- No violations.

## Project Structure

```text
src/
└── ui/
    ├── state.rs           # InputField::next/prev (the cycle);
    │                       #   dialog state gains cursor positions for
    │                       #   the free-text fields; quantity stepping
    ├── handlers/
    │   └── welcome.rs      # Up/Down: field cycle (or dropdown in
    │                       #   Category); Left/Right: quantity step (or
    │                       #   category cycle); typing/Backspace move
    │                       #   the cursor
    ├── draw.rs             # draw_item_dialog renders the text cursor
    │                       #   (a highlighted cell after the focused
    │                       #   field's text; the raw editor's pattern)
    ├── hints.rs / help.rs  # the dialog's final binding set
    └── tests/
        └── input.rs / state.rs   # cycling, stepping, cursor tests
```

**Structure Decision**: The field cycle is a method on `InputField`
(`next()`/`prev()`), so 010's Rate/Buffer fields extend it in one
place. The cursor is per free-text field (Name, Quantity), stored on
the dialog state as a byte position; the renderer draws a block
cursor at the position (the raw editor's cursor-cell approach).

## Phase 0 Research

- **D1 — Current dialog input (verified)**: the dialog handler
  (`handlers/welcome.rs`) routes Up/Down to
  `move_cursor_up/down`, which only move the category dropdown's
  highlight (`state.rs`); Left/Right cycle the category
  (`cycle_category_selection`); Tab cycles fields via
  `cycle_dialog_field`. The gaps: field cycling on Up/Down (outside
  the Category field), quantity stepping on Left/Right, and the text
  cursor.
- **D2 — The cursor model**: the dialog's free-text fields append at
  the end (push/pop) — the insertion position is always the end of
  the text. A visible cursor is therefore a *rendering* concern: a
  highlighted cell immediately after the field's text when the field
  is focused (US2/AC1-3 are satisfied by end-insertion semantics; no
  mid-string editing is requested). The cursor position is derived
  from the text length — no new state is needed.
- **D3 — Quantity stepping**: the dialog's quantity is a `String`;
  stepping parses it (empty → 1), adds ±1, clamps at 1 (US3/AC3), and
  writes back. Typing still appends (US3/AC4).
- **D4 — EC-001 resolution**: in the Category field, Up/Down keep the
  dropdown-highlight behavior; field cycling is reachable from the
  other three fields and via Tab everywhere. The handler matches the
  focused field first.
- **D5 — Renderer cursor**: the raw editor already renders a cursor
  cell (`screens/editor.rs` — a `Rect` with a background style at
  the cursor position). The dialog reuses the pattern: compute the
  cell after the focused field's label + text, render a block there.

## Phase 1 Design

### Data model

```text
InputField::next() / prev()   // Name → Category → Quantity → Mode → Name (wrapping)
```

Quantity stepping: parse (default 1) → clamp(1, +∞) → ±1 →
to_string. Cursor: derived from the focused field's text length (no
new state).

### Contracts

- Dialog bindings: Up/Down = cycle fields (dropdown highlight in
  Category); Left/Right = adjust the focused value (quantity ±1,
  category cycling); Tab = cycle alias; Space = Mode toggle /
  Category membership (existing).
- Hints (AddEditPopup) and help updated; extraction tests gain the
  tokens.

### Quickstart

- Down ×4 from Name wraps back to Name; Up reverses.
- In Category, Up/Down move the dropdown; Tab still cycles.
- In Quantity: Right 3→4, Left 4→3, Left at 1 stays 1; typing appends.
- The cursor is visible in Name/Quantity when focused, absent in
  Category/Mode.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
