# Feature Specification: Persist Quantities in the Trip File

**Feature Branch**: `012-trip-quantities-persisted`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"save quantities to output yaml"

> **Scope note**: The trip file (the output YAML of a trip) currently
> stores trip parameters and each selected item's name, category, and
> packed state — but deliberately not the quantities, which are
> recalculated on load (spec 004, FR-006). This feature reverses that
> decision for the output file: the compiled quantities are saved with
> each item so the file is a complete, self-contained snapshot of the
> trip. The input configuration file (config.yaml) is not affected; it
> keeps storing only fixed quantities, as before.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quantities are saved to the trip file (Priority: P1)

The user compiles a packing list (quantities calculated from trip parameters and per-item configuration), works through it, and exits. The saved trip file now contains each selected item's quantity alongside its name, category, and packed state. The file is a complete snapshot: someone reading it — with or without the application — sees exactly how many of each item the trip calls for.

**Why this priority**: This is the entire request. Without stored quantities the output file is incomplete as a shareable artifact: a printed or shared list shows items but not how many, and any consumer must re-run the calculation to recover the numbers.

**Independent Test**: Compile a list with known quantities, exit, and inspect the trip file; verify each item carries its quantity. Open the file in a text editor and confirm the quantities are readable.

**Acceptance Scenarios**:

1. **Given** a compiled packing list with calculated quantities, **When** the trip is saved, **Then** every selected item in the trip file carries its quantity.
2. **Given** a saved trip file, **When** the user reads it (in a text editor or another tool), **Then** the quantity of each item is visible without re-running any calculation.
3. **Given** an item with a fixed quantity, **When** the trip is saved, **Then** its fixed quantity is stored like any other item's.

---

### User Story 2 - A resumed trip restores its saved quantities (Priority: P1)

The user resumes a saved trip. The quantities shown are the ones saved in the trip file — the trip continues exactly where it left off, even if the calculation inputs (modifiers, per-item configuration) have changed since the save. Recalculation happens only when the user actually changes trip parameters, not on load.

**Why this priority**: Storing quantities is only useful if they are honored on load. Restoring them makes the file authoritative for its own trip and protects a compiled list from later configuration changes (e.g., an edited multiplier silently reshuffling a half-packed trip).

**Independent Test**: Save a trip, change a quantity-relevant configuration value, resume the trip; verify the quantities are the saved ones, not recalculated.

**Acceptance Scenarios**:

1. **Given** a saved trip file with stored quantities, **When** the trip is resumed, **Then** each item shows the quantity stored in the file.
2. **Given** a resumed trip, **When** the user changes a trip parameter (e.g., weekdays), **Then** quantities recalculate from the new parameters, overwriting the restored values.
3. **Given** a trip file saved by an older version (no quantities stored), **When** it is resumed, **Then** quantities are calculated as before (backward-compatible load).

---

### User Story 3 - Hand edits to quantities are honored (Priority: P2)

The user edits the trip file by hand — for example raising an item's quantity — and resumes the trip. The edited quantity loads and is shown; on the next save the file contains the edited value (unless parameters changed in between, which recalculates as usual).

**Why this priority**: The trip file is specified as hand-editable (spec 004, US-3); storing quantities extends that promise to the numbers themselves, keeping the file fully usable without the application.

**Independent Test**: Edit a quantity in a saved trip file with a text editor, resume, and verify the edited quantity appears in the list and survives the next save.

**Acceptance Scenarios**:

1. **Given** a hand-edited quantity in the trip file, **When** the trip is resumed, **Then** the edited quantity is loaded and displayed.
2. **Given** a resumed trip with a hand-edited quantity, **When** the trip is saved again (without parameter changes), **Then** the edited quantity is preserved in the file.

---

### User Story 4 - The input configuration file is unchanged (Priority: P2)

The user's item catalog (the input configuration file) keeps its current behavior: it stores fixed quantities for fixed items and nothing for automatic items. Only the output trip file gains stored quantities. The separation between input catalog and compiled trip is preserved.

**Why this priority**: The input/compiled separation is an existing design principle (spec 004); persisting calculated quantities into the catalog would couple every trip's numbers to the shared catalog and break that separation.

