# Feature Specification: TUI Enhancements

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Implemented

**Input**: User description:
1. The tree shall be collapsable
2. The tool shall support quantity generation based on length input, weather input, activity input
3. Activity and weather input shall also be connected to the categories of items
4. Use colors in the TUI

> **Alignment note (2026-09-20)**: This spec was reviewed against the implemented
> application and updated to describe the shipped behavior. Input point 3 is
> fulfilled through configurable named multipliers (travel styles, weather,
> activities) that scale item quantities, managed on the configuration screen
> and in the configuration file — not through per-category highlighting.
> The collapsed/expanded state is a per-session view state and is not persisted.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Collapsible Tree Navigation (Priority: P1)

The user launches the application and sees a tree of categories and items. The user can collapse or expand categories to navigate the list more efficiently. The visual state of each category (collapsed/expanded) is clearly indicated, and the cursor moves to the category header when it is collapsed so it stays visible.

**Why this priority**: This improves usability by allowing users to focus on relevant sections of the packing list without visual clutter.

**Independent Test**: This story can be tested independently by verifying that the user can collapse/expand categories and see the visual state update accordingly.

**Acceptance Scenarios**:

1. **Given** the application is launched, **When** the user views the initial screen, **Then** all categories are expanded by default.
2. **Given** the user collapses a category, **When** they view the tree, **Then** the category is collapsed, its items are hidden, and the cursor is on the category header.
3. **Given** the user expands a collapsed category, **When** they view the tree, **Then** the category is expanded and its items are visible.
4. **Given** the user toggles a whole category's selection, **When** the items were added to the trip, **Then** the category collapses; when the items were removed, it expands.

---

### User Story 2 - Quantity Generation Based on Inputs (Priority: P1)

The user inputs their trip details — trip length in weekdays, travel style, weather, and planned activities — on the trip setup screen. The system automatically generates quantities of items based on these inputs and updates the packing list in real-time.

**Why this priority**: This automates the tedious process of calculating how much to pack, reducing user effort and improving accuracy.

**Independent Test**: This story can be tested independently by inputting trip details and verifying that the packing list updates with the correct quantities.

**Acceptance Scenarios**:

1. **Given** the user inputs a length of stay of 7 days, **When** the packing list is generated, **Then** the quantities of items reflect a 7-day trip.
2. **Given** the user selects "cold" weather, **When** the packing list is generated, **Then** the quantities of items increase according to the cold-weather multiplier.
3. **Given** the user selects "hiking" as an activity, **When** the packing list is generated, **Then** the quantities of items increase according to the hiking multiplier.
4. **Given** the user changes the weather input from "hot" to "cold", **When** the packing list is regenerated, **Then** the quantities update to reflect the new multiplier.

---

### User Story 3 - Configurable Modifiers for Styles, Weather, and Activities (Priority: P2)

The user can manage the set of travel styles, weather options, and activities and their quantity multipliers on a dedicated configuration screen, reachable from any screen. The modifiers are stored in the configuration file and can also be edited through the in-app raw config editor. The selected travel style, weather, and activities on the trip setup screen are drawn from this configured set.

**Why this priority**: Making the modifiers configurable lets users tailor the quantity generation to their own packing habits instead of relying on fixed built-in values.

**Independent Test**: This story can be tested independently by adding a new modifier on the configuration screen, selecting it on the trip setup screen, and verifying the quantities change accordingly.

**Acceptance Scenarios**:

1. **Given** the user adds a new travel style with a multiplier on the configuration screen, **When** they cycle travel styles on the trip setup screen, **Then** the new style is available and applies its multiplier.
2. **Given** the user edits a multiplier, **When** the corresponding input is selected, **Then** quantities are recalculated with the new multiplier.
3. **Given** the user deletes a modifier that is currently selected, **When** the deletion is confirmed, **Then** the selection falls back to a valid option and quantities are recalculated.

---

### User Story 4 - Visual Feedback with Colors (Priority: P2)

