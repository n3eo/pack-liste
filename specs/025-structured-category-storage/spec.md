# Feature Specification: Label-Based Item Storage

**Feature Branch**: `025-structured-category-storage`

**Created**: 2026-09-21 (revised the same day)

**Status**: Draft (revision 3 — adds the singleton concept and the label-chip display over the label redesign)

**Input**: User description: "Update spec 25 to refactor the categories to labels. Split ': ' each becomes a label, remove the category field. One item can have multiple labels. Deduplicate the list so each item is unique. In the UI and the compiled list all items are treated individually." + "Add support for an item with multiple categories to be a 'singleton'. A singleton is synchronized between all its labels when toggling. Show all labels in dark/light grey behind an item's name."

> **Supersession note**: This revision replaces the previous
> category/subcategory design in this spec. The persisted formats
> change as before (over the freezes of specs 013/024) — the strict
> single-format principle carries over: exactly one current format,
> no compatibility layer. The earlier revision's two-field design is
> entirely superseded: there are no categories and no subcategories,
> only labels.
>
> **Revision 3 note**: revision 2's "every item is treated
> individually" is refined, not reversed: a multi-label item is still
> ONE item (one quantity, one edit), but its toggles — selection and
> packed — are SYNCHRONIZED across all its labels when it is a
> singleton. The singleton flag is per item, persisted, and the
> default for migrated multi-label items.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Items carry labels, not categories (Priority: P1)

A user opens their configuration file and finds no category fields: each item declares a list of `labels` — plain strings with no separator convention. The old "Ausland: Karten" becomes two labels, "Ausland" and "Karten", on the item. An item can carry any number of labels (one or more); the labels are the item's entire grouping. The trip file mirrors this: each trip item carries the labels of the packed thing. A user hand-editing the file writes plain label strings; there is no hierarchy, no separator, no special characters.

**Why this priority**: The storage model is the essence — everything else (grouping, display, dedup) builds on the labels.

**Independent Test**: A configuration whose items declare `labels` loads; the application groups and displays them; a save produces the same labels back.

**Acceptance Scenarios**:

1. **Given** an item previously stored with the category "Ausland: Karten", **When** the label format is written, **Then** the item carries the labels "Ausland" and "Karten".
2. **Given** an item with several groupings to express, **When** it is stored, **Then** it simply has several labels (e.g. `labels: [Ausland, Karten, Dokumente]`) — one item, many labels, no primary/secondary distinction.
3. **Given** the configuration file, **When** it is loaded, **Then** no category or subcategory field exists anywhere; the labels are the item's only grouping data.
4. **Given** a trip file, **When** it is saved and reloaded, **Then** each trip item carries its labels.
5. **Given** a user hand-edits an item's labels, **When** the application loads it, **Then** the edit is honored exactly as written.

---

### User Story 2 - The item list is deduplicated; every item is individual (Priority: P1)

A user's configuration contains each real thing exactly once: entries that would be exact duplicates (the same name and the same labels) are merged into one. The verified data has 24 such duplicates today (e.g. "Ladekabel" five times, "Tickets" twice) — after the refactor each appears once. In the UI and the compiled list, every item is treated individually: an item with multiple labels is ONE item (one row, one quantity, one packed state), not one row per label. Selecting, packing, editing, and recalculating act on the item; the labels only determine where it appears in the grouping.

**Why this priority**: Deduplication is explicitly requested and changes the data (206 → 182 items for the real config); individual treatment is the semantic that makes multi-label items coherent.

**Independent Test**: A configuration with duplicate entries loads as the deduplicated set; an item with two labels appears in both label groups but is one editable thing with one quantity.

**Acceptance Scenarios**:

1. **Given** two identical entries (same name, same labels), **When** the configuration loads, **Then** exactly one item exists (the duplicate is gone; the save writes it once).
2. **Given** an item with the labels "Ausland" and "Karten", **When** the left panel and the compiled list render, **Then** the item appears under the "Ausland" group and under the "Karten" group — one item, one quantity, one edit (the toggle synchronization is the singleton story, US5).
3. **Given** the compiled list, **When** the same item is reachable through two labels, **Then** it is counted once — the statistics reflect the deduplicated items, not label occurrences.
4. **Given** two entries with the same name but different labels (e.g. "Zahnbürste" fixed-quantity and "Zahnbürste" automatic — both exist in the real data), **When** the configuration loads, **Then** both remain: they are different items (dedup merges only exact name+labels matches).

