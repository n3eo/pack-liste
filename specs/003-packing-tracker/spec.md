# Feature Specification: Packing Tracker

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description:
1. Every item shall have an editable quantity associated
2. Every item has a toggle to mark if it has already been packed or not
3. Use visual highlighting of non-packed items
4. Each category also is marked visually if not all items are packed

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Edit Item Quantities (Priority: P1)

The user launches the application and sees a list of items with their current quantities. The user can edit the quantity of any item to reflect how many of that item they need to pack (e.g., "T-shirts: 5").

**Why this priority**: This allows users to accurately reflect their packing needs, ensuring they bring the right amount of each item.

**Independent Test**: This story can be tested independently by editing the quantity of an item and verifying that the change persists.

**Acceptance Scenarios**:

1. **Given** the application is launched, **When** the user views the initial screen, **Then** each item displays its current quantity.
2. **Given** the user edits the quantity of an item, **When** they save the change, **Then** the quantity is updated and persists across application restarts.
3. **Given** the user sets the quantity of an item to zero, **When** they view the packing list, **Then** the item is hidden or clearly marked as excluded.

---

### User Story 2 - Toggle Packed Status (Priority: P1)

The user can mark items as packed or unpacked using a toggle. Packed items are visually distinct from unpacked items, and the system provides a summary of packing progress.

**Why this priority**: This helps users track their packing progress and ensures they don’t forget essential items.

**Independent Test**: This story can be tested independently by toggling the packed status of an item and verifying that the visual state updates.

**Acceptance Scenarios**:

1. **Given** the application is launched, **When** the user views the initial screen, **Then** all items are marked as unpacked by default.
2. **Given** the user toggles an item as packed, **When** they view the packing list, **Then** the item is visually distinct (e.g., green background) and the category’s visual state updates.
3. **Given** the user toggles an item as unpacked, **When** they view the packing list, **Then** the item is visually distinct (e.g., red background) and the category’s visual state updates.
4. **Given** the user restarts the application, **When** they view the packing list, **Then** the packed status of all items persists.

---

### User Story 3 - Visual Highlighting of Unpacked Items (Priority: P2)

The system uses visual highlighting to indicate unpacked items (e.g., red background) and packed items (e.g., green background). This provides immediate feedback on packing progress.

**Why this priority**: Visual feedback improves usability by making it easy to identify what still needs to be packed.

**Independent Test**: This story can be tested independently by toggling items and verifying that the visual state updates correctly.

**Acceptance Scenarios**:

1. **Given** an item is marked as unpacked, **When** the user views the packing list, **Then** the item is highlighted in red.
2. **Given** an item is marked as packed, **When** the user views the packing list, **Then** the item is highlighted in green.
3. **Given** a category contains both packed and unpacked items, **When** the user views the category, **Then** the category is highlighted in yellow to indicate partial packing.

---

### User Story 4 - Visual Marking of Categories (Priority: P2)

The system visually marks categories where not all items are packed (e.g., yellow background or warning symbol). This helps users quickly identify which categories still need attention.

**Why this priority**: This improves efficiency by allowing users to focus on categories that are not yet complete.

**Independent Test**: This story can be tested independently by marking some items in a category as packed and verifying that the category’s visual state updates.

**Acceptance Scenarios**:

1. **Given** a category where all items are packed, **When** the user views the category, **Then** the category is highlighted in green.
2. **Given** a category where some items are unpacked, **When** the user views the category, **Then** the category is highlighted in yellow with a warning symbol.
3. **Given** a category where no items are packed, **When** the user views the category, **Then** the category is highlighted in red with a warning symbol.

---

### Edge Cases

- What happens if the user sets the quantity of an item to a negative number?
  **Answer**: The system displays an error message and does not update the quantity until a valid (non-negative) number is provided.
- How does the system handle very large quantities (e.g., 1000+)?
  **Answer**: The TUI remains responsive, and the quantity is displayed in a compact format (e.g., "1K" for 1000).
- What happens if the user toggles the packed status of a category (e.g., marks all items as packed at once)?
  **Answer**: The system updates the packed status of all items in the category and marks the category as fully packed.
- How does the system handle items with zero quantity?
  **Answer**: Items with zero quantity are hidden by default but can be revealed via a toggle or filter.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to edit the quantity of each item in the packing list.
- **FR-002**: System MUST provide a toggle for each item to mark it as packed or unpacked.
- **FR-003**: System MUST use visual highlighting to indicate unpacked items (e.g., red background) and packed items (e.g., green background).
- **FR-004**: System MUST visually mark categories where not all items are packed (e.g., yellow background or warning symbol).
- **FR-005**: System MUST persist the packed status and quantities across application restarts.
- **FR-006**: System MUST provide a summary of packing progress (e.g., "5/10 items packed").

### Key Entities *(include if feature involves data)*

- **Item**: A packing item (e.g., "T-shirt", "Laptop") with an editable quantity and packed status.
- **Category**: A group of items (e.g., "Clothing", "Electronics") with a visual state indicating whether all items are packed.
- **Packed Status**: A boolean (packed/unpacked) for each item.
- **Quantity**: A numeric value for each item, editable by the user.
- **Packing Progress**: A summary of how many items are packed vs. unpacked.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can edit the quantity of any item in under 3 key presses.
- **SC-002**: Users can toggle the packed status of any item in under 2 key presses.
- **SC-003**: Unpacked items are visually distinct from packed items (e.g., red vs. green background).
- **SC-004**: Categories with unpacked items are visually distinct from fully packed categories (e.g., yellow vs. green background).
- **SC-005**: 100% of user edits (quantities and packed status) persist across application restarts.
- **SC-006**: Users can view their packing progress summary in under 5 seconds.

## Assumptions

- Users have a basic understanding of terminal applications and navigation.
- The application is designed for a single user, as per the project constitution.
- The TUI uses standard keyboard navigation (e.g., arrow keys, Enter, Space for toggles).
- Visual highlighting uses colors that are distinguishable for users with color vision deficiencies.
- The system provides a way to filter or hide items with zero quantity (e.g., a toggle for "Show Zero Quantity Items").