The system uses colors in the TUI to provide visual feedback: selected versus deselected items, packed versus unpacked state, the focused panel and input fields, and category headers.

**Why this priority**: Colors improve usability by making the interface more intuitive and reducing cognitive load.

**Independent Test**: This story can be tested independently by verifying that the TUI uses colors to provide clear visual feedback.

**Acceptance Scenarios**:

1. **Given** an item is selected for the trip, **When** the user views the tree, **Then** the selection indicator is shown in a distinct color from deselected items.
2. **Given** an item is packed, **When** the user views the list, **Then** the packed indicator is shown in a distinct color from unpacked items.
3. **Given** the user moves focus between panels or fields, **When** they view the screen, **Then** the focused element is visually distinct.

---

### Edge Cases

- What happens if the user inputs conflicting weather and activity (e.g., "hot weather" and "snowboarding activity")?
  **Answer**: When activities are selected, their average multiplier replaces the weather multiplier in the calculation. The behavior is deterministic and documented in the help text; no warning is displayed.
- How does the system handle invalid inputs (e.g., zero weekdays)?
  **Answer**: Trip length is constrained to a valid range (1–30) by the input controls; invalid values cannot be entered.
- What happens if the user collapses a category that contains selected items?
  **Answer**: The collapsed state persists for the session, but the selected items remain part of the packing list.
- How does the system handle very long lists (e.g., 1000+ items)?
  **Answer**: The TUI remains responsive; both panels scroll with the cursor and only render the visible window.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to collapse and expand categories in the tree, with the cursor moving to the category header when collapsing.
- **FR-002**: System MUST support user inputs for trip length (weekdays), travel style, weather, and activities.
- **FR-003**: System MUST generate quantities of items based on the trip length and the configured multipliers for travel style, weather, and activities.
- **FR-004**: System MUST allow users to manage travel styles, weather options, and activities and their multipliers on a configuration screen reachable from any screen, and persist them in the configuration file.
- **FR-005**: System MUST use colors in the TUI to provide visual feedback (selection, packed state, focus, headers).
- **FR-006**: System MUST update the packing list in real-time based on user inputs and tree interactions.
- **FR-007**: System MUST keep the collapsed/expanded state of categories as a session view state; it is not persisted across restarts.

### Key Entities *(include if feature involves data)*

- **Category**: A named group of items (e.g., "Clothing", "Electronics"). Categories can be collapsed or expanded.
- **Item**: A specific packing item (e.g., "T-shirt", "Laptop"). Items can be selected or deselected.
- **Trip parameters**: User-provided data for trip length (weekdays), travel style, weather, and activities.
- **Modifier**: A named multiplier for a travel style, weather condition, or activity, managed by the user and stored in the configuration file.
- **Packing List**: A dynamic list of items, updated in real-time based on user inputs and selections.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can collapse or expand any category in under 2 key presses.
- **SC-002**: System generates quantities of items within 500ms of input submission.
- **SC-003**: 100% of user inputs (length, style, weather, activities) are reflected in the packing list within 1 second.
- **SC-004**: The TUI uses at least 3 distinct colors for visual feedback (selection, packed state, focus).
- **SC-005**: Users can distinguish between collapsed and expanded states of categories at a glance.
- **SC-006**: 90% of users successfully generate a tailored packing list on their first attempt.
- **SC-007**: 100% of user-defined modifiers persist across application restarts.

## Assumptions

- Users have a basic understanding of terminal applications and navigation.
- The application is designed for a single user, as per the project constitution.
- The TUI uses standard keyboard navigation (e.g., arrow keys, Enter, Space for selection).
- Travel styles, weather options, and activities are user-configurable named modifiers with multipliers; built-in defaults are provided out of the box.
- When activities are selected, their average multiplier replaces the weather multiplier in the quantity calculation.
- Colors used in the TUI are accessible and distinguishable for users with color vision deficiencies.
