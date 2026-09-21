# Contract: Two-Layer Display Tree

The user-facing contract of the nested display: what the user sees and can do at each layer. The stored category strings are NOT part of this contract's changes — they are frozen (see the data model).

## The split rule

Every category string is displayed as two layers, split on the FIRST ": ":

```text
"Ausland: Karten"           →  Group "Ausland"  >  Sub-category "Karten"
"Dokumente_Karte: Unterhaltung" → Group "Dokumente_Karte" > Sub-category "Unterhaltung"
"Kulturbeutel: Kulturbeutel"   → Group "Kulturbeutel" > Sub-category "Kulturbeutel"
"OhneTrenner"                  → Group "OhneTrenner" (items directly beneath, no sub-row)
"A: B: C"                      → Group "A" > Sub-category "B: C"
```

## The rendered tree (both panels)

```text
▼ Ausland                        ← Group header (top layer)
  ▼ Karten                       ← Sub-category header (indented)
    ✓ Führerschein international x1
    ✓ Auslandreiseversicherung x1
  ▼ Handgepäck                   ← Sibling sub-category under the same group
    ✓ T-Shirt x6
▼ Dokumente_Karte
  ▼ Dokumente/Karten
    ✓ Kreditkarte x6
```

- Group headers show the top layer; sub-category headers show the part after ": ", indented one level.
- The compiled list (right panel) shows the same tree over the selected items, with its packing indicators on the headers.
- Items with multiple categories appear under each sub-category they belong to (existing multi-membership behavior, now per layer).

## Interactions per layer

| Cursor on | `C` (collapse) | `Ctrl+Space` (select) | `R` (recalc item) | `Ctrl+R` (recalc category) |
|-----------|-----------------|------------------------|--------------------|-----------------------------|
| Group header | hides the whole sub-tree | selects/deselects ALL items under the group | recalculates every item under the group | same as R there |
| Sub-category header | hides its items only | selects/deselects the sub-category's items | recalculates the sub-category's items | recalculates the sub-category |
| Item | (unchanged: its category) | (unchanged) | recalculates the item | recalculates its sub-category |

- Up/Down walk the visible rows in display order (group, sub-category, items), never landing on a hidden row.
- The sticky header on scroll pins the outermost scrolled-off header (group or sub-category) so no item is orphaned from its context.
- Navigating the compiled list expands the cursor item's sub-category AND its group on the left, collapsing others (existing sync, extended).

## The frozen storage contract

- Category strings are stored and reloaded verbatim; the split is display-only.
- A save-load round trip is byte-identical for every category string.
- The add/edit dialog's picker lists and assigns full category strings.

## Invariants (pinned by tests)

1. The split uses the first ": " only; the stored string is never rewritten.
2. Categories sharing a top layer render under one group header.
3. Separator-free categories render as a group with items directly beneath.
4. Collapse at the group level hides the sub-tree; at the sub-category level, the items only.
5. Selection/recalculation at the group level covers every item under the group.
6. Navigation visits visible rows only, in display order.
7. The sticky header pins the outermost missing header.
