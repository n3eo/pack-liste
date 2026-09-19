# Feature Specification: Quantity Calculation

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description:
Update the quantity calculation implementation to consider the input week days, and weigh based on travel style, weather input, activities, ...

## User Scenarios & Testing *(mandatory)*

### User Story 5 - Packing List Management (Priority: P2)

The user can manage packing lists, including creating, selecting, renaming, exporting, and importing packing lists. The system ensures that all packing lists are stored persistently in `~/.config/pack-liste/` and can be retrieved for future trips.

**Why this priority**: Packing list management is essential for users who want to save and reuse their packing configurations across multiple trips.

**Independent Test**: This story can be tested independently by creating, renaming, exporting, and importing packing lists.

**Acceptance Scenarios**:

1. **Given** the user creates a new packing list, **When** the system processes the request, **Then** a new empty packing list is created with a default name (e.g., "Unnamed Packing List - <timestamp>").
2. **Given** the user renames a packing list to "Summer Trip 2026", **When** the system processes the request, **Then** the packing list name is updated to "Summer Trip 2026" and persists across sessions.
3. **Given** the user exports the packing list to a file named "summer_trip.json", **When** the system processes the request, **Then** the packing list is saved to `~/.config/pack-liste/summer_trip.json` with all selected items, their quantities, and the packing list name.
4. **Given** the user imports a packing list from "summer_trip.json", **When** the system processes the request, **Then** the packing list is loaded from the file and displayed in the UI with all items and their quantities.
5. **Given** the user selects an existing packing list named "Winter Gear", **When** the system processes the request, **Then** the selected packing list is loaded and displayed in the UI with all items and their quantities.
6. **Given** the user tries to import a packing list with an invalid name (e.g., empty or >50 characters), **When** the system processes the request, **Then** an error message is displayed and the import is aborted.

---

### User Story 1 - Calculate Quantities Based on Weekdays (Priority: P1)

The user inputs the number of weekdays for their trip (e.g., 5 weekdays). The system updates the quantities of items based on this input, ensuring that users pack appropriately for the duration of their trip.

**Why this priority**: Accurate quantity calculations based on trip duration are essential for ensuring users bring enough items without overpacking.

**Independent Test**: This story can be tested independently by inputting weekdays and verifying that the quantities update correctly.

**Acceptance Scenarios**:

1. **Given** the user inputs 3 weekdays, **When** the packing list is generated, **Then** the quantities of items (e.g., "T-shirts") are adjusted to reflect a 3-day trip.
2. **Given** the user inputs 10 weekdays, **When** the packing list is generated, **Then** the quantities of items are adjusted to reflect a 10-day trip.
3. **Given** the user changes the number of weekdays from 5 to 7, **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect a 7-day trip.
4. **Given** the user deselects a category or item, **When** the packing list is generated, **Then** the deselected category/item is hidden from the packing list and marked as excluded. Users can reselect it to restore it to the packing list.
5. **Given** the user exports the packing list, **When** the system processes the export, **Then** the exported JSON file contains all selected items with their calculated quantities and the packing list name.

---

### User Story 2 - Weigh Quantities Based on Travel Style (Priority: P1)

The user selects a travel style (e.g., "minimalist", "luxury", "backpacker"). The system adjusts the quantities of items based on the selected travel style, ensuring that users pack appropriately for their preferred style.

**Why this priority**: Different travel styles require different quantities of items (e.g., a minimalist traveler may pack fewer clothes than a luxury traveler).

**Independent Test**: This story can be tested independently by selecting a travel style and verifying that the quantities update correctly.

**Acceptance Scenarios**:

1. **Given** the user selects "minimalist" travel style, **When** the packing list is generated, **Then** the quantities of items are reduced to reflect minimalist packing.
2. **Given** the user selects "luxury" travel style, **When** the packing list is generated, **Then** the quantities of items are increased to reflect luxury packing.
3. **Given** the user changes the travel style from "minimalist" to "backpacker", **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect backpacker packing.

---

### User Story 3 - Weigh Quantities Based on Weather Input (Priority: P1)

The user inputs weather conditions for their trip (e.g., "hot", "cold", "rainy"). The system adjusts the quantities of items based on the weather input, ensuring that users pack appropriately for the expected conditions.

**Why this priority**: Weather conditions significantly impact what items users need to pack (e.g., more warm clothing for cold weather).

**Independent Test**: This story can be tested independently by inputting weather conditions and verifying that the quantities update correctly.

**Acceptance Scenarios**:

1. **Given** the user inputs "hot weather", **When** the packing list is generated, **Then** the quantities of summer clothing items increase.
2. **Given** the user inputs "cold weather", **When** the packing list is generated, **Then** the quantities of winter clothing items increase.
3. **Given** the user changes the weather input from "hot" to "cold", **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect cold weather.

---

### User Story 4 - Weigh Quantities Based on Activities (Priority: P1)

The user inputs activities for their trip (e.g., "hiking", "beach", "city tour"). The system adjusts the quantities of items based on the activities, ensuring that users pack appropriately for their planned activities.

