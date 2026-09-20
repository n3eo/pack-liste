# Implementation Plan: No Packed State on the Left Panel

**Branch**: `008-hide-packed-left` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Remove the left panel's per-item packed indicator entirely (the
second `✓`/`○` symbol and its green/red coloring): the left panel
shows each item's selection symbol, name, and quantity — nothing
about packed state, packed or unpacked alike. The right panel (the
compiled list) keeps its full packed-state display: per-item
indicators, category packing markers, and the progress summary,
exactly as today. Items stay visible on the left regardless of
packed state; only the indicator goes.

**Sequencing note**: Independent of the other planned features; can
land in any order. Touches only the left panel's item-row rendering.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30 (unchanged)

**Storage**: None — display only; `Item.packed` and its persistence
are untouched.

**Testing**: `cargo test` — 42 existing tests; a new state/render
test asserts the left panel's row content is identical for packed
and unpacked items (the spec's independent test, verbatim: pack and
unpack on the right; the left renders identically).

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; the right panel's packed display
is untouched (US2); no data-model change — `Item.packed` stays.

**Scale/Scope**: One renderer function (`draw_all_items_panel`'s item
row): delete the `item_symbol` span and its color span; the row
becomes selection symbol + name + quantity.

## Constitution Check

- **Principle I**: satisfied — the removal deletes dead rendering
  code; nothing is added unused.
- **Principle II**: satisfied — gate clean.
- **Principle III**: satisfied — the left panel's hint line already
  says "Space Select" (no packed mention); the change is covered by
  a test.
- No violations.

## Project Structure

```text
src/
└── ui/
    └── screens/
        └── main.rs      # draw_all_items_panel's item row: remove the
                        #   packed symbol + color spans (lines ~146-160
                        #   in the current file); the right panel's
                        #   draw_selected_items_panel is untouched
```

**Structure Decision**: A surgical renderer edit; no state, handler,
or model changes.

## Phase 0 Research

- **D1 — The exact code (verified)**: `draw_all_items_panel` renders
  per item: `"  "` + `selection_symbol` (green/red by selected) +
  `" "` + `item_symbol` (`"✓ "`/`"○ "` by packed, green/red) + `" "` +
  name + `" x"` + quantity (`src/ui/screens/main.rs:146-165`). The
  removal deletes the `item_symbol` computation and its two spans;
  the row keeps selection, name, quantity.
- **D2 — The right panel is separate (verified)**:
  `draw_selected_items_panel` has its own `item_symbol` rendering
  (`main.rs:362-366`) and the category all-packed markers — all
  untouched (US2).
- **D3 — No hidden packed coupling**: the left panel's cursor
  highlighting, collapse behavior, and selection toggling do not
  read `packed`; only the removed spans did.

## Phase 1 Design

### Data model

None — display only.

### Contracts

- Left panel row: `[selection symbol] [name] x[quantity]` — no
  packed indicator, packed or unpacked.
- Right panel: unchanged (per-item indicators, category markers,
  progress summary).

### Quickstart

- Pack an item on the right; the left panel's row for it is
  byte-identical to before packing (screenshot-diff or the test).
- Unpack it; still identical.
- The right panel's indicators and progress still update normally.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
