# Feature Specification: Packing Tracker

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Implemented

**Input**: User description:
1. Every item shall have an editable quantity associated
2. Every item has a toggle to mark if it has already been packed or not
3. Use visual highlighting of non-packed items
4. Each category also is marked visually if not all items are packed

> **Alignment note (2026-09-20)**: This spec was reviewed against the implemented
> application and updated to describe the shipped behavior. Quantities are
> either automatically calculated from trip parameters (the default) or fixed
> per item; user edits apply to fixed-quantity items, while auto quantities are
> recalculated on every trip-parameter change. Visual feedback uses colored
> state symbols and a progress summary rather than full-row background colors,
> and category headers do not carry a packed/unpacked state.
>
> **Update (2026-09-20, spec 015)**: Two gaps this spec recorded are now
> closed. FR-002's fixed-quantity mode is now editable in the TUI (the
> edit dialog's Mode field; previously config-file only), and the
> original input point 4 (category marked when not all items packed) is
> restored as the compiled list's category-header packing indicator.
> Category headers now carry an all-packed/partially-packed marker;
> the "do not carry a packed state" sentence above is superseded.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Item Quantities: Automatic or Fixed (Priority: P1)

Every item has a quantity. By default the quantity is calculated automatically from the trip parameters (length, travel style, weather, activities). Items whose count does not scale with the trip (for example a toothbrush) can be marked as fixed-quantity; their quantity is set once and never recalculated. Fixed quantities persist across restarts; automatic quantities are recalculated on every load and are not persisted.

**Why this priority**: This allows users to accurately reflect their packing needs, ensuring they bring the right amount of each item.

**Independent Test**: This story can be tested independently by marking an item as fixed, editing its quantity, changing trip parameters, and verifying the fixed quantity is kept while auto quantities change.

**Acceptance Scenarios**:

1. **Given** the application is launched, **When** the user views the initial screen, **Then** each item displays its current quantity.
2. **Given** an item is marked as fixed-quantity, **When** the user edits its quantity, **Then** the new quantity is stored and persists across application restarts.
3. **Given** an item has automatic quantity, **When** the user changes trip parameters, **Then** the quantity is recalculated from the new parameters.
4. **Given** an item has automatic quantity, **When** the application exits and restarts, **Then** the quantity is recalculated (not restored from a stored value).

---

### User Story 2 - Toggle Packed Status (Priority: P1)

The user can mark items as packed or unpacked using a toggle on the compiled packing list. Packed items are visually distinct from unpacked items, and the system provides a summary of packing progress (total, packed, unpacked, percentage).

**Why this priority**: This helps users track their packing progress and ensures they don't forget essential items.

**Independent Test**: This story can be tested independently by toggling the packed status of an item and verifying the visual state and progress summary update.

**Acceptance Scenarios**:

1. **Given** a trip is resumed from a trip file, **When** the user views the packing list, **Then** the packed states from the file are restored.
2. **Given** the user toggles an item as packed, **When** they view the packing list, **Then** the item's state symbol is distinct (packed) and the progress summary updates.
3. **Given** the user toggles an item as unpacked, **When** they view the packing list, **Then** the item's state symbol is distinct (unpacked) and the progress summary updates.
4. **Given** the user exits the application, **When** they resume the trip, **Then** the packed status of all items persists via the trip file.

---

### User Story 3 - Visual Highlighting of Packed State (Priority: P2)

The system uses colored state symbols to indicate packed and unpacked items on both panels. This provides immediate feedback on packing progress while the list is being worked.

**Why this priority**: Visual feedback improves usability by making it easy to identify what still needs to be packed.

**Independent Test**: This story can be tested independently by toggling items and verifying that the visual state updates correctly.

**Acceptance Scenarios**:

1. **Given** an item is marked as unpacked, **When** the user views the packing list, **Then** the item's state symbol is shown in the unpacked color.
2. **Given** an item is marked as packed, **When** the user views the packing list, **Then** the item's state symbol is shown in the packed color.
3. **Given** the user toggles an item's packed state, **When** they view the packing list, **Then** the symbol and color change immediately.

---

### User Story 4 - Packing Progress Summary (Priority: P2)

The system shows a summary of packing progress on the compiled list: how many items are selected, how many are packed, how many are unpacked, and the completion percentage. This helps users quickly see how much packing work remains.

**Why this priority**: This improves efficiency by giving users an at-a-glance overview without scanning the whole list.

**Independent Test**: This story can be tested independently by marking some items as packed and verifying the summary updates.

**Acceptance Scenarios**:

1. **Given** the compiled list is displayed, **When** the user views the summary, **Then** it shows the selected total, packed count, unpacked count, and progress percentage.
2. **Given** the user packs or unpacks an item, **When** they view the summary, **Then** the counts and percentage update immediately.

---

### Edge Cases

- What happens if the user sets the quantity of an item to zero?
  **Answer**: Quantities are constrained to a minimum of 1; the calculation clamps to at least 1 and fixed quantities must be at least 1.
- How does the system handle very large quantities?
  **Answer**: The trip length input is capped (30 days) and the calculation base is capped (20), keeping displayed quantities in a compact range.
- What happens if the user toggles the packed status of a whole category at once?
  **Answer**: Category-level toggling applies to selection (adding/removing items to the trip), not to packed state; packed state is toggled per item.
- How does the system handle items with zero quantity?
  **Answer**: Quantities never drop below 1, so no zero-quantity handling is needed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST assign every item a quantity, calculated automatically from trip parameters by default.
- **FR-002**: System MUST allow items to be marked as fixed-quantity, whose value is set once, editable, and never recalculated.
- **FR-003**: System MUST provide a toggle for each item on the compiled list to mark it as packed or unpacked.
- **FR-004**: System MUST use colored state symbols to indicate packed and unpacked items.
- **FR-005**: System MUST persist packed status via the trip file and fixed quantities via the configuration file across application restarts; automatic quantities are recalculated, not persisted.
- **FR-006**: System MUST provide a summary of packing progress (selected total, packed, unpacked, percentage).

### Key Entities *(include if feature involves data)*

- **Item**: A packing item (e.g., "T-shirt", "Laptop") with a quantity (automatic or fixed) and packed status.
- **Quantity mode**: Whether an item's quantity is automatic (recalculated from trip parameters) or fixed (user-set, persistent).
- **Packed Status**: A boolean (packed/unpacked) for each item, persisted in the trip file.
- **Packing Progress**: A summary of how many items are packed vs. unpacked, with a completion percentage.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can toggle the packed status of any item in under 2 key presses.
- **SC-002**: Packed and unpacked items are visually distinct via colored state symbols.
- **SC-003**: 100% of packed statuses and fixed quantities persist across application restarts.
- **SC-004**: Users can view their packing progress summary at a glance on the compiled list.
- **SC-005**: Automatic quantities are never persisted; they are recalculated from the current trip parameters on every load.

## Assumptions

- Users have a basic understanding of terminal applications and navigation.
- The application is designed for a single user, as per the project constitution.
- The TUI uses standard keyboard navigation (e.g., arrow keys, Space for toggles).
- Visual feedback uses colored symbols that are distinguishable for users with color vision deficiencies.
- Most items scale with trip parameters; fixed quantities are the exception and are marked per item in the configuration file.
