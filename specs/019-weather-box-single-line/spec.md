# Feature Specification: Weather Box Single Line

**Feature Branch**: `019-weather-box-single-line`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"the weather box is until the bottom of the screen, keep it at a single line"

> **Scope note**: On the welcome screen, the four trip-parameter fields
> (packing list file, weekdays, travel style, weather) are laid out as
> equal blocks. After the activities field was removed (spec 017), the
> layout's last row constraint became a flexible "fill remaining space"
> rule, so the weather box stretches from its position down to the
> bottom of the screen — a tall, mostly empty box. The other three
> fields keep their compact single-line height. This feature restores
> the uniform compact layout: the weather box is the same fixed height
> as the other fields, and the freed space below the fields stays
> empty.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The weather field is a compact single-line block (Priority: P1)

The user opens the welcome screen and sees the four trip-parameter fields as visually equal, compact blocks: packing list file, trip duration, travel style, and weather. The weather box no longer stretches to the bottom of the screen; it is the same height as the other three fields, showing its value and its adjustment hint on one line each. The space below the fields is empty, and the navigation hints at the bottom of the screen remain fully visible.

**Why this priority**: This is the entire request — the stretched weather box is a visual defect introduced by the activities removal; restoring the uniform field layout fixes it.

**Independent Test**: Open the welcome screen at several terminal sizes, including small (80x24) and tall; verify the weather box's height matches the other fields' height at every size and that no field touches the screen bottom or the hint footer.

**Acceptance Scenarios**:

1. **Given** the welcome screen at any supported terminal size, **When** it renders, **Then** the weather box's height equals the packing-list-file, weekdays, and travel-style boxes' height.
2. **Given** a tall terminal, **When** the welcome screen renders, **Then** the weather box ends right after its content and the space between it and the hint footer is empty.
3. **Given** the welcome screen, **When** the user cycles focus through the fields, **Then** all four fields remain fully visible and the focus highlight renders correctly on each.

---

### Edge Cases

- **EC-001**: What happens at very small terminal heights?
  **Answer**: The four fixed-height fields keep priority over the empty space; below the minimum supported size the existing too-small behavior applies. The fields never overlap the hint footer.
  **Acceptance Criteria**: At every supported size, the fields and the hint footer never overlap.

- **EC-002**: Does the freed space below the fields get used for anything?
  **Answer**: No — it stays empty. No new content is added below the fields; the fix is purely the weather box's height.
  **Acceptance Criteria**: The area below the weather box contains no content.

- **EC-003**: Do the other three fields change?
  **Answer**: No — their layout, height, and content are untouched; only the weather box's stretching is removed.
  **Acceptance Criteria**: The file, weekdays, and travel-style boxes render identically to before this change.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The weather field on the welcome screen MUST render at the same fixed height as the other trip-parameter fields (a single-line value block), at every supported terminal size.
- **FR-002**: The weather field MUST NOT stretch to fill the remaining vertical space of the welcome screen.
- **FR-003**: The layout of the other three trip-parameter fields MUST remain unchanged.
- **FR-004**: The welcome screen's hint footer MUST remain fully visible and unoverlapped at every supported terminal size.

### Key Entities

- **Welcome screen field layout**: The vertical arrangement of the four trip-parameter fields; the last field must not absorb the screen's remaining height.
- **Weather field**: The trip parameter selecting the weather condition; displays its current value and adjustment hint.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At 100% of supported terminal sizes, the weather box's height equals the other fields' height.
- **SC-002**: Zero overlap between the trip-parameter fields and the hint footer at any supported size.
- **SC-003**: All existing welcome-screen behavior (focus cycling, value adjustment, file editing) passes its tests unchanged.

## Assumptions

- The fix is a layout change only; no field content, binding, or behavior changes.
- The four fields keep their current order: file, weekdays, travel style, weather.
- The empty space below the fields is acceptable and expected; no filler content is requested.
- The regression was introduced by spec 017's removal of the activities field (the layout's last constraint became a fill-remaining rule); this spec fixes the layout without restoring any activity functionality.
