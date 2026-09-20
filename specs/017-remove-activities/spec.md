# Feature Specification: Remove Activity Options

**Feature Branch**: `017-remove-activities`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"remove the activity options"

> **Scope note**: Activities are one of the three quantity-modifier
> kinds (travel styles, weather, activities). They appear in the
> welcome screen's input fields, the configuration screen's third
> section, the quantity calculation (where selected activities'
> average multiplier replaces the weather multiplier), the
> configuration file, and the trip file. This feature removes the
> activity dimension entirely: no activity input, no activity
> configuration, no activity multipliers, no activity persistence.
> The calculation falls back to weather alone (the pre-activity rule).

## What is removed (verified in code)

| Surface | Today | After |
|---------|-------|-------|
| Welcome screen | Activities field with cursor + Space toggle | Field gone; focus cycle is file → weekdays → style → weather |
| Config screen | Third section (Tab cycles styles → weather → activities) | Two sections: styles → weather |
| Calculation | Activities' average replaces weather multiplier | Weather multiplier applies directly (spec 005's pre-activity rule) |
| Config file | `selected_activities`, `modifiers.activities` | Not written; ignored on load (harmless leftover) |
| Trip file | `selected_activities` | Not written; ignored on load |
| Help screen | Activities entries | Removed |

## User Scenarios & Testing *(mandatory)*

### User Story 1 - No activity input anywhere (Priority: P1)

The user plans a trip without ever being asked about activities. The welcome screen shows the remaining inputs — trip file, weekdays, travel style, weather — and nothing else. The configuration screen manages travel styles and weather only. No screen, hint, or help text mentions activities.

**Why this priority**: This is the entire request. The activity dimension is removed from the product surface completely.

**Independent Test**: Visit every screen; verify no activity input, list, hint, or help entry exists anywhere.

**Acceptance Scenarios**:

1. **Given** the welcome screen, **When** the user cycles the fields, **Then** the cycle is file → weekdays → travel style → weather (no activities field).
2. **Given** the configuration screen, **When** the user switches sections, **Then** only travel styles and weather exist.
3. **Given** the help screen, **When** read, **Then** no activity-related binding or description appears.

---

### User Story 2 - The calculation uses weather directly (Priority: P1)

Quantities are calculated from weekdays, travel style, and weather — the weather multiplier applies directly, with no activity path. This is spec 005's original rule before activities were added: base (weekdays, capped) × travel-style multiplier × weather multiplier, rounded, minimum 1. The activity-priority rule (activities replace weather) is removed along with its input.

**Why this priority**: The calculation must stay deterministic and correct after the removal; weather is the sole condition multiplier again.

**Independent Test**: Set weekdays, style, and weather; verify quantities follow the direct rule; verify no input can alter the weather multiplier's effect.

**Acceptance Scenarios**:

1. **Given** weekdays 7, style 1.0, weather 1.5, **When** quantities are calculated, **Then** each auto item's quantity is 11 (7 × 1.5, rounded).
2. **Given** any trip state, **When** quantities are calculated, **Then** the weather multiplier always applies (nothing can replace it).
3. **Given** the calculation's notes, **When** read, **Then** they reference weekdays, style, and weather only.

---

### User Story 3 - Existing files load without activities (Priority: P2)

Configuration and trip files written before this feature (which contain `selected_activities` and `modifiers.activities`) load without error; the activity data is ignored and not rewritten. New saves omit the activity fields entirely.

**Why this priority**: The user has existing files; a hard failure on leftover fields would block them for data the feature simply no longer uses.

**Independent Test**: Load a pre-feature config and trip file; verify they load, the trip resumes, and the next save contains no activity fields.

**Acceptance Scenarios**:

1. **Given** a config file containing `selected_activities` and `modifiers.activities`, **When** it is loaded, **Then** it loads without error and the activity data is ignored.
2. **Given** a trip file containing `selected_activities`, **When** the trip resumes, **Then** it loads and the field is ignored.
3. **Given** any save after this feature, **When** the file is inspected, **Then** no activity fields are written.

### Edge Cases

- **EC-001**: What happens to the weather multiplier's role?
  **Answer**: It applies directly, always — the activity-priority rule is gone with its input. The calculation is the pre-activity rule.
  **Acceptance Criteria**: No input or state can suppress the weather multiplier.

- **EC-002**: Are the built-in activity multipliers (hiking, beach, ...) kept anywhere?
  **Answer**: No — the default activity list is removed with the dimension. Config files that still carry `modifiers.activities` simply have that section ignored on load.
  **Acceptance Criteria**: The application defines no activity multipliers and reads none.

- **EC-003**: Does the welcome screen's focus cycle skip cleanly?
  **Answer**: Yes — the cycle is file → weekdays → style → weather → file; Up/Down and the field rendering work identically to before, minus one field.
  **Acceptance Criteria**: Focus cycling visits exactly the four remaining fields.

- **EC-004**: What about the config screen's Tab cycle with two sections?
  **Answer**: Tab/BackTab cycle travel styles ↔ weather; the third section is gone.
  **Acceptance Criteria**: Section switching visits exactly two sections.

- **EC-005**: Are the removed activity fields deleted from existing files on next save?
  **Answer**: They are not written, which removes them from the file on the next save (the file is rewritten in full). The load-side tolerance (EC-002) covers files not yet re-saved.
  **Acceptance Criteria**: A re-saved file contains no activity fields; an un-re-saved legacy file still loads.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST NOT present any activity input, selection, configuration, or display on any screen.
- **FR-002**: The quantity calculation MUST apply the weather multiplier directly; the activity-priority rule MUST be removed.
- **FR-003**: The configuration file MUST NOT be written with activity fields; activity fields present in existing files MUST be ignored on load without error.
- **FR-004**: The trip file MUST NOT be written with activity fields; the field present in existing trip files MUST be ignored on load without error.
- **FR-005**: The help screen and all hints MUST contain no activity references.
- **FR-006**: The welcome screen's field cycle MUST cover exactly the four remaining fields; the config screen's section cycle MUST cover exactly the two remaining sections.

### Key Entities *(include if feature involves data)*

- **Quantity calculation (revised)**: base (weekdays, capped at 20) × travel-style multiplier × weather multiplier, rounded, minimum 1 — the pre-activity rule.
- **Removed dimension**: Activities — input field, config section, multipliers, persistence, and calculation path are all deleted.
- **Legacy tolerance**: Load-side ignoring of leftover activity fields in existing files.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero activity references across all screens, hints, and help.
- **SC-002**: 100% of calculations follow the direct weather rule, verified by the calculation tests (activity tests removed, weather tests extended).
- **SC-003**: 100% of pre-feature config and trip files load without error.
- **SC-004**: 100% of new saves contain no activity fields.
- **SC-005**: All remaining behavior (travel styles, weather, weekdays, fixed items) passes its tests unchanged.

## Assumptions

- The removal is permanent and complete: no hidden activity state, no dormant code paths (constitution Principle I — removed, not annotated).
- The weather multiplier becomes the sole condition input; its values and management are unchanged.
- Legacy files are tolerated on load (ignored fields) but rewritten without them on save; this is not version compatibility machinery, just not failing on harmless extra data.
- Specs 002 and 005 (Implemented) will need alignment notes recording the removal; spec 015's US3 (unknown-activity fallback) becomes moot with the dimension.
- The help extraction test (spec 015) is updated in the same change so the help cannot retain stale activity entries.
