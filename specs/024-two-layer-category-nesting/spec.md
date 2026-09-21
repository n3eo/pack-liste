# Feature Specification: Two-Layer Category Nesting

**Feature Branch**: `024-two-layer-category-nesting`

**Created**: 2026-09-21

**Status**: Draft

**Input**: User description: "Implement two nesting layers. Split all categories by ': ' and make this the new layer. Ausland: Karten → Ausland > Karten > Führerschein international x1"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Categories display as two nested layers (Priority: P1)

A user opens the packing list and sees the categories organized as a two-level tree instead of one flat level: every category name that contains the separator ": " is split into a top-layer group and a sub-category. The left panel shows the top layer as the outer grouping and the sub-category as the inner grouping, with the items under their sub-category. The compiled list on the right shows the same two-layer structure. A category without the separator (if any exists) appears as a top-layer group with no sub-layer, its items directly beneath. The user's example: "Ausland: Karten" becomes the group "Ausland", the sub-category "Karten", and beneath it "Führerschein international x1".

**Why this priority**: The visible two-layer tree is the core of the request — the user explicitly shows the expected rendering ("Ausland > Karten > Führerschein international x1").

**Independent Test**: A list whose categories use the "X: Y" form renders as the two-layer tree: top-layer headers, sub-category headers beneath them, items beneath their sub-category.

**Acceptance Scenarios**:

1. **Given** a category "Ausland: Karten" containing the item "Führerschein international", **When** the list is displayed, **Then** a top-layer header "Ausland" is shown, a sub-category header "Karten" beneath it, and the item beneath the sub-category.
2. **Given** two categories sharing the same top layer ("Ausland: Karten" and "Ausland: Handgepäck"), **When** the list is displayed, **Then** both sub-categories appear under the single "Ausland" group, not as two separate top-level entries.
3. **Given** a category without the ": " separator, **When** the list is displayed, **Then** it appears as a top-layer group with its items directly beneath (no empty sub-layer is invented).
4. **Given** the compiled list on the right panel, **When** it is displayed, **Then** it uses the same two-layer grouping as the left panel.

---

### User Story 2 - Navigation and collapse work on both layers (Priority: P1)

A user navigates the two-layer tree with the cursor: Up/Down walk the rows in display order (top-layer header, its sub-category headers, their items), and the collapse action works on both layers — collapsing a top-layer group hides its sub-categories and items; collapsing a sub-category hides its items. The selection actions (select item, select whole category) work per layer: selecting a top-layer group selects everything beneath it; selecting a sub-category selects its items. The cursor never lands on a hidden row, and expanding restores the previous view.

**Why this priority**: A tree that cannot be navigated or collapsed is a regression against the current single-layer behavior — the interaction model must follow the new structure.

**Independent Test**: With a multi-group list, the cursor walks every visible row in order; collapsing a top-layer header hides its sub-tree; collapsing a sub-category hides only its items; selection at each layer selects the expected item set.

**Acceptance Scenarios**:

1. **Given** the two-layer tree fully expanded, **When** the cursor moves down from the top, **Then** it visits the top-layer header, then each sub-category header, then each item, in display order.
2. **Given** the cursor on a top-layer header, **When** collapse is triggered, **Then** the group's sub-categories and items disappear and the cursor stays on the group's header.
3. **Given** the cursor on a sub-category header, **When** collapse is triggered, **Then** only that sub-category's items disappear; the sibling sub-categories and the top-layer header remain.
4. **Given** a collapsed top-layer group, **When** the category-selection action is triggered on it, **Then** all items beneath the group (across its sub-categories) change selection state together.
5. **Given** a collapsed group anywhere in the tree, **When** the cursor navigates past it, **Then** no hidden row is visited.

---

### User Story 3 - The stored format is unchanged (Priority: P2)

A user's existing configuration and packing-list files load exactly as before: the category strings (with their ": " separators) are stored verbatim; the two-layer structure is derived at display time, not persisted as a new format. Saving and reloading a list produces byte-identical category values. A user who hand-edits a file keeps full control — the separator convention is the only contract.

**Why this priority**: The persisted files are a human-editable contract (established in earlier specs); the nesting must be a view over the existing data, not a migration.

**Independent Test**: A list saved before the feature loads identically after it; a save-load round trip preserves every category string exactly.

**Acceptance Scenarios**:

1. **Given** a configuration file with categories in the "X: Y" form, **When** it is loaded, **Then** every category string is preserved verbatim (no rewriting, no migration).
2. **Given** a list edited in the application (items added, categories changed), **When** it is saved and reloaded, **Then** the category strings are byte-identical to what was loaded.
3. **Given** the add/edit dialog's category picker, **When** the user picks a category, **Then** the full "X: Y" string is what the item receives.

---

### Edge Cases

- What happens when a category contains more than one ": " (e.g. "A: B: C")?
  **Answer**: The split is on the first separator only: the part before it is the top layer, everything after it is the sub-category ("A" > "B: C"). No deeper nesting is invented.
- What happens when two different top layers have sub-categories with the same name (e.g. "Reiseart: Auto" and "Unternehmungen: Auto")?
  **Answer**: They remain distinct sub-categories under their distinct top-layer groups; the full category string remains the identity, the layers are only a display grouping.
- What happens when a top layer has exactly one sub-category (e.g. "Kulturbeutel: Kulturbeutel")?
  **Answer**: The two layers still render (group header, then its single sub-category header); no special collapsing of redundant layers.
- What happens to the sticky category header on scroll?
  **Answer**: The sticky-header behavior applies per visible header level: when the viewport starts mid-group, the enclosing header (top layer or sub-category, whichever was scrolled past) is pinned so no item is orphaned.
- What happens to the recalculate scopes (item / category / all)?
  **Answer**: The category scope continues to operate on the full category string; recalculating "the current category" recalculates the sub-category under the cursor (and a top-layer header recalculates everything beneath the group).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every category string containing ": " MUST be split on the first occurrence into a top layer and a sub-category for display; the stored string MUST remain unchanged.
- **FR-002**: Both panels MUST render the two-layer tree: top-layer headers, sub-category headers beneath their group, items beneath their sub-category.
- **FR-003**: Categories without the separator MUST render as top-layer groups with their items directly beneath.
- **FR-004**: The cursor MUST navigate the visible rows in display order and MUST never land on a row hidden by collapse.
- **FR-005**: The collapse action MUST work on both layers: a top-layer header collapses its whole sub-tree; a sub-category header collapses only its items.
- **FR-006**: The category-selection action MUST work on both layers: a top-layer header selects all items beneath the group; a sub-category header selects its items.
- **FR-007**: The persisted configuration and packing-list formats MUST be unchanged: category strings are stored and reloaded verbatim.
- **FR-008**: The add/edit dialog's category picker MUST offer and assign the full category string verbatim.
- **FR-009**: The sticky-header scroll behavior MUST apply to the two-layer tree: no visible item is orphaned from its enclosing header.
- **FR-010**: The recalculate scopes MUST keep their meaning: the item scope recalculates the item, the category scope recalculates the category under the cursor (sub-category, or everything beneath a top-layer header), the all scope recalculates everything.

### Key Entities

- **Top layer**: The part of a category string before the first ": "; the outer grouping level.
- **Sub-category**: The part after the first ": "; the inner grouping level. For separator-free categories, no sub-category exists.
- **Category string**: The full stored value ("X: Y"); the item's identity and the persisted contract. The layers are a display derivation, never stored.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every category containing ": " renders as the two-layer tree in both panels; the user's example ("Ausland: Karten" with "Führerschein international") displays as group "Ausland", sub-category "Karten", item beneath.
- **SC-002**: Categories sharing a top layer appear under one group (e.g. all three "Ausland: …" categories under a single "Ausland" header).
- **SC-003**: Navigation visits every visible row in display order and no hidden row, at every collapse state.
- **SC-004**: Collapse and selection work at both layers with the expected item sets.
- **SC-005**: A save-load round trip preserves every category string byte-identically; files written before the feature load without error.
- **SC-006**: The full test suite passes and all existing behaviors not related to the grouping (quantities, packing, persistence) are unchanged.

## Assumptions

- The separator is exactly ": " (colon + space), as the user specified; categories in the existing data all use it, and separator-free categories are handled as the flat case.
- The split is on the first occurrence only; no deeper nesting is introduced.
- The persisted file formats are frozen (established in earlier specs); the nesting is purely a display-time derivation.
- The existing single-layer interactions (collapse, select, recalculate) extend to the two layers; no new key bindings are introduced.
- The category picker in the add/edit dialog continues to list full category strings; a hierarchical picker is out of scope.