**Independent Test**: Compile and save a trip, then inspect the input configuration file; verify it contains no calculated quantities.

**Acceptance Scenarios**:

1. **Given** a compiled trip with calculated quantities, **When** the trip is saved, **Then** the input configuration file is unchanged in its quantity behavior (fixed quantities only).
2. **Given** the input configuration file, **When** inspected, **Then** it contains no trip-specific calculated quantities.

### Edge Cases

- **EC-001**: What happens when a trip file without quantities (older version) is loaded?
  **Answer**: The quantity field is optional on load; missing quantities are calculated from the trip parameters as before. No error, no data loss.
  **Acceptance Criteria**: A pre-feature trip file loads and calculates exactly as before this feature.

- **EC-002**: What happens when the catalog changed between save and resume (items removed or renamed)?
  **Answer**: The existing reconciliation (spec 004, FR-004) applies unchanged: unknown items are dropped, categories are matched. Stored quantities survive for items that reconcile.
  **Acceptance Criteria**: A resumed trip with a changed catalog keeps stored quantities for all surviving items.

- **EC-003**: What happens if a stored quantity is invalid (zero, negative, non-numeric)?
  **Answer**: Invalid stored quantities are ignored on load and the quantity is recalculated; the load never fails because of a bad quantity.
  **Acceptance Criteria**: A trip file with an invalid quantity loads successfully with that item's quantity recalculated.

- **EC-004**: Does a stored quantity survive a parameter change round trip?
  **Answer**: No — changing trip parameters recalculates (existing behavior), and the next save stores the recalculated values. Stored quantities are a snapshot, not an override of the calculation.
  **Acceptance Criteria**: After a parameter change and save, the file contains the recalculated quantities.

- **EC-005**: Are quantities stored for unselected (deselected) items?
  **Answer**: No — the trip file stores only the compiled list (selected items), as before; quantities are stored for exactly the items already in the file.
  **Acceptance Criteria**: The set of items in the trip file is unchanged by this feature; only their entries gain a quantity.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST save each selected item's quantity to the trip file when the trip is saved.
- **FR-002**: The system MUST restore the stored quantities when a trip is resumed, showing them without recalculating on load.
- **FR-003**: The system MUST recalculate quantities only in response to user changes (trip parameters, per-item configuration), never merely on load.
- **FR-004**: The stored quantity MUST be optional on load: trip files without quantities (older versions) load and calculate as before.
- **FR-005**: The system MUST ignore invalid stored quantities on load and recalculate those items, without failing the load.
- **FR-006**: The input configuration file MUST keep its existing quantity behavior (fixed quantities only; no calculated quantities stored).
- **FR-007**: The trip file MUST remain human-readable and hand-editable, with quantities readable and editable like the other fields.
- **FR-008**: The existing reconciliation on resume (drop unknown items, match current categories) MUST be unaffected by stored quantities.

### Key Entities *(include if feature involves data)*

- **Trip file** (output YAML): The persisted snapshot of one trip; now stores per-item quantities in addition to trip parameters, names, categories, and packed states.
- **Stored quantity**: The item's compiled quantity at save time; authoritative on resume until a user change triggers recalculation; optional on load for backward compatibility.
- **Input configuration file** (config.yaml): The item catalog; unchanged by this feature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of saved trips contain a quantity for every selected item.
- **SC-002**: 100% of resumed trips show the stored quantities without recalculation on load.
- **SC-003**: 100% of pre-feature trip files (no quantities) load and calculate identically to before.
- **SC-004**: A quantity edited by hand in the trip file is visible in the application after resume.
- **SC-005**: The input configuration file's contents are unchanged by this feature (no calculated quantities stored there).

## Assumptions

- Stored quantities are a snapshot of the compiled list at save time; they do not override the calculation once the user changes any relevant input (existing recalculation triggers).
- The quantity field is optional in the trip file format so older files remain loadable; the format change is additive.
- The trip file remains the single output artifact; no separate export is added.
- Recalculation triggers are exactly the existing ones (trip parameter changes, per-item configuration changes); loading a trip is not a trigger.
- The input configuration file keeps storing fixed quantities only, preserving the input/compiled separation.
