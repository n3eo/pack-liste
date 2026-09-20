# Feature Specification: Quantity Calculation

**Feature Branch**: `N/A`

**Created**: 2026-09-19

**Status**: Implemented

**Input**: User description:
Update the quantity calculation implementation to consider the input week days, and weigh based on travel style, weather input, activities, ...

> **Alignment note (2026-09-20)**: This spec was reviewed against the implemented
> application and updated to describe the shipped behavior. The original spec
> bundled packing-list management (naming, JSON export/import, storage under
> `~/.config/pack-liste/`); those concerns moved to spec 004 (trip files,
> path-identified, human-readable YAML). This spec now covers only the quantity
> calculation itself. The calculation is deterministic; conflicting inputs
> (activities vs. weather) are resolved by rule, without a user-facing warning.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Calculate Quantities Based on Weekdays (Priority: P1)

The user inputs the number of weekdays for their trip (e.g., 5 weekdays). The system updates the quantities of items based on this input, ensuring that users pack appropriately for the duration of their trip.

**Why this priority**: Accurate quantity calculations based on trip duration are essential for ensuring users bring enough items without overpacking.

**Independent Test**: This story can be tested independently by inputting weekdays and verifying that the quantities update correctly.

**Acceptance Scenarios**:

1. **Given** the user inputs 3 weekdays, **When** the packing list is generated, **Then** the quantities of items are adjusted to reflect a 3-day trip.
2. **Given** the user inputs 10 weekdays, **When** the packing list is generated, **Then** the quantities of items are adjusted to reflect a 10-day trip.
3. **Given** the user changes the number of weekdays from 5 to 7, **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect a 7-day trip.

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

1. **Given** the user inputs "hot" weather, **When** the packing list is generated, **Then** the quantities of items are adjusted by the hot-weather multiplier.
2. **Given** the user inputs "cold" weather, **When** the packing list is generated, **Then** the quantities of items are adjusted by the cold-weather multiplier.
3. **Given** the user changes the weather input from "hot" to "cold", **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect cold weather.

---

### User Story 4 - Weigh Quantities Based on Activities (Priority: P1)

The user inputs activities for their trip (e.g., "hiking", "beach", "city tour"). The system adjusts the quantities of items based on the activities, ensuring that users pack appropriately for their planned activities.

**Why this priority**: Activities determine what items users need to pack (e.g., hiking gear for a hiking trip).

**Independent Test**: This story can be tested independently by inputting activities and verifying that the quantities update correctly.

**Acceptance Scenarios**:

1. **Given** the user inputs "hiking" as an activity, **When** the packing list is generated, **Then** the quantities of items are adjusted by the hiking multiplier.
2. **Given** the user inputs multiple activities, **When** the packing list is generated, **Then** the quantities of items are adjusted by the average of the activities' multipliers.
3. **Given** the user changes the selected activities, **When** the packing list is regenerated, **Then** the quantities of items are updated to reflect the new set.

---

### Edge Cases

- **EC-001**: What happens if the user inputs conflicting data (e.g., "hot weather" and "snowboarding activity")?
  **Answer**: When activities are selected, the average of their multipliers replaces the weather multiplier in the calculation. The rule is deterministic; no warning is displayed.
  **Acceptance Criteria**: With activities selected, the weather multiplier has no effect on the result.

- **EC-002**: How does the system handle invalid inputs (e.g., zero or negative weekdays)?
  **Answer**: Trip length is a validated value constrained to 1–30; values outside the range are rejected by the input control.
  **Acceptance Criteria**: The trip length input cannot produce a value outside 1–30.

- **EC-003**: What happens if the user inputs a very large number of weekdays (e.g., 100)?
  **Answer**: The input is capped at 30, and the calculation base is additionally capped at 20 days; multipliers still apply on top of the capped base.
  **Acceptance Criteria**: For any valid input, the base never exceeds 20; the final quantity is the capped base times the combined multipliers, with a minimum of 1.

- **EC-004**: How does the system handle an unknown modifier name (e.g., a travel style missing from the configuration)?
  **Answer**: Unknown names fall back to a neutral multiplier of 1.0 so the calculation never fails.
  **Acceptance Criteria**: An unknown style, weather, or activity never blocks the calculation.

  > **Update (2026-09-20, spec 015)**: FR-010 is now uniformly enforced:
  > unknown activities resolve to 1.0 in the activity average (previously
  > they were silently dropped, which could hand control back to the
  > weather multiplier). All three modifier kinds now share the exact
  > fallback behavior this requirement specifies.

- **EC-005**: What happens to fixed-quantity items when trip parameters change?
  **Answer**: Fixed-quantity items keep their quantity regardless of trip parameters; only automatic items are recalculated.
  **Acceptance Criteria**: A fixed item's quantity is identical before and after any parameter change.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST update the quantity calculation implementation to consider input weekdays.
- **FR-002**: System MUST weigh quantity calculations based on travel style.
- **FR-003**: System MUST weigh quantity calculations based on weather input.
- **FR-004**: System MUST weigh quantity calculations based on activities.
- **FR-005**: System MUST update the packing list with calculated quantities in real-time.
- **FR-006**: System MUST resolve conflicting inputs deterministically: when activities are selected, their average multiplier replaces the weather multiplier.
- **FR-007**: System MUST validate trip length inputs (1–30) and cap the calculation base at 20 days.
- **FR-008**: System MUST clamp every calculated quantity to a minimum of 1.
- **FR-009**: System MUST keep fixed-quantity items outside the automatic calculation.
- **FR-010**: System MUST fall back to a neutral multiplier (1.0) for unknown modifier names.

### Key Entities *(include if feature involves data)*

- **Weekdays**: The trip length in days; validated to 1–30, default 7.
- **Travel Style**: A named modifier (e.g., "minimalist", "luxury") with a quantity multiplier.
- **Weather**: A named modifier (e.g., "hot", "cold") with a quantity multiplier.
- **Activities**: Named modifiers (e.g., "hiking", "beach"); multiple may be selected; their average multiplier applies.
- **Quantity Calculation**: The deterministic rule: base (weekdays, capped at 20) × travel-style multiplier × effective weather multiplier (weather, or the average of selected activities), rounded, minimum 1.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Quantity calculations consider weekdays, travel style, weather, and activities.
- **SC-002**: Users see updated quantities within 1 second of input changes.
- **SC-003**: 100% of calculated quantities are accurate based on the provided inputs (deterministic, unit-tested rule).
- **SC-004**: The system resolves conflicting inputs (activities vs. weather) deterministically by rule.
- **SC-005**: Invalid inputs (trip length outside 1–30) are rejected by the input controls.
- **SC-006**: Every calculated quantity is at least 1, for any valid input.

## Assumptions

- Users have a basic understanding of the inputs required for quantity calculations (weekdays, travel style, weather, activities).
- The application is designed for a single user, as per the project constitution.
- The system provides immediate feedback when inputs are changed.
- The quantity calculation algorithm is deterministic and produces consistent results for the same inputs.
- When activities are selected, their average multiplier replaces the weather multiplier.
- Modifier names and multipliers are user-configurable (see spec 002); unknown names fall back to 1.0.

> **Alignment note (2026-09-20, spec 017)**: The activity dimension is
> removed by spec 017 (remove-activities). The calculation is the
> pre-activity rule: base (weekdays, capped at 20) × travel-style
> multiplier × weather multiplier, rounded, minimum 1. The
> activity-priority rule ("activities replace weather") and the
> unknown-activity fallback are gone with the dimension; unknown
> travel-style and weather names still fall back to 1.0.
