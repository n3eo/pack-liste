# Data Model: Harmonized Navigation Text

## Entities

### Canonical Binding

The single definition of a (key, action text) pair. Every hint line and the help screen reference these; no other string source exists.

| Field | Type | Rules |
|-------|------|-------|
| key | token | The key as displayed (e.g. `↑/↓`, `Ctrl+R`, `Space`). Unique within one meaning. |
| action | text | The canonical action wording (e.g. `Navigate`, `Recalc item`). Exactly one per (key, meaning). |

**Validation rules** (from spec FR-001/FR-003/FR-005):
- A key may appear multiple times only when the meanings are genuinely distinct (e.g. `Space` = Select on the left panel, Packed on the right panel); each distinct meaning is its own canonical entry.
- No two canonical entries may have the same (key, action) pair.
- Every canonical entry must be referenced by at least one surface composition or the help screen (no dead entries — Constitution I).

### Surface Composition

The ordered list of canonical bindings a surface displays. One composition per surface (LeftPanel, RightPanel, Welcome, Config, AddEditPopup, DeletePopup, ConfirmEditPopup, ModifierEditor, RawEditor).

| Field | Type | Rules |
|-------|------|-------|
| surface | enum | One of the nine existing surfaces. |
| bindings | list of canonical binding references | Ordered for display; references the canonical table by identity — never re-types the strings. |

**Validation rules** (from spec FR-002/FR-003):
- The composition contains exactly the surface's complete binding set: every binding the surface responds to, none it does not.
- No (key, action) pair appears twice within one composition.
- The two main panels' compositions are disjoint as rendered on one screen: a pair appears in at most one of them (the shared pairs live in the right panel's composition only).

### Help Agreement

The help screen's relationship to the canonical table (not a new entity — a derived view).

| Field | Type | Rules |
|-------|------|-------|
| key tokens | from the canonical table | Every key the help screen lists is a canonical key. |
| descriptions | long text | The help screen's verbose descriptions; each must contain its key's canonical action word (e.g. the `↑/↓` description contains "Navigate"). |

## State Transitions

None — the canonical table is static display data. The only "state" is the existing adaptive layout decision (pack rows to fit / collapse to the short help pointer), which is recomputed per frame from (width, height, entries) and is unchanged by this feature.

## Relationships

```text
Canonical Binding (1) ──referenced by── (N) Surface Compositions
Canonical Binding (1) ──token-sourced── (1) Help Screen entry
Surface Composition (1) ──rendered by── (1) hint line / hint rows
```

## Persistence

None. The canonical table and compositions are compile-time display data; nothing is written to or read from the configuration or trip files.
