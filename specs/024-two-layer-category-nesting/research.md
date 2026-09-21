# Research: Two-Layer Category Nesting

## R1 — The data shape (verified)

All 35 categories in the real config use the "X: Y" form; 9 distinct top layers (Ausland, Dokumente_Karte, Ferienwohnung, Geschäftsreise, Kulturbeutel, Reiseart, Städtereisen, Technik, Unternehmungen). Duplicate sub-category names exist across groups (Auto ×2, Klamotten ×2, Sonstiges ×3) — the full string must remain the identity. One group has a single sub-category named like its group ("Kulturbeutel: Kulturbeutel").

**Decision**: Split on the FIRST ": " (before = top layer, after = sub-category, verbatim including any further ": "). Separator-free categories render as a group with items directly beneath (no invented sub-layer).

**Rationale**: Matches the user's example and the spec's edge cases; the data confirms no multi-separator strings exist today, but the rule handles them deterministically.

**Alternatives considered**: Splitting on every ": " into deeper nesting — rejected: the spec caps at two layers.

## R2 — The row model (decision)

**Decision**: `PanelRow` gains a group variant: `PanelRow::Group(String)` (the top layer) alongside `Category(String)` (now the sub-category, still carrying the FULL category string as its identity for collapse/selection/recalculation) and `Item(usize)`. The display order per group: `Group(top)`, then per sub-category `Category(full)`, then its items. `left_rows`/`right rows` build this order; the sub-category header renders only the part after ": " (indented), while its identity stays the full string.

**Rationale**: The existing single-layer code (collapse, selection, sticky, recalc) keys on the category string; keeping the full string as `Category`'s payload means those mechanisms keep working per sub-category unchanged, and the group is a new orthogonal level.

**Alternatives considered**:
- A nested tree struct with recursive rendering — rejected: the flat row list is what navigation, scroll, and sticky headers already consume; a tree would force a rewrite of all three.
- Encoding the layer into the string ("Ausland > Karten") — rejected: violates FR-007 (the stored string is the identity; display strings must not leak into state).

## R3 — Collapse at both layers (decision)

**Decision**: The existing `collapsed_categories: Vec<String>` holds FULL category strings (sub-category level, unchanged semantics). A new `collapsed_groups: Vec<String>` holds top-layer names. Collapse on a `Group` header toggles the group (hides its sub-categories and items); collapse on a `Category` header toggles the sub-category (hides its items only). The cursor clamps to the header it collapsed, as today.

**Rationale**: Two independent flags match the two independent layers; the existing vector's semantics are untouched (no migration, no format change).

**Alternatives considered**: One map from "layer:path" keys — rejected: two typed vectors are simpler and the state is not persisted anyway.

## R4 — Selection at both layers (decision)

**Decision**: `Ctrl+Space` on a `Group` header selects/deselects every item whose category's top layer matches; on a `Category` header it keeps today's per-category behavior (all items in that full-string category). The group-level selection derives from the top layer of each item's categories.

**Rationale**: FR-006's "top-layer header selects all items beneath the group" is exactly a top-layer match; the existing per-category path is reused for the sub-layer.

**Alternatives considered**: Selecting only visible (expanded) items — rejected: selection is a data operation, not a view operation; the current per-category selection also ignores collapse state.

## R5 — Sticky headers per level (decision)

**Decision**: `sticky_rows` pins the innermost enclosing header that scrolled off: if the viewport starts on an item, pin its sub-category header; if it starts on a sub-category header whose group header is also off-screen, pin the group header instead (the outermost missing header wins, so the tree context is always visible). One pinned row maximum, as today.

**Rationale**: FR-009 requires no orphaned items; pinning the outermost missing header gives the most context for the same one-row cost.

**Alternatives considered**: Pinning both levels (two rows) — rejected: costs an extra list row; the sub-category is visually indented under the pinned group, and the group alone suffices when the sub-category is visible.

## R6 — Recalculate scopes (decision)

**Decision**: `RecalculateScope::Category(String)` keeps its full-string meaning (a sub-category). A new `RecalculateScope::Group(String)` recalculates every item whose category's top layer matches. Plain `R` on a `Category` header → its category; on a `Group` header → the group; on an item → the item (unchanged). `Ctrl+R` resolves the cursor's sub-category (or the group when on a group header); `Ctrl+Shift+R` stays `All`.

**Rationale**: FR-010 keeps each scope's meaning; the group is the one new resolution the second layer introduces.

**Alternatives considered**: Reusing `Category` with the top-layer name — rejected: ambiguous against a full-string category; a distinct variant is explicit.

## R7 — The right panel and the sync (decision)

**Decision**: The right (compiled) panel builds the same two-layer tree over the selected items (grouped by primary category, as today, then split). `sync_left_view_to_right_cursor` expands the cursor item's sub-category AND its group on the left, collapsing others, as it does today per category.

**Rationale**: FR-002 requires the same tree on both panels; the sync's existing expand-the-current rule extends naturally.

**Alternatives considered**: Keeping the right panel flat — rejected: the spec's example shows the tree on the compiled list too.

## R8 — The add/edit dialog (decision)

**Decision**: The category picker is unchanged — it lists full category strings verbatim (FR-008). The displayed entry may show the split visually (group prefix dimmed), but the assigned value is the full string.

**Rationale**: FR-008 is explicit; a hierarchical picker is out of scope per the spec's assumptions.

**Alternatives considered**: A two-step picker (group, then sub-category) — rejected: out of scope, and the flat list with 35 entries is already navigable with Left/Right.
