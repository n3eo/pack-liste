# Feature Specification: Packing List TUI

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Implemented

**Input**: User description:
1. The user shall be able to generate a travel packing list
2. Items are grouped into categories
3. Items can be in multiple categories
4. The user-selected categories and individual items result in the packing list
5. The packing list is auto-updating and displayed on the right side
6. The left side shows a tree with the categories and items the user can select and deselect
7. At the start of a new list, all items are selected, and the user must use a subtractive process and deselect groups/items
8. The categories and items are stored in the ".config" directory of this app and can be extended by the user through the TUI
9. Support import and export of the categories and item list

> **Alignment note (2026-09-20)**: This spec was reviewed against the implemented
> application and updated to describe the shipped behavior. Input point 9
> (import/export of the category/item catalog) is fulfilled by the editable,
> human-readable configuration file rather than a dedicated import/export
> command; see User Story 3.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate and Customize Packing List (Priority: P1)

The user launches the application and sees a tree of categories and items on the left side, with all items selected by default on a new or empty list. The user deselects unwanted categories or individual items, and the packing list on the right side updates in real-time to reflect the changes. The user can save the final packing list for their trip.

**Why this priority**: This is the core functionality of the application, enabling users to create a tailored packing list efficiently.

**Independent Test**: This story can be tested independently by verifying that the user can launch the app, deselect items, and see the packing list update accordingly. It delivers the primary value of the feature.

**Acceptance Scenarios**:

1. **Given** the application is launched with a new or empty list, **When** the user views the initial screen, **Then** all categories and items are selected by default.
2. **Given** the user deselects a category, **When** the packing list is viewed, **Then** all items in that category are removed from the packing list.
3. **Given** the user deselects an individual item, **When** the packing list is viewed, **Then** that item is removed from the packing list.
4. **Given** the user reselects a deselected category or item, **When** the packing list is viewed, **Then** the items are added back to the packing list.

---

### User Story 2 - Extend Categories and Items (Priority: P2)

The user can add new items to the existing list through the TUI (choosing from existing categories or entering a new one). These additions are saved to the application's configuration directory and persist across application restarts.

**Why this priority**: This allows users to customize the application to their specific needs, increasing its long-term utility.

**Independent Test**: This story can be tested independently by adding a new item, restarting the app, and verifying the addition persists.

**Acceptance Scenarios**:

1. **Given** the user is on the items tree screen, **When** they add a new item with an existing category, **Then** the item appears under that category and is selected by default.
2. **Given** the user is on the items tree screen, **When** they add a new item with a new category name, **Then** the item appears under the new category and is selected by default.
3. **Given** the user restarts the application, **When** they view the categories/items tree, **Then** the newly added items are present.

---

### User Story 3 - Edit the Item Catalog via the Configuration File (Priority: P3)

The user can extend or restructure their item catalog outside the TUI by editing the application's configuration file, which stores the categories and items in a human-readable format. The file can be edited by hand or through the in-app raw config editor; changes are applied without restarting the application.

**Why this priority**: This enables users to bulk-edit their catalog (for example after converting an existing packing list from a spreadsheet) and to back up or share the catalog by copying the file.

**Independent Test**: This story can be tested independently by editing the configuration file (by hand or through the in-app editor), applying the changes, and verifying the tree reflects them.

**Acceptance Scenarios**:

1. **Given** the user edits the configuration file to add items or categories, **When** the changes are applied (in-app editor save, or next application start), **Then** the categories and items from the file appear in the tree.
2. **Given** the user copies the configuration file to another machine, **When** the application starts there, **Then** the same categories and items are available.
3. **Given** the user edits the configuration file with invalid content, **When** the change is applied through the in-app editor, **Then** the application shows an error and keeps the existing catalog unchanged.

---

### Edge Cases

- What happens when a category is empty (no items)?
  **Answer**: The category is not shown; categories are derived from the items that exist.
- How does the system handle duplicate item names across categories?
  **Answer**: Duplicate item names are allowed, as items can belong to multiple categories. Trip files reconcile items by name (case-insensitive), matching the first occurrence.
- What happens if the configuration directory is not writable?
  **Answer**: The application reports the error when saving and continues with the in-memory list for the session.
- How does the system handle very large lists (e.g., 1000+ items)?
  **Answer**: The TUI remains responsive; both panels scroll with the cursor and only render the visible window.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to generate a travel packing list based on selected categories and items.
- **FR-002**: System MUST group items into categories.
- **FR-003**: System MUST allow items to belong to multiple categories.
- **FR-004**: System MUST display a tree of categories and items on the left side of the TUI for selection and deselection.
- **FR-005**: System MUST auto-update and display the packing list on the right side of the TUI based on the selected categories and items.
- **FR-006**: System MUST start with all items selected by default on a new or empty list, requiring the user to deselect unwanted items or categories.
- **FR-007**: System MUST store categories and items in the application's configuration directory (`.config`), in a human-readable, hand-editable format.
- **FR-008**: System MUST allow users to add, edit, or delete items through the TUI.
- **FR-009**: System MUST allow the item catalog to be extended or restructured by editing the configuration file, by hand or through the in-app raw config editor, and apply the changes without restart.

### Key Entities *(include if feature involves data)*

- **Category**: A named group of items (e.g., "Clothing", "Electronics"). Categories are derived from the items that exist; each category can contain multiple items, and items can belong to multiple categories.
- **Item**: A specific packing item (e.g., "T-shirt", "Laptop"). Items can be selected or deselected independently or as part of a category.
- **Packing List**: A dynamic list of selected items, updated in real-time as the user selects or deselects categories and items.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can generate a packing list in under 1 minute from application launch.
- **SC-002**: Users can select or deselect any category or item with no more than 2 key presses.
- **SC-003**: The packing list updates in real-time, with changes reflected in under 100ms.
- **SC-004**: 100% of user-added items persist across application restarts.
- **SC-005**: Catalog edits made through the configuration file are reflected in the application without restart.
- **SC-006**: Users can add a new item in under 30 seconds.

## Assumptions

- Users have a basic understanding of terminal applications and navigation.
- The `.config` directory is writable by the application.
- The configuration file uses a human-readable format (YAML) so it can be edited by hand or shared by copying.
- The application is designed for a single user, as per the project constitution.
- The TUI uses standard keyboard navigation (e.g., arrow keys, Enter, Space for selection).
- Categories and items are stored in a structured format to allow for easy editing and persistence.
