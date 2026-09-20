# Implementation Plan: Add Dialog Quantity and Mode

**Branch**: `021-add-dialog-quantity-mode` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Give the add-item dialog the edit dialog's Quantity and Mode fields:
Quantity (typed value becomes the new item's quantity; empty → 1) and
Mode (automatic default / fixed via Space toggle — fixed keeps the
typed quantity forever, automatic uses it as the starting value). The
two dialogs end with identical fields, order, bindings, and hint line
(US3) — the unified `draw_item_dialog` (spec 022) already renders
both; the add path gains the two fields' state and save semantics.

**Sequencing note**: Composes with spec 020 (dialog Up/Down field
cycling) — 020 changes how fields are cycled, this spec changes which
fields exist. Either order works; if 020 lands first, the new fields
join its cycle. Composes with the amended spec 018: adding an item
triggers no recalculation (018 EC-004) — the typed quantity IS the
item's quantity until the user recalculates.

## Technical Context

**Language/Version**: Rust 1.98 (2024 edition)

**Primary Dependencies**: ratatui 0.30, crossterm 0.29 (unchanged)

**Storage**: None new — the new item's quantity and auto_quantity flag
persist through the existing `SavedItem` schema (`quantity` written
for fixed items, `auto_quantity` always).

**Testing**: `cargo test` — 42 existing tests; new state-layer tests:
add-with-quantity-5 → item quantity 5; add-empty → 1; add-fixed →
`auto_quantity: false` and the quantity survives a recalculate;
add-automatic → recalculates on the next explicit recalculate (018
model). The dialog-parity invariant (US3) is asserted: both
`InputMode` variants carry the same field set.

**Target Platform**: Terminal TUI (Linux)

**Project Type**: CLI/TUI application (single binary)

**Performance Goals**: Unchanged.

**Constraints**: Warning-free gate; the add dialog's default mode is
automatic (US2/AC1); the shared renderer must not fork (US3 — one
`draw_item_dialog`); hints/help updated in the same change
(extraction tests).

**Scale/Scope**: `InputMode::AddingItem` gains `quantity: String` and
`auto_quantity: bool`; the add handler's save path carries them; the
renderer's Mode field already renders for any `auto_quantity:
Option<bool>` — the add path passes `Some(...)`; hints/help/extraction
updates; ~4 tests.

## Constitution Check

- **Principle I**: satisfied — the new fields are read and written.
- **Principle II**: satisfied — gate clean per task.
- **Principle III**: satisfied — help/hints document the shared field
  set; tests ship with the feature.
- No violations.

## Project Structure

```text
src/
└── ui/
    ├── state.rs          # InputMode::AddingItem gains quantity +
    │                     #   auto_quantity; toggle_edit_mode covers both
    │                     #   dialog modes; apply/save paths carry them
    ├── handlers/
    │   ├── welcome.rs    # the dialog handler's AddingItem arms gain
    │   │                 #   quantity typing + Space mode toggle
    │   └── dialogs.rs    # save_new_item gains (quantity, auto_quantity);
    │                     #   start_adding_item initializes them
    ├── draw.rs           # the add path passes auto_quantity: Some(..)
    │                     #   to the shared draw_item_dialog
    ├── hints.rs / help.rs    # the shared dialog bindings (already shared)
    └── tests/
        └── input.rs / state.rs   # the new add-semantics tests
```

**Structure Decision**: The fields live on `InputMode::AddingItem`
(mirroring `EditingItem`); the shared renderer needs no change beyond
passing the mode through. One dialog implementation, two entry points.

## Phase 0 Research

- **D1 — Current add path (verified)**: `InputMode::AddingItem` has
  `field, name, category, quantity` — wait, it already has `quantity:
  String` (the dialog state) but `save_new_item` parses it and the
  item is always created automatic; there is no Mode field in the add
  dialog's state or rendering (the renderer passes
  `auto_quantity: None` for add). The gap: the Mode field's state,
  toggle, rendering, and save semantics.
- **D2 — Renderer parity is nearly free**: `draw_item_dialog` renders
  the Mode field when `auto_quantity: Option<bool>` is `Some`. The add
  path currently passes `None`; passing `Some(mode)` completes US3's
  identical-fields requirement with a one-line change.
- **D3 — Save semantics**: `save_new_item(name, category, quantity)`
  gains `auto_quantity: bool`; the created `Item` gets
  `auto_quantity` set accordingly (fixed items keep the typed
  quantity; automatic items start there). Under 018's model no
  recalculation follows the add (EC-001 here, EC-004 in 018).
- **D4 — Mode toggle**: `toggle_edit_mode` currently matches only
  `EditingItem`; it generalizes to both dialog modes (the state
  method takes the mode by mutable reference or the handler matches
  both arms).

## Phase 1 Design

### Data model

```text
InputMode::AddingItem { field, name, category, quantity, category_list, category_index, auto_quantity: bool }
```

Default on open: `auto_quantity: true` (US2/AC1). Space on the Mode
field toggles it (both dialog modes). Save: the new item's
`auto_quantity` = the dialog's value; quantity = parsed typed value
(default 1).

### Contracts

- Both dialogs: fields Name → Category → Quantity → Mode; identical
  bindings; one hint line (AddEditPopup) and one help section for both.
- Extraction tests: no new tokens needed (the bindings already exist
  for the edit dialog); the parity is asserted by a state test.

### Quickstart

- Add with quantity 5 → the item shows 5.
- Add with empty quantity → 1.
- Add with Mode fixed, quantity 3 → survives any recalculate.
- Add with Mode automatic, quantity 3 → recalculates on the next
  explicit recalculate (018) or parameter change (pre-018).
- Both dialogs: same fields, same cycle, same hints.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None. | | |