---

### User Story 3 - The UI groups by labels, flat (Priority: P1)

A user opens the main screen and finds the items grouped by label — a flat, single-level grouping: one header per label, the items carrying that label beneath it. An item with multiple labels appears under each of its labels' groups. The two-layer tree (group/sub-category) is gone: labels are peers, not a hierarchy. Collapse works per label group; selection and recalculation work per label group (acting on every item carrying that label); navigation and scrolling reach every row in both panels, with the sticky-header and cursor-highlight behavior intact.

**Why this priority**: The grouping is how the user sees and works the list; it must be complete, correct, and fully navigable on the new flat model.

**Independent Test**: With a labeled configuration loaded, the panels show one header per label with the right items beneath; collapse/select/recalc work per label; scrolling reaches both extremes in both panels.

**Acceptance Scenarios**:

1. **Given** items with the labels "Ausland" and "Karten", **When** the main screen renders, **Then** an "Ausland" header and a "Karten" header each list the items carrying them (a "Ausland, Karten" item appears under both).
2. **Given** a label group, **When** collapse is triggered on its header, **Then** the group's items hide; the cursor stays on the header.
3. **Given** a label header, **When** the group-selection action fires, **Then** every item carrying that label flips selection together.
4. **Given** a label header, **When** the recalculate action fires, **Then** every automatic item carrying that label recalculates.
5. **Given** either panel, **When** the user scrolls fully down and fully up, **Then** every row is reachable — the first label header at the top, the last item at the bottom — with the sticky header never displacing the cursor and the highlight following it.
6. **Given** the add/edit dialog, **When** the user assigns labels, **Then** the item carries the chosen labels (multiple selectable; at least one).

---

### User Story 4 - The old format is gone, cleanly (Priority: P2)

A user with the existing separator-format configuration runs the new version: the old format is not loaded — no silent reinterpretation. The migration is one-time and external (a conversion or a hand edit): each "X: Y" category becomes the labels "X" and "Y"; duplicates merge; from then on only the label format exists. No dual-format parsing, no compatibility layer, one current format.

**Why this priority**: The project's strict single-format principle (spec 013) carries over; the migration is a deliberate one-time step.

**Independent Test**: An old-format file fails to load with a clear error naming the expected structure; a labeled file loads; no code path accepts both.

**Acceptance Scenarios**:

1. **Given** a configuration in the old format (with `categories:`), **When** the application loads it, **Then** it is rejected with an error naming the expected fields.
2. **Given** the one-time migration, **When** the user applies it, **Then** the application loads and saves the label format from then on.
3. **Given** the codebase after this feature, **When** searched for the separator convention or category fields in the storage paths, **Then** nothing remains — labels are the only grouping data.

---

### User Story 5 - Singletons: multi-label items whose toggles synchronize (Priority: P1)

