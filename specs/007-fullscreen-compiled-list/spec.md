# Feature Specification: Fullscreen Compiled List View

**Feature Branch**: `007-fullscreen-compiled-list`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description: "Add a full screen options where only the compiled list is shown"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Toggle a fullscreen view of the compiled list (Priority: P1)

A user has compiled their packing list and wants to focus on it without the distraction of the all-items panel. From the main screen they press a single dedicated key to switch to a fullscreen view that shows only the compiled packing list — the selected items grouped by category with their quantities and packed state. Pressing the same key again returns to the regular two-panel main screen. The trip's data is unaffected by the switch; it is purely a different presentation of the same list.

**Why this priority**: This is the core of the request — a focused, distraction-free view of the finished list. Without the toggle itself, nothing else in this feature exists.

**Independent Test**: From the main screen with items selected, press the fullscreen key; verify only the compiled list is shown, navigation and packing actions work, and toggling back restores the two-panel view with all state intact.

**Acceptance Scenarios**:

1. **Given** the main screen with a compiled list of selected items, **When** the user presses the fullscreen key, **Then** the view switches to a fullscreen display showing only the compiled list.
2. **Given** the fullscreen view, **When** the user presses the same fullscreen key again, **Then** the regular main screen returns with the same selections, packed states, and cursor position.
3. **Given** the fullscreen view, **When** the user exits the application, **Then** the trip is saved exactly as it would be from the main screen.

---

### User Story 2 - Work the list in fullscreen (Priority: P2)

A user in the fullscreen view wants to do the same packing work they can do on the compiled list in the main screen: move the cursor through items, mark items as packed or unpacked, and see their progress. All actions available for the compiled list in the regular view remain available in fullscreen with the same key bindings.

**Why this priority**: A view-only fullscreen would force constant switching back and forth; being able to work directly in fullscreen is what makes the mode useful.

**Independent Test**: In fullscreen, navigate with up/down, toggle packed state on several items, and check the progress indicator updates; verify the same changes are reflected after returning to the main screen.

**Acceptance Scenarios**:

1. **Given** the fullscreen view, **When** the user moves the cursor up and down, **Then** the cursor moves through the compiled list's items and remains visible.
2. **Given** the fullscreen view, **When** the user toggles the packed state of an item, **Then** the item's state changes and any progress display updates.
3. **Given** changes made in fullscreen, **When** the user returns to the main screen, **Then** the same packed states are shown there.

---

### User Story 3 - Fullscreen respects the compiled list's grouping and readability (Priority: P3)

A user opens the fullscreen view and sees the compiled list presented clearly: items grouped by category with visible category headers, quantities shown, and packed/unpacked states distinguishable at a glance. The list scrolls smoothly when it is longer than the screen, and the view follows the cursor.

**Why this priority**: The fullscreen view is only better than the split view if it is at least as readable; grouping and scrolling are what make a long list usable.

**Independent Test**: With a list longer than the terminal height, scroll from top to bottom in fullscreen; verify headers, quantities, and states stay readable and the cursor stays in view.

**Acceptance Scenarios**:

1. **Given** the fullscreen view with multiple categories, **When** the list is displayed, **Then** category headers are visible and items appear under their category.
2. **Given** a compiled list longer than the terminal height, **When** the user moves the cursor past the bottom edge, **Then** the view scrolls and the cursor stays visible.
3. **Given** the fullscreen view, **When** items are packed or unpacked, **Then** their states are visually distinguishable without ambiguity.

---

### User Story 4 - Fullscreen works at small terminal sizes (Priority: P4)

A user opens the fullscreen view in a small terminal (down to the minimum supported size). The view uses the full available area, all content fits, and no function is lost compared to larger sizes.

**Why this priority**: The fullscreen view should benefit from, not fight, the responsive layout work; small-size support is important but secondary to the core toggle and working view.

**Independent Test**: Open the fullscreen view at the minimum supported size; verify the list fills the terminal, remains navigable, and all actions work.

**Acceptance Scenarios**:

1. **Given** the fullscreen view at the minimum supported terminal size, **When** the user navigates and toggles items, **Then** all content fits and every action works as at larger sizes.

### Edge Cases

- What happens when the fullscreen view is toggled while no items are selected (an empty compiled list)?
- What happens when the fullscreen view is opened and then the terminal is resized smaller or larger?
- What happens to the cursor position when toggling into fullscreen while the cursor is on the all-items panel?
- What happens if the user opens help or the config screen from fullscreen — where do they return?
- What happens if the dedicated fullscreen key is pressed while a popup (add/edit/delete) is open?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The user MUST be able to toggle a fullscreen view from the main screen that shows only the compiled packing list, and toggle back to the regular main screen.
- **FR-002**: In the fullscreen view, the compiled list MUST be shown grouped by category with quantities and packed states visible.
- **FR-003**: In the fullscreen view, the user MUST be able to navigate the list and toggle packed states with the same key bindings as the compiled list in the regular view.
- **FR-004**: Toggling the fullscreen view MUST NOT modify any trip data; selections, packed states, and the trip file are unchanged by the view switch itself.
- **FR-005**: The fullscreen view MUST scroll to keep the cursor visible when the list is longer than the screen.
- **FR-006**: The fullscreen view MUST be usable at the minimum supported terminal size, using the full available area.
- **FR-007**: The fullscreen view MUST be toggled on and off by a single dedicated key that does not conflict with existing key bindings; the same key enters and leaves the view.
- **FR-008**: Screens reachable from fullscreen (help, configuration) MUST return to the fullscreen view when closed, preserving the cursor position.

### Key Entities *(include if feature involves data)*

- **Compiled list**: The selected items of the current trip, grouped by category, with quantities and packed states; the same data as the right panel of the main screen.
- **Fullscreen view**: A presentation mode of the main screen showing only the compiled list, toggled on and off without data changes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can switch to the fullscreen view and back in under 2 seconds with a single key press each way, with no data changes.
- **SC-002**: 100% of the actions available for the compiled list in the regular view are available in the fullscreen view.
- **SC-003**: In fullscreen, a list of 100+ items can be navigated end to end with the cursor always visible.
- **SC-004**: At the minimum supported terminal size, the fullscreen view shows the same information and supports the same actions as at full terminal size.
- **SC-005**: Users report the fullscreen view as "less cluttered" than the regular main screen in a review session.

## Assumptions

- The compiled list is the same data as the right panel of the main screen; no separate list is compiled for fullscreen.
- The fullscreen view is a presentation mode of the main screen, not a separate screen with its own state.
- Existing key bindings on the main screen remain unchanged; the fullscreen toggle uses a single dedicated key that is currently unused, and the same key toggles back.
- The minimum supported terminal size follows the project's responsive-UI decision (80x24).
- Exiting the application from fullscreen behaves exactly as exiting from the main screen (same save behavior).
- No new settings or configuration are added for this feature.
