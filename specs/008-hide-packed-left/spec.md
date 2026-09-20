# Feature Specification: Hide Packed Items on the Left Panel

**Feature Branch**: `008-hide-packed-left`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description: "do not show on the left side if a element was packed"

> **Re-invoked 2026-09-20** with: "Do not show the already packed marker on
> the left split". The existing spec already covers this: the packed
> indicator (✓/○) on the left panel is removed together with the packed
> items themselves (see the edge case "Does the packed indicator on the
> left panel disappear entirely?" and Assumptions). No spec change was
> needed; the feature pointer is set back to this spec.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Packed items disappear from the left panel (Priority: P1)

A user is working through their packing list. When they mark an item as packed (on the right panel), the item disappears from the left panel — the all-items tree. The left panel becomes a working list of what still needs attention: unpacked items. The right panel continues to show the full compiled list including packed items, so the user can still review or unmark them there.

**Why this priority**: This is the entire request — the left panel should reflect packing progress by hiding what is done, without losing any data.

**Independent Test**: Select items, mark one as packed on the right panel, and verify it vanishes from the left panel while remaining on the right; unmark it and verify it returns to the left panel.

**Acceptance Scenarios**:

1. **Given** the left panel shows an item that is selected for the trip, **When** the user marks that item as packed, **Then** the item is no longer shown on the left panel.
2. **Given** an item is packed and hidden from the left panel, **When** the user unmarks it as packed on the right panel, **Then** the item reappears on the left panel.
3. **Given** an item is packed, **When** the user views the right panel, **Then** the item is still shown there with its packed state.

---

### User Story 2 - Left panel stays consistent and navigable (Priority: P2)

A user navigates the left panel while items progressively disappear as they pack. The panel must stay consistent: category headers remain visible (even when all their items are packed), the cursor never lands on an invisible row, and navigation skips hidden items smoothly. Counts and any selection actions on the left panel apply only to the visible (unpacked) items.

**Why this priority**: Hiding rows changes what the cursor moves over; without consistent behavior, navigation would break or feel erratic exactly when the feature is being used.

**Independent Test**: With several items packed across different categories, navigate the left panel end to end; verify the cursor visits only visible rows, category headers stay in place, and no crash or jump occurs.

**Acceptance Scenarios**:

1. **Given** some items of a category are packed, **When** the user navigates the left panel, **Then** the cursor moves over the remaining unpacked items and the category header, skipping the hidden ones.
2. **Given** all items of a category are packed, **When** the user views the left panel, **Then** the category header is still shown (with no items under it).
3. **Given** the cursor is on an item that becomes packed (hidden), **When** the hiding takes effect, **Then** the cursor moves to a valid visible row.

---

### User Story 3 - Selection state is unaffected by hiding (Priority: P3)

A user's trip data is unchanged by this feature: hiding is purely a view filter on the left panel. Packed items remain selected for the trip, are saved to the trip file on exit, and are restored on resume. Toggling a category's selection on the left panel applies to the items as before — packed items are not silently deselected or reselected by the view.

**Why this priority**: The feature must not corrupt the trip; data integrity is what makes the view filter safe to use.

**Independent Test**: Pack items, exit, resume the trip; verify packed items are still in the trip file and still hidden on the left panel after resume.

**Acceptance Scenarios**:

1. **Given** items are packed and hidden from the left panel, **When** the user exits and resumes the trip, **Then** the packed items are restored with their packed state and remain hidden on the left panel.
2. **Given** packed items are hidden on the left panel, **When** the user toggles the category's selection, **Then** the packed items' selection state changes together with the rest of the category (they are not excluded from the toggle by being hidden).
3. **Given** the left panel hides packed items, **When** the user exits, **Then** the trip file contains all selected items including packed ones.

### Edge Cases

- What happens when every item is packed (the left panel would be empty of items)?
  **Answer**: Category headers remain; the panel shows the structure with no unpacked items. The user can still navigate headers and unmark items on the right panel.
- What happens when the cursor is on the last visible item and it becomes packed?
  **Answer**: The cursor moves to the previous visible row (or the nearest valid row), never onto a hidden row.
- What happens when an item is packed and unpacked rapidly while navigating?
  **Answer**: The row reappears at its original position in the category; the cursor stays on a valid row.
- Does the packed indicator on the left panel disappear entirely?
  **Answer**: Yes — since packed items are not shown, the packed indicator on the left panel becomes unreachable for packed items; packed state is managed on the right panel.
- What happens to the left panel's scroll position when rows disappear above the viewport?
  **Answer**: The scroll offset adjusts so the cursor stays visible; no blank gap or jump.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The left panel MUST NOT show items that are marked as packed.
- **FR-002**: The right panel MUST continue to show packed items with their packed state.
- **FR-003**: Category headers on the left panel MUST remain visible even when all their items are packed.
- **FR-004**: Navigation on the left panel MUST only visit visible rows (unpacked items and category headers); the cursor MUST never rest on a hidden row.
- **FR-005**: When the row under the cursor becomes hidden, the cursor MUST move to a valid visible row.
- **FR-006**: Hiding packed items MUST be a view filter only: selection state, packed state, trip file contents, and quantity calculations MUST be unaffected.
- **FR-007**: Category selection toggles on the left panel MUST apply to all items of the category, including packed (hidden) ones.
- **FR-008**: The hiding behavior MUST apply consistently after resuming a trip (packed items restored from the trip file are hidden on the left panel).

### Key Entities *(include if feature involves data)*

- **Left panel (all-items tree)**: The working view of the item catalog grouped by category; with this feature, filtered to unpacked items.
- **Packed state**: Per-item boolean, toggled on the right panel, persisted in the trip file.
- **View filter**: The rule "packed items are hidden on the left panel"; presentation-only, no data effect.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of packed items are absent from the left panel within one render cycle of being marked packed.
- **SC-002**: 100% of unpacked items remain visible on the left panel.
- **SC-003**: The cursor never rests on a hidden row during any navigation or packing sequence.
- **SC-004**: Trip files saved with this feature are identical in content to those saved without it (no data change).
- **SC-005**: After resuming a trip, all packed items are hidden on the left panel without user interaction.

## Assumptions

- Packed state is toggled on the right panel (the compiled list); the left panel's packed indicator becomes vestigial and is removed with this feature.
- The right panel always shows the full compiled list, packed and unpacked, as the source of truth for packed state.
- Hiding is automatic and always active on the left panel; no toggle or setting is added.
- Category headers stay visible so the structure remains recognizable and category-level actions remain reachable.
- Existing key bindings and screens are unchanged; only what the left panel renders changes.
