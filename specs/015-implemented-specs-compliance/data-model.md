# Data Model: Implemented-Specs Compliance Review

**Feature**: Implemented-Specs Compliance Review
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This feature changes one persisted entity (Item's category membership),
one dialog state (the edit dialog's mode field), and adds two
presentation-only derivations. No storage format is removed; the config
format gains an additive per-item field.

## Entity changes

### Item (src/models/item.rs) — US6

**Before**: `category: String` (exactly one category per item).

**After**:

```text
Item {
    id: String,
    name: String,
    categories: Vec<String>,   // one or more; was `category: String`
    base_quantity: u32,
    quantity: u32,
    packed: bool,
    editable: bool,
    auto_quantity: bool,
}
```

**Validation rules**:
- `categories` is non-empty (an item always belongs to at least one
  category; the edit dialog enforces this — saving with zero
  memberships is rejected).
- Duplicate entries within one item's `categories` are deduplicated on
  save.
- All other fields and their rules are unchanged (quantity ≥ 1,
  `auto_quantity` semantics per spec 003).

**State transitions**: none new — selection (`editable`) and packed
state remain per-item singletons shared across all category
occurrences. Toggling selection or packed state through any occurrence
changes the item once; every occurrence reflects it on the next render.

**Persistence (config.yaml `SavedItem`)**:
- Written: `categories: [..]` per item.
- Read: `categories` if present; otherwise the legacy single
  `category: <string>` value is lifted into a one-element list (serde
  default at the field level). Existing files load unchanged.

### TripItem (src/config/trip_file.rs) — US6 ripple

Unchanged shape (`name`, `category`, `packed`): the trip file stores the
item's *primary* category (the first membership) for display grouping.
`TripFile::reconcile` matches a trip item to a catalog item by
case-insensitive name against **any** of the catalog item's categories,
preferring an exact name+category match as today.

### InputMode::EditingItem (src/ui/app.rs) — US1

Gains one field:

```text
EditingItem {
    index: usize,
    field: InputField,          // gains variant: Mode
    name: String,
    category: String,
    quantity: String,
    auto_quantity: bool,        // NEW: carried from the item, written back on save
    category_list: Vec<String>,
    category_index: usize,
}
```

`InputField` gains `Mode`, cycled by Tab in the existing rotation
(Name → Category → Quantity → Mode → Name). While `Mode` is focused:
Space toggles `auto_quantity`; the quantity field's editability follows
the mode (fixed: editable; automatic: shown as calculated, editing it is
a no-op that the mode field makes visible rather than silent).

**Validation rules**: toggling to fixed keeps the current quantity
(EC-003); no fixed item can hold a quantity below 1 (EC-004 — existing
minimum-1 rule, unchanged).

**State transitions**: `auto → fixed` freezes the current quantity at the
moment of save; `fixed → auto` recalculates on save via the existing
`update_quantities`.

## New presentation-only derivations (no stored state)

### Category packing state — US2

Derived per category header on the compiled list at draw time:

```text
all_packed(category) = every selected item of the category has packed == true
```

Marker: `✓` (green) when all packed, `○` (red) otherwise — the existing
per-item symbol vocabulary. Left panel headers are unchanged (EC-002).

### Modifier fallback — US3

Not an entity: the multiplier resolution for selected activities changes
from "drop unknown names" to "unknown names resolve to 1.0", making all
three modifier kinds uniform. The calculation input/output structures
are unchanged.

## Test-layer addition — US5

The pure state transitions currently implemented as `App` methods move
to `AppDrawState` (or free functions over `&mut AppDrawState`):

```text
AppDrawState {
    ...existing fields...
}

// extracted, terminal-free, unit-testable:
- navigation: move_cursor_up/down (per screen and input mode)
- toggles: toggle_item_selection, toggle_category_selection,
  toggle_item_packed, toggle_category_collapse
- edit dialog: apply_edit_save (mode + fields → item)
- collapse: set_category_collapsed
```

`App`'s handlers become one-line delegates. No behavior change; the
extraction is what makes the input tests possible (EC-007).

## State transitions summary

| Transition | Trigger | Effect |
|------------|---------|--------|
| auto → fixed | Mode field toggle + save | quantity frozen at current value |
| fixed → auto | Mode field toggle + save | quantity recalculated immediately |
| selection toggle (any occurrence) | Space / Ctrl+Space / category toggle | item `editable` flips once; all occurrences consistent |
| packed toggle | Space on compiled list | item `packed` flips; category indicator re-derives next frame |
| unknown modifier selected | parameter change | contributes 1.0; activity path stays active |