**Why this priority**: Activities determine what items users need to pack (e.g., hiking gear for a hiking trip).

**Independent Test**: This story can be tested independently by inputting activities and verifying that the quantities update correctly.

**Acceptance Scenarios**:

1. **Given** the user inputs "hiking" as an activity, **When** the packing list is generated, **Then** the quantities of hiking gear increase.
2. **Given** the user inputs "beach" as an activity, **When** the packing list is generated, **Then** the quantities of beachwear items increase.
3. **Given** the user changes the activity from "hiking" to "beach", **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect the new activity.
4. **Given** the user exports the packing list with activities, **When** the system processes the export, **Then** the exported JSON file contains the selected activities and all items with their calculated quantities.

---

### Edge Cases

- **EC-001**: What happens if the user inputs conflicting data (e.g., "hot weather" and "snowboarding activity")?
  **Answer**: The system prioritizes the activity input and adjusts quantities accordingly, with a warning message displayed to the user.
  **Acceptance Criteria**: System must display a warning message and update quantities based on the prioritized input.

- **EC-002**: How does the system handle invalid inputs (e.g., negative weekdays)?
  **Answer**: The system displays an error message and does not update the packing list until valid inputs are provided.
  **Acceptance Criteria**: System must validate inputs and show clear error messages for invalid values.

- **EC-003**: What happens if the user inputs a very large number of weekdays (e.g., 100)?
  **Answer**: The system calculates quantities proportionally but caps the maximum quantity for any item to a reasonable limit (e.g., 20).
  **Acceptance Criteria**: System must cap quantities at 20 and handle large inputs gracefully.

- **EC-004**: How does the system handle overlapping inputs (e.g., "hot weather" and "cold weather")?
  **Answer**: The system prioritizes the most recent input and displays a warning message.
  **Acceptance Criteria**: System must prioritize the most recent input and show a warning message.

- **EC-005**: What happens when a user tries to export a packing list with no name?
  **Answer**: The system automatically assigns a default name (e.g., "Unnamed Packing List - <timestamp>") and prompts the user to rename it.
  **Acceptance Criteria**: System must handle unnamed lists gracefully and provide a way to rename them.

- **EC-006**: What happens when a user tries to import a corrupted JSON file?
  **Answer**: The system displays an error message and does not load the file.
  **Acceptance Criteria**: System must validate imported files and show clear error messages for corruption.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST update the quantity calculation implementation to consider input weekdays.
- **FR-002**: System MUST weigh quantity calculations based on travel style.
- **FR-003**: System MUST weigh quantity calculations based on weather input.
- **FR-004**: System MUST weigh quantity calculations based on activities.
- **FR-005**: System MUST update the packing list with calculated quantities in real-time.
- **FR-006**: System MUST handle conflicting inputs gracefully (e.g., prioritize activities over weather).
- **FR-007**: System MUST validate user inputs (e.g., non-negative weekdays).
- **FR-008**: System MUST support packing list export to JSON files.
- **FR-009**: System MUST support packing list import from JSON files.
- **FR-010**: System MUST allow packing list renaming.
- **FR-011**: System MUST persist packing lists in `~/.config/pack-liste/` directory.
- **FR-012**: System MUST support editable item quantities and packed status toggles.

### Key Entities *(include if feature involves data)*

- **Weekdays**: Days of the week for the trip.
- **Travel Style**: User-defined travel style (e.g., "minimalist", "luxury").
- **Weather Input**: User-provided weather conditions (e.g., "hot", "cold").
- **Activities**: User-defined activities (e.g., "hiking", "beach").
- **Quantity Calculation**: Algorithm to determine item quantities based on inputs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Quantity calculations consider weekdays, travel style, weather, and activities.
- **SC-002**: Packing lists can be exported to and imported from JSON files with 100% data fidelity.
- **SC-003**: All packing lists must have a unique name (max 50 characters) and support renaming.
- **SC-004**: Users can see updated quantities within 1 second of input changes.
- **SC-005**: 100% of calculated quantities are accurate based on the provided inputs.
- **SC-006**: System handles conflicting inputs (e.g., "hot weather" and "snowboarding activity") gracefully.
- **SC-007**: Users receive feedback when inputs are invalid (e.g., negative weekdays).

## Assumptions

- Users have a basic understanding of the inputs required for quantity calculations (weekdays, travel style, weather, activities).
- The application is designed for a single user, as per the project constitution.
- The system provides immediate feedback when inputs are changed.
- The quantity calculation algorithm is deterministic and produces consistent results for the same inputs.
- The system prioritizes activities over weather when both are provided (e.g., "snowboarding" overrides "hot weather").
- Packing lists are stored in `~/.config/pack-liste/` directory.
- Export/import functionality uses JSON format with UTF-8 encoding.

## Clarifications

### Session 2026-09-19

- Q: How does the subtractive selection process work in the packing list feature? Specifically, what happens when a user deselects a category or item? (FR-005) → A: The system hides deselected categories/items from the packing list and marks them as excluded. Users can reselect it to restore it to the packing list.
