# Data Model: Two-Layer Category Nesting

## Entities

### Category String (existing, unchanged)

The persisted identity of an item's grouping: the full "X: Y" string. Frozen contract — stored and reloaded verbatim (FR-007). No new fields; no migration.

### Layer Split (derived, per frame)

The pure derivation from a category string to its two display layers.

| Field | Type | Rules |
|-------|------|-------|
| top layer | substring | The part before the FIRST ": ". For separator-free categories, the whole string. |
| sub-category | substring or none | The part after the FIRST ": ", verbatim (may itself contain ": "). `None` when the category has no separator. |

**Validation rules** (from spec FR-001):
- The split uses the first occurrence of ": " only.
- The derivation never mutates or rewrites the stored string.
- Separator-free category: top layer = the whole string, no sub-category (FR-003).

### Panel Row (extended)

The flat, navigable row list both panels render. New variant:

| Variant | Payload | Meaning |
|---------|---------|---------|
| `Group(String)` | the top-layer name | A top-layer header row. |
| `Category(String)` | the FULL category string | A sub-category header row (renders the part after ": ", indented; identity is the full string). |
| `Item(usize)` | index into packing_list.items | An item row (unchanged). |

**Display order**: `Group(top)`, then for each of its sub-categories `Category(full)` followed by its items; separator-free categories render `Group(name)` with items directly beneath (no `Category` row).

### Collapse State (extended)

| Field | Type | Rules |
|-------|------|-------|
| `collapsed_categories` | Vec of full category strings | Existing; sub-category level. A collapsed sub-category hides its items only. |
| `collapsed_groups` | Vec of top-layer names | New; group level. A collapsed group hides its sub-category headers and items. |

Both are session state, not persisted. A collapsed group's sub-categories may independently be collapsed; expanding the group restores each sub-category's own state.

### Selection Scope (extended)

`Ctrl+Space` resolves by cursor row:
- On `Group(top)`: every item whose ANY category's top layer matches (FR-006).
- On `Category(full)`: every item in that full-string category (existing behavior).

### Recalculate Scope (extended)

| Variant | Meaning |
|---------|---------|
| `Item(usize)` | The one item (unchanged). |
| `Category(String)` | The full-string sub-category (unchanged meaning). |
| `Group(String)` | NEW: every item whose category's top layer matches. |
| `All` | Everything (unchanged). |

## State Transitions

```text
Category string --split(first ": ")--> {top layer, sub-category?}   (pure, per frame)
Cursor on Group header --C--> collapsed_groups toggles; sub-tree hidden; cursor stays
Cursor on Category header --C--> collapsed_categories toggles; items hidden; cursor stays
Cursor on Group header --Ctrl+Space--> all items under the group flip selection
Cursor on Category header --Ctrl+Space--> all items in the category flip selection (existing)
Right-panel navigation --> left panel expands the item's sub-category AND group (sync)
```

## Relationships

```text
Category String (1) --derives--> Layer Split (1)
Top layer (1) --groups--> Category Strings (N)
Panel Row list (1) --renders--> Group / Category / Item rows
Collapse State (2 vectors) --filters--> Panel Row list (visible rows)
```

## Persistence

Unchanged. The configuration and trip files store category strings verbatim; the layers, collapse state, and row model are display-time only and never serialized.
