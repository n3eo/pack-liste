# Data Model: Label-Based Item Storage (revision 3)

## Entities

### Item (the in-memory model — changed)

| Field | Type | Rules |
|-------|------|-------|
| name | String | Unchanged. |
| labels | Vec\<String\> | One or more plain strings; the item's only grouping data. Order preserved as written; no hierarchy, no separator. |
| singleton | bool | When true, the item's selection and packed toggles are synchronized across all its labels (one state each). Default false; meaningful only for multi-label items. |
| quantity, packed, selected, auto_quantity, rate, buffer | unchanged | — |

**Validation rules** (FR-001/FR-002): `labels` is non-empty; the strings are stored and compared verbatim.

### SavedItem (the configuration-file schema — changed)

| Field | Type | Serialization |
|-------|------|---------------|
| name | String | required |
| labels | Vec\<String\> | required; an empty list rejects |
| singleton | bool | `#[serde(default)]` + `skip_serializing_if` when false |
| quantity, auto_quantity, rate, reserve, buffer_multiplier | unchanged | — |

`deny_unknown_fields` stays: an old-format item (with `categories:`) rejects with the unknown-field error (FR-004).

### AppConfig (the configuration-file root — changed)

The top-level `categories` list is REMOVED (FR-002); `items` carries the labeled items; the other fields unchanged.

### TripItem (the trip-file schema — changed)

| Field | Type | Change |
|-------|------|--------|
| name | String | unchanged |
| labels | Vec\<String\> | replaces `category` |
| singleton | bool | new; same optionality as SavedItem |
| packed, quantity | unchanged | the item's own state (the snapshot) |

### PanelRow (the row model — flattened)

| Variant | Payload | Meaning |
|---------|---------|---------|
| `Label(String)` | the label name | a flat group header |
| `Item(usize)` | index into packing_list.items | an item row (appears under every label it carries) |

The Group/Category variants die with the two-layer tree.

### Occurrence state (session — new)

| State | Type | Rules |
|-------|------|-------|
| `occurrence_overrides` | BTreeMap\<(usize, String), bool\> | A per-session override of an item's toggle for ONE (item, label) occurrence. Effective state = the override if present, else the item's own. |

- Toggling a SINGLETON occurrence flips the item's own state and clears its overrides (synchronized — FR-011).
- Toggling a NON-SINGLETON occurrence writes the override (independent — FR-012).
- The persisted files store the item's own state only; overrides start empty on load (a disagreeing singleton normalizes to the item's state — FR-014).

### Collapse state (session — renamed)

`collapsed_labels: Vec<String>` replaces `collapsed_categories` + `collapsed_groups`; a collapsed label hides its items (the cursor clamps to the header, as today).

## State Transitions

```text
Load:     yaml → SavedItem{labels, singleton?} → dedup by (name, labels) → Item{labels, singleton}
Toggle:   singleton occurrence → flip item.selected/packed, clear overrides (all labels follow)
          non-singleton occurrence → write occurrence_overrides[(item, label)]
Rows:     items → one Label header per distinct label → the items carrying it
Chips:    per label: DarkGray when effective == item's own state, Gray when it differs
Save:     Item{labels, singleton} → SavedItem → yaml (the item's own state only)
Reconcile: trip item ← config item by name; labels + singleton updated, packed/quantity kept
```

## Relationships

```text
Item (1) --mirrors--> SavedItem (1) --yaml--> config file
Item (1) --mirrors--> TripItem (1) --yaml--> trip file
Item (N) --appears under--> Label (1..*)     (one row per label it carries)
occurrence_overrides --per (item, label)--> the effective toggle state
```

## Persistence

Both files are strict single-format: `labels` (+ optional `singleton`) is the only grouping data; the old `categories:` form rejects. The migration is external and one-time: each "X: Y" → labels [X, Y] with `singleton: true`; "X" → labels [X]; exact duplicates merge (first wins, reported).
