# Research: Label-Based Item Storage (revision 3)

## R1 — The label model (decision)

**Decision**: `Item` carries `labels: Vec<String>` (one or more, order preserved as written) and `singleton: bool`. The old "X: Y" category splits into the two labels "X" and "Y" at migration time; the app never splits anything (no separator exists in the new format). `primary_category`, `in_category`, and `category_layers` are deleted; every consumer reads the labels directly.

**Rationale**: The user's format is the model (revision 2's decision, unchanged); the singleton flag joins it as a plain per-item field (FR-014).

**Alternatives considered**: Deriving "singleton" from having multiple labels — rejected: the user explicitly wants it a per-item choice (FR-009/FR-011/FR-012 both exist).

## R2 — The flat row model (decision)

**Decision**: `PanelRow` becomes two variants: `Label(String)` (a label header) and `Item(usize)`. `left_rows`/`right_rows` build one header per distinct label (sorted), the items carrying it beneath; an item appears under EVERY label it carries. The Group/Category variants and the two-layer tree die. Collapse: `collapsed_labels: Vec<String>` (session state) replaces both `collapsed_categories` and `collapsed_groups`. Group-selection and group-recalc resolve to "items carrying this label" — a direct `labels.contains` check.

**Rationale**: FR-007's flat grouping; the existing scroll/sticky machinery operates on the flat row list unchanged (one header level instead of two — the pin, the cursor-follow, and the last-row guarantee carry over).

**Alternatives considered**: Keeping a two-level render (label > item) — rejected: labels are peers by design; the chips (R5) carry the multi-label context inside the row.

## R3 — The occurrence state for non-singletons (decision)

**Decision**: `AppDrawState` gains `occurrence_overrides: std::collections::BTreeMap<(usize, String), bool>` — a per-session override of an item's toggle for ONE (item, label) occurrence. The effective state of an occurrence = the override if present, else the item's own state. Toggling an occurrence: a singleton flips the item's own state (and clears any stale overrides for it); a non-singleton writes the override. The persisted files store the item's own state only; on load, overrides start empty (a hand-edited disagreeing singleton normalizes to the item's state — FR-014's first-occurrence-wins is exactly "the item's state is the state").

**Rationale**: FR-011/FR-012 need both semantics; the override map keeps the persisted format simple (one item, one state) while giving non-singletons per-occurrence freedom. The map is session state, like the collapse state.

**Alternatives considered**: Persisting per-label states in the yaml — rejected (the plan's complexity note): the human-editable format stays one-item-one-state; per-occurrence divergence is a within-session working mode.

## R4 — The yaml schemas (decision)

**Configuration file** — `SavedItem`:

```yaml
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
```

`labels` is required (at least one — an empty list rejects); `singleton` is optional-in-format with `default` + `skip_serializing_if` when false (single-label items and plain non-singletons keep the file clean; the flag is meaningful only for multi-label items). `AppConfig` loses its top-level `categories`.

**Trip file** — `TripItem` gains `labels: Vec<String>` and `singleton: bool` (same optionality) alongside name/packed/quantity; the trip snapshot stores the item's own state (R3).

**Decision**: `deny_unknown_fields` stays on both — the old `categories:` field rejects with serde's unknown-field error (FR-004).

**Rationale**: The singleton default-true-on-migration, default-false-on-fresh-write asymmetry lives in the migration, not the schema: the schema's serde default is `false`; the migration writes `singleton: true` for multi-label items explicitly.

**Alternatives considered**: Making `singleton` required — rejected: it would force noise on the 176 single-label items in the real data.

## R5 — The chips (decision)

**Decision**: A shared helper renders the chip spans for an item row: for each label (in the item's list order), a span ` [Label]` — `Color::DarkGray` when the occurrence's effective state (R3) matches the item's own state, `Color::Gray` when it differs. The chips sit between the name and the quantity; the row's truncation budget accounts for them (the name truncates first, then the chips, never the quantity — SC-009). Both panels use the helper; the compiled list shows the same chips.

**Rationale**: FR-013's match/differ semantics; `Color::Gray` (light grey, ANSI 37) and `Color::DarkGray` (ANSI 90) both exist in ratatui 0.30 (verified). A singleton's overrides are always clear, so its chips are uniformly dark — the visual signature of synchronization.

**Alternatives considered**: Chips before the name — rejected: the name is the primary content; the chips are context (the user said "behind an items name").

## R6 — The dedup (decision)

**Decision**: The dedup runs on load in `config_io.rs`: the key is (name, labels) with the labels compared as an ordered list exactly as written; the first occurrence wins; later exact matches are dropped. The six same-name-same-labels quantity-mode variants in the real data merge (first wins — the migration report lists them). Same-name-different-labels entries (e.g. the two "Zahnbürste" variants with different labels) remain separate items.

**Rationale**: FR-005; the verified data (206 → 182). Doing it on load (not save) means the in-memory list is always the deduplicated truth and the save writes it once.

**Alternatives considered**: Dedup on save only — rejected: the UI would show duplicates until the first save; the load is the single entry point.

## R7 — The dialog's label assignment + singleton toggle (decision)

**Decision**: The add/edit dialog's category field becomes a labels field: the existing picker lists the distinct labels (Left/Right cycles, Space toggles membership — the multi-select the old dialog had for categories, now for labels), plus free-text entry adds a new label. At least one label is required to save. A Singleton field (Space toggles) sits after the labels; it defaults to false for new items and shows the item's flag when editing.

**Rationale**: FR-009; the dialog already has the multi-select interaction pattern (the old `selected_categories` machinery returns, repurposed for labels); the singleton is one more toggle field in the existing cycle.

**Alternatives considered**: A separate label-manager screen — rejected: out of scope; the dialog's picker plus free text covers the cases.

## R8 — The reconcile (decision)

**Decision**: `reconcile` matches trip items to config items by name (case-insensitive, as today) and updates the trip item's labels and singleton flag from the config item; packed and quantity survive as today. The trip file's own labels are the snapshot; the config is the input.

**Rationale**: FR-003; the same by-name contract as today, extended to the new fields.

**Alternatives considered**: Matching by (name, labels) — rejected: the trip file is a snapshot of an earlier config; by-name remains the stable join key.

## R9 — What must NOT change (the behavior contract)

The collapse semantics (per label group now), the group-selection and group-recalc actions, the sticky-header pin (with the cursor-never-displaced retry and the last-row guarantee), scrolling to both extremes in both panels, the cursor highlight on every row type, the add/edit dialog's field cycling, the quantity/recalc model, the status line, and the hints/help system. The existing tests pin these; the PTY pass re-verifies them on the flat model.

**Decision**: The implementation updates the behavior tests' constructors (category → labels) without weakening assertions, and re-runs the PTY checks (flat grouping, synchronized toggles, chips, scroll extremes).

## R10 — Test impact (decision)

- `config_io.rs`: rewritten — strict label load, dedup on load (206→182-style), singleton optionality, round trip, old-format rejection.
- `state.rs`: the flat row model, the singleton toggle synchronization, the non-singleton override, the chip match/differ model (a pure helper test), collapse/select/recalc per label.
- The category-era tests (two-layer tree, group/sub-category collapse, multi-membership) are deleted with the model; the scroll/sticky tests carry over with label-based rows.

**Rationale**: Tests follow the model (Constitution III); dead tests for dead behavior violate Principle I.
