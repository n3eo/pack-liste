# Feature Specification: Packing List I/O

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Implemented

**Input**: User description:
1. Add export ability and import ability of a specific packing list
2. Support naming a packing list

> **Alignment note (2026-09-20)**: This spec was reviewed against the implemented
> application and updated to describe the shipped behavior. "Naming a packing
> list" is fulfilled by the trip file path: each trip is identified by the file
> it is persisted to, and the path is remembered between sessions. There is no
> separate in-app list name or a JSON export/import command; the trip file
> itself is the shareable, hand-editable artifact.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Identify a Trip by Its File (Priority: P1)

The user sets a packing list file path on the trip setup screen (or passes it as a command-line argument when launching the application). The path identifies the trip; it is remembered between sessions, and the compiled packing list is saved to that file on exit. Relative paths resolve against the current working directory; `~` expands to the home directory.

**Why this priority**: A stable identity per trip is what allows continuing, sharing, and backing up a specific packing list.

**Independent Test**: This story can be tested independently by setting a path, exiting, relaunching, and verifying the same trip is offered for resuming.

**Acceptance Scenarios**:

1. **Given** the user sets a packing list file path on the trip setup screen, **When** they exit the application, **Then** the compiled packing list is saved to that path and the path is remembered.
2. **Given** the user launches the application with the file path as the first command-line argument, **When** the application starts, **Then** that trip is loaded directly.
3. **Given** the user launches the application without an argument and a previous trip file exists, **When** the trip setup screen appears, **Then** the user is asked whether to resume the previous trip.

---

### User Story 2 - Continue a Trip from Its File (Priority: P1)

The user can continue working on a saved trip. On resume, the trip's parameters (length, travel style, weather, activities) and each item's packed state are restored. The saved selection is reconciled against the current item catalog: items that no longer exist in the catalog are dropped, and items are matched to their current category.

**Why this priority**: Continuing a previously saved trip is the core of the I/O feature; without it, every session would start from scratch.

**Independent Test**: This story can be tested independently by saving a trip, changing the item catalog, resuming the trip, and verifying the reconciliation.

**Acceptance Scenarios**:

1. **Given** a previously saved trip file, **When** the user resumes it, **Then** the trip parameters and packed states are restored.
2. **Given** the item catalog changed since the trip was saved (items removed or renamed), **When** the trip is resumed, **Then** items no longer in the catalog are dropped from the trip and the rest are matched to their current categories.
3. **Given** the user resumes a trip and exits, **When** they inspect the trip file, **Then** it has been updated in place with the current state.

---

### User Story 3 - Hand-Editable Trip File (Priority: P2)

The trip file is a human-readable file that can be edited with or without the application. It stores the trip parameters and the selected items with their packed state; quantities are not stored — they are recalculated from the trip parameters on load. The file can be copied or shared as the export of a specific packing list, and placed anywhere to be imported (opened) again.

**Why this priority**: A hand-editable, portable file makes sharing and backup trivial and keeps the user in control of their data.

**Independent Test**: This story can be tested independently by editing a trip file by hand, resuming it, and verifying the changes are reflected; and by copying a trip file and opening it as a new trip.

**Acceptance Scenarios**:

1. **Given** the user edits a trip file by hand (for example toggling packed states), **When** they resume the trip, **Then** the edited state is loaded.
2. **Given** the user copies a trip file to another location or machine, **When** it is opened with the application, **Then** the same trip is available there.
3. **Given** the user inspects a saved trip file, **When** they read it, **Then** it contains the trip parameters and the selected items with packed states, and no stored quantities.

---

### Edge Cases

- What happens if the user resumes a trip whose file contains items unknown to the catalog?
  **Answer**: Unknown items are dropped during reconciliation; the rest of the trip loads normally.
- What happens if the trip file path points to a nonexistent file?
  **Answer**: The trip is treated as new: all items start selected, and the file is created on exit.
- What happens if the user declines resuming the previous trip?
  **Answer**: A fresh list starts with all items selected; the previous trip file is left untouched, and its path remains remembered for the next session.
- What happens if the trip file cannot be parsed?
  **Answer**: The application reports the error and does not silently discard the trip.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow the user to set a packing list file path that identifies the trip, via the trip setup screen or as the first command-line argument.
- **FR-002**: System MUST save the compiled packing list (trip parameters and selected items with packed state) to the trip file on exit, in place.
- **FR-003**: System MUST remember the last used trip file path between sessions and offer to resume it on start.
- **FR-004**: System MUST reconcile a resumed trip against the current item catalog (drop unknown items, match current categories) by case-insensitive item name.
- **FR-005**: System MUST store the trip file in a human-readable, hand-editable format that can be edited, copied, and shared without the application.
- **FR-006**: System MUST NOT persist calculated quantities in the trip file; quantities are recalculated from the trip parameters on load.
- **FR-007**: System MUST resolve relative trip file paths against the current working directory and expand `~` to the home directory.

### Key Entities *(include if feature involves data)*

- **Trip file**: The persisted, human-readable artifact of one trip: trip parameters plus selected items with packed state. Updated in place on exit.
- **Item catalog** (configuration file): The user's master list of items and categories, separate from any trip. Trips are reconciled against it on load.
- **Trip file path**: The identity of a trip; remembered between sessions, settable via UI or CLI.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can set a trip file path in under 10 seconds.
- **SC-002**: 100% of trips are saved on exit to their file, in place, without user interaction.
- **SC-003**: A saved trip resumes with identical trip parameters and packed states.
- **SC-004**: A trip file edited by hand loads with the edited state.
- **SC-005**: A trip file copied elsewhere opens as the same trip (portable export/import).
- **SC-006**: The trip file contains no stored quantities; all quantities are recalculated on load.

## Assumptions

- Users have a basic understanding of file systems and can locate and edit trip files.
- The application is designed for a single user, as per the project constitution.
- The trip file uses a human-readable format (YAML) with UTF-8 encoding.
- The item catalog (configuration file) and trip files are separate: the catalog is the input, trip files are compiled outputs.
- One trip maps to one file; managing multiple trips means managing multiple files.
