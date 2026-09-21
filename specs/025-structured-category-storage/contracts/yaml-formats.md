# Contract: The Label YAML Formats and the Singleton Semantics

The persisted data contract after this feature: two strict yaml schemas built on labels, plus the observable singleton semantics.

## The configuration file

```yaml
weekdays: 7
travel_style: backpacker
weather: mixed
modifiers:
  travel_styles:
  - name: standard
    multiplier: 1.0
  weather:
  - name: mixed
    multiplier: 1.0
items:
- name: Führerschein international
  labels:
  - Ausland
  - Karten
  singleton: true
  quantity: 1
  auto_quantity: false
- name: Zahnbürste
  labels:
  - Kulturbeutel
  quantity: 1
  auto_quantity: true
last_trip_file: /path/to/trip.yaml
```

Rules:
- Every item has `labels` — one or more plain strings; an empty list rejects.
- `singleton` is optional: present (true) only when the item is a synchronized multi-label item; absent otherwise.
- NO top-level `categories` list; no category/subcategory fields; no ": " convention anywhere.
- Unknown fields reject (strict schema): an old-format item with `categories:` fails with an error naming the unknown field.
- The load DEDUPLICATES: entries with the same name and the same labels merge (first occurrence wins); same name with different labels are distinct items.
- The other fields are unchanged.

## The trip file

```yaml
weekdays: 7
travel_style: backpacker
weather: mixed
items:
- name: Führerschein international
  labels:
  - Ausland
  - Karten
  singleton: true
  packed: false
  quantity: 1
- name: Zahnbürste
  labels:
  - Kulturbeutel
  packed: true
  quantity: 1
```

Rules: `labels` (+ optional `singleton`) per trip item; `packed`/`quantity` unchanged (the item's own state — the snapshot). Unknown fields reject.

## The singleton semantics (the observable contract)

| | Singleton | Non-singleton |
|---|-----------|---------------|
| Toggling selection under one label | Flips the item everywhere (one selection state) | Flips only that occurrence |
| Marking packed in the compiled list | The item is packed — every occurrence, counted once | Only that occurrence |
| The chips behind the name | Uniformly dark grey (always matching) | Dark grey when the occurrence matches the item's state, light grey when it differs |
| Persisted | The flag with the item; the item's one state | The flag (absent = false); the item's own state |

A hand-edited file whose singleton occurrences disagree loads as ONE state (the item's state — first occurrence wins); within a session the synchronization makes disagreement impossible.

## The display form

The panels group flat: one header per label, the items carrying it beneath. An item with multiple labels appears under each. Behind every item's name, its labels render as grey chips: ` [Ausland] [Karten]` — dark grey when the occurrence's state matches, light grey when it differs. The quantity always stays in the row; the name truncates before the chips do.

## The migration (one-time, external)

```text
categories: ['Ausland: Karten']  →  labels: [Ausland, Karten] + singleton: true
categories: ['Kulturbeutel']     →  labels: [Kulturbeutel]              (no singleton key)
exact duplicates (same name+labels) → merged, first wins, reported
```

The app contains no migration code; after the conversion only the label format exists.

## Invariants (pinned by tests)

1. A saved configuration stores `labels` per item; no category fields, no ": ", no top-level category list.
2. The load deduplicates: same (name, labels) entries merge; same name with different labels remain.
3. A singleton's toggles synchronize: one selection state, one packed state, flipping under any label flips everywhere.
4. A non-singleton's occurrences toggle independently; the persisted file keeps the item's own state.
5. The chips render behind the name, dark when matching, light when differing; the quantity never leaves the row.
6. The tree/collapse/selection/recalc/scroll behaviors hold on the flat label model.