A user marks a multi-label item as a "singleton": toggling it — selecting it for the trip, or marking it packed — happens in lockstep across ALL its labels. Toggling the item under "Ausland" also toggles the same item under "Karten" (and every other label it carries); there is no way to pack the "Ausland" occurrence while leaving the "Karten" occurrence unpacked. The singleton flag is part of the item, persisted with it, and shown in the UI. A non-singleton multi-label item keeps independent toggles per label occurrence (revision 2's behavior). The user sees which case an item is: every item's labels render as small chips behind its name, in grey — dark grey for labels whose occurrence currently matches the item's toggled state, light grey when that occurrence's state differs (only possible for non-singletons).

**Why this priority**: The singleton is the user's explicit model for "this is one physical thing appearing in several groups" — the most common multi-label case (one toothbrush listed under two labels is ONE toothbrush); without synchronization the compiled list would double-count physical things.

**Independent Test**: A singleton item with two labels: toggling it under either label flips both occurrences; a non-singleton with two labels: toggling flips only the occurrence under the active label; the chips behind each item's name show all its labels, greyed by match/differ.

**Acceptance Scenarios**:

1. **Given** a singleton item with the labels "Ausland" and "Karten", **When** the user toggles its selection under "Ausland", **Then** the item's selection flips for BOTH occurrences (the "Karten" occurrence shows the same state immediately).
2. **Given** a singleton item, **When** the user marks it packed in the compiled list, **Then** the packed state applies to the item — every occurrence, every group, and the statistics count it once as packed.
3. **Given** a NON-singleton item with the labels "Ausland" and "Karten", **When** the user toggles it under "Ausland" only, **Then** only the "Ausland" occurrence changes; the "Karten" occurrence keeps its state.
4. **Given** any item, **When** it renders in either panel, **Then** all its labels appear behind its name as grey chips — dark grey when the occurrence's toggle state matches the item's, light grey when it differs.
5. **Given** the add/edit dialog, **When** the user assigns multiple labels, **Then** they can mark the item a singleton or not; the choice persists with the item.
6. **Given** the migration, **When** an old multi-part category becomes multiple labels, **Then** the item is a singleton by default (one physical thing, synchronized).

---

### Edge Cases

- What happens to entries with the same name but different labels or quantities (the six "variant" names in the real data, e.g. two "Zahnbürste" entries differing in quantity mode)?
  **Answer**: They are different items and both remain. Dedup merges only exact matches (same name AND same labels); anything else is a distinct thing the user keeps for review.
- What happens when dedup merges entries that differ in a non-key field (same name + same labels, but one fixed-quantity and one automatic — the real data has this, e.g. two "Zahnbürste" entries)?
  **Answer**: Same name + same labels IS the duplicate key — they merge. The merge keeps the FIRST occurrence's settings, and the migration reports every merged pair with the differing fields, so nothing is lost silently.
- What happens when a label appears on no item (an orphaned label)?
  **Answer**: Labels come from items only — there is no separate label list, so an orphaned label cannot exist.
- What happens when an item has one label that is also another item's second label (e.g. "Technik" as a top-level and as a second-level)?
  **Answer**: Labels are flat strings — both items simply carry the label "Technik" and appear under the same header. The old hierarchy's meaning is gone by design.
- What happens to the collapse state?
  **Answer**: Session state keyed by label name, as today; not persisted, unaffected by the format change.
- What happens to the sticky headers and scrolling?
  **Answer**: The same row model, one header level instead of two — the pin, the cursor-follow, and the last-row guarantee carry over unchanged and are re-verified.
- What happens when a singleton's labels disagree (e.g. a hand-edited file has one occurrence packed and another not)?
  **Answer**: The load normalizes a singleton to ONE state (first occurrence wins); the migration report lists any normalized item. Within a session the synchronization makes disagreement impossible.
- What happens when a single-label item is marked a singleton?
  **Answer**: Harmless — with one label there is nothing to synchronize; the flag persists and simply has no visible effect until a second label is added.
- What happens when the user removes a label from a singleton in the edit dialog?
  **Answer**: The item keeps its singleton flag; the removed label's occurrence disappears with the label. Removing the LAST label is rejected (at least one label required).
- What happens to the chips when the row is too narrow to show them all?
  **Answer**: The chips truncate after the name (the existing row-truncation rule with the visible indicator); the full label set is visible in the edit dialog.
- What happens when a group action (select-label, recalc-label) hits a singleton?
  **Answer**: The group action acts on the item (as it always does); because the item is one entity, the singleton's synchronized state follows — the item is selected/packed once, not once per label.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The configuration file MUST store each item's grouping as a `labels` list of plain strings; no category, subcategory, or separator convention MAY exist in the stored format.
- **FR-002**: An item MAY carry one or more labels; the labels are the item's only grouping data, with no hierarchy or ordering semantics beyond their list order.
- **FR-003**: The trip file MUST store each item's labels alongside its other fields.
- **FR-004**: The application MUST reject a configuration or trip file that does not match the label format (unknown or missing fields), with an error naming the expected structure — no silent reinterpretation of any older format.
- **FR-005**: The item list MUST be deduplicated: entries with the same name and the same labels are one item. Entries differing in name or labels remain separate items.
- **FR-006**: In the UI and the compiled list, every item MUST be treated individually: one row per item per label group it appears in, with selection, packed state, quantity, and editing acting on the single item everywhere.
- **FR-007**: The panels MUST group items by label — one flat header per label, the items carrying it beneath; an item appears under every label it carries.
- **FR-008**: Collapse, group-selection, and group-recalculation MUST work per label group (acting on every item carrying that label), with the cursor and scrolling behaviors (sticky header, highlight, both-extremes reachability) unchanged.
- **FR-009**: The add/edit dialog MUST let the user assign one or more labels to an item (at least one required) and MUST let the user mark a multi-label item as a singleton or not; the flag persists with the item.
- **FR-010**: The migration from the old format MUST be one-time and external; no dual-format parsing or compatibility layer MAY remain in the code.
- **FR-011**: An item marked as a singleton MUST have its toggles — selection and packed — synchronized across all its labels: toggling any occurrence toggles the item everywhere, and the item has exactly one selection state and one packed state.
- **FR-012**: An item NOT marked as a singleton MUST keep independent toggle states per label occurrence.
- **FR-013**: Every item's labels MUST render behind its name as grey chips in both panels — dark grey when the occurrence's toggle state matches the item's, light grey when it differs.
- **FR-014**: The singleton flag MUST be persisted in the configuration file (and carried in the trip file's snapshot semantics); a load normalizes a disagreeing singleton to one state, first occurrence winning.

### Key Entities

- **Label**: A plain string grouping an item. Flat — no hierarchy, no special characters, no separator. An item carries one or more.
- **Item**: The individual thing — name, labels, quantity, packed, selected. Deduplicated by (name, labels); treated as one entity everywhere it appears.
- **Label group**: The set of items carrying a given label; the unit of collapse, group-selection, and group-recalculation in the UI.
- **Singleton**: A per-item flag: the item's toggles are synchronized across all its labels (one selection state, one packed state). A non-singleton multi-label item toggles per occurrence.
- **Label chip**: The visual form of an item's labels behind its name — grey, dark when the occurrence's state matches the item's, light when it differs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A saved configuration stores `labels` per item (and per trip item); no category/subcategory field and no ": " appears anywhere.
- **SC-002**: The item list is deduplicated — the real configuration's 206 entries load as 182 items (the 24 exact duplicates merged); a save writes each once.
- **SC-003**: An item with multiple labels appears under each of its labels' groups but is one item: selecting/packing/editing it anywhere changes it everywhere; the statistics count items, not label occurrences.
- **SC-004**: The panels show one flat header per label with the correct items beneath; collapse, group-selection, and group-recalculation work per label.
- **SC-005**: Scrolling reaches the first label header and the last item in both panels, in every collapse state, with the sticky header and cursor highlight intact.
- **SC-006**: An old-format file is rejected with a clear error; a labeled file round-trips its labels verbatim.
- **SC-007**: The full test suite passes; all behaviors not related to the grouping model are unchanged.
- **SC-008**: A singleton's toggles are synchronized: toggling under any label flips every occurrence, and its selection/packed state is one value; a non-singleton's occurrences toggle independently.
- **SC-009**: Every item shows its labels as grey chips behind its name in both panels, dark when matching, light when differing; the chips never push the quantity out of the row.

## Assumptions

- The verified real data grounds the dedup design: 206 entries, 182 unique (name, labels) pairs, 24 exact duplicates (identical in all fields or differing only in quantity mode), zero multi-category items today — the label model's multi-label capability is new power, not a migration requirement.
- Same-name-different-labels entries (and same-name-same-labels entries differing in quantity mode) are distinct items kept for review; only exact (name, labels) matches merge, first occurrence wins, and the migration reports every merge.
- The strict single-format principle (spec 013) carries over: one current format, no compat layer, one-time migration.
- The flat label grouping replaces the two-layer tree; the collapse/selection/recalc/scroll semantics carry over per label group.
- The trip file's reconcile-by-name behavior carries over, matching on labels.
- The add/edit dialog's label assignment supports multiple labels; the exact picker interaction (multi-select from existing labels plus free text) is a planning decision.
- The singleton flag defaults to TRUE for migrated multi-label items (an old "X: Y" category is one physical thing under two labels); single-label items carry the flag with no effect.
- The occurrence state for non-singletons (per-label selection/packed) is session state derived from the item's toggles; the persisted file stores the item's own state, and the load normalizes singletons (first occurrence wins).
- The chip greys are the terminal's dark grey and light grey (or the closest available shades); the exact shades are a planning decision, the match/differ semantics are not.
