# Feature Specification: Edit Dialog Navigation

**Feature Branch**: `020-edit-dialog-navigation`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"In the edit menu use up and down to cycle through the fields
Also improve readability by having a cursor in free text fields
also support increasing and lowering quantities"

> **Scope note**: The add/edit item dialog is the one surface that
> still deviates from the uniform arrow model (spec 014): its fields
> are cycled with Tab, Up/Down only move the category dropdown's
> highlight, and the quantity must be typed digit by digit. This
> feature brings the dialog into the model: Up/Down cycle the dialog's
> fields (Name → Category → Quantity → Mode), Left/Right adjust the
> focused value — stepping the quantity up and down when the Quantity
> field is focused — and the free-text fields (Name, Quantity) show a
> visible text cursor so the user can see where typed characters land.
> Tab keeps working as an alias for field cycling; the category
> dropdown's Up/Down highlight movement is preserved.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Up/Down cycle the dialog's fields (Priority: P1)

The user opens the add or edit dialog and moves between its four fields with Up and Down: pressing Down steps Name → Category → Quantity → Mode → Name (wrapping), pressing Up steps the reverse. The field focus highlight follows, so the user always sees which field is active. Up/Down no longer do nothing when a non-category field is focused.

**Why this priority**: This is the first explicit request, and the dialog is the last surface where Up/Down are not navigation — the uniform model's biggest remaining gap.

**Independent Test**: Open the edit dialog; press Down repeatedly and verify the focus visits all four fields in order and wraps; press Up and verify the reverse order; verify the category dropdown's highlight movement still works when the Category field is focused.

**Acceptance Scenarios**:

1. **Given** the edit dialog with the Name field focused, **When** the user presses Down three times, **Then** the focus is on the Mode field.
2. **Given** the edit dialog with the Name field focused, **When** the user presses Down four times, **Then** the focus is back on the Name field (wrapping).
3. **Given** the edit dialog with any field focused, **When** the user presses Up, **Then** the focus moves to the previous field (wrapping from Name to Mode).
4. **Given** the edit dialog with the Category field focused and the dropdown open, **When** the user presses Up/Down, **Then** the dropdown highlight moves (existing behavior preserved).

---

### User Story 2 - A visible cursor in the free-text fields (Priority: P1)

The Name and Quantity fields are free-text inputs. When one is focused, a visible text cursor (a highlighted block or underscore at the insertion position) shows where the next typed character will land and where Backspace removes. The cursor position advances as the user types and retreats as the user deletes; the cursor is only visible while its field is focused.

**Why this priority**: The second explicit request — without a cursor, the user cannot see the insertion point in a field whose content may be longer than what is visually obvious next to the label, making typing error-prone.

**Independent Test**: Focus the Name field, type a name, and verify the cursor sits after the last character; press Backspace and verify the cursor retreats with the deleted character; focus another field and verify the cursor disappears.

**Acceptance Scenarios**:

1. **Given** the Name field focused with text "Shirt", **When** the dialog renders, **Then** a cursor is visible immediately after "Shirt".
2. **Given** the Name field focused, **When** the user types a character, **Then** the cursor moves one position right (after the new character).
3. **Given** the Name field focused, **When** the user presses Backspace, **Then** the last character is removed and the cursor sits after the new last character.
4. **Given** the Category or Mode field focused, **When** the dialog renders, **Then** no text cursor is shown (these fields are not free-text).

---

### User Story 3 - Left/Right increase and lower the quantity (Priority: P1)

With the Quantity field focused, Left and Right adjust the quantity: Right increases it by one, Left decreases it by one, never below the minimum of 1. The change is visible immediately in the field. Typing digits still works; the arrows are an adjustment, not a replacement for typing. With the Category field focused, Left/Right keep their existing category-cycling behavior.

**Why this priority**: The third explicit request, and the uniform model's "horizontal = adjust the value" applied to the dialog's one numeric field — matching the weekdays field on the welcome screen.

**Independent Test**: Focus the Quantity field with value 3; press Right twice and verify 5; press Left and verify 4; press Left four times and verify 1 (clamped, never 0); type a digit and verify it still appends.

**Acceptance Scenarios**:

1. **Given** the Quantity field focused with value 3, **When** the user presses Right, **Then** the field shows 4.
2. **Given** the Quantity field focused with value 3, **When** the user presses Left, **Then** the field shows 2.
3. **Given** the Quantity field focused with value 1, **When** the user presses Left, **Then** the field still shows 1 (the minimum).
4. **Given** the Quantity field focused, **When** the user types a digit, **Then** the digit appends to the field as before.
5. **Given** the Category field focused, **When** the user presses Left/Right, **Then** the category selection cycles (existing behavior, unchanged).

---

### User Story 4 - Hints, help, and tests state the dialog's model (Priority: P2)

The dialog's hint line and the help screen describe the final binding set: Up/Down cycle fields (and move the dropdown highlight in the Category field), Left/Right adjust the focused value (category cycling, quantity stepping), Tab remains as the field-cycling alias. The hint/help extraction tests are extended so the dialog's bindings cannot drift.

**Why this priority**: An undocumented binding is an unfindable binding; the project's hint system and drift-guard tests are the established discovery and enforcement surfaces.

**Independent Test**: Read the dialog's hint line and the help screen's dialog section; verify the new bindings are documented; remove an entry and verify the extraction test fails.

**Acceptance Scenarios**:

1. **Given** the add/edit dialog's hint line, **When** read, **Then** it lists Up/Down for field cycling, Left/Right for value adjustment, and Tab as the alias.
2. **Given** the help screen's dialog section, **When** read, **Then** the same bindings are documented in the same terms.
3. **Given** the extraction tests, **When** a dialog binding is removed from the hints or help, **Then** the test fails.

---

### Edge Cases

- **EC-001**: With the Category field focused, do Up/Down cycle fields or move the dropdown highlight?
  **Answer**: The dropdown highlight — the category list is a vertical list, and moving through it is the navigation in that context; field cycling is available from any other field, and Tab remains available everywhere as the alias.
  **Acceptance Criteria**: Up/Down in the Category field move the dropdown highlight; Up/Down in the other three fields cycle the field focus.

- **EC-002**: What does the quantity do when it is empty or unparsable and Left/Right are pressed?
  **Answer**: An empty or unparsable quantity field starts stepping from 1 (the minimum): Right goes to 2, Left stays at 1. The arrows never produce an invalid value.
  **Acceptance Criteria**: Left/Right always leave the quantity field holding a valid number ≥ 1.

- **EC-003**: Does the text cursor support Left/Right movement within the text?
  **Answer**: No — the cursor shows the insertion position (always at the end, matching the current append-only editing); Left/Right are value-adjustment keys in the dialog, not text-cursor keys. The cursor is a readability aid, not a text-navigation feature.
  **Acceptance Criteria**: Left/Right never move the text cursor within a field's text; the cursor stays at the end.

- **EC-004**: Does the Mode field participate in Up/Down cycling?
  **Answer**: Yes — it is one of the dialog's four fields; Up/Down move the focus to and from it, and Space keeps toggling it when focused.
  **Acceptance Criteria**: The focus cycle visits all four fields including Mode.

- **EC-005**: Does the add dialog behave the same as the edit dialog?
  **Answer**: Yes — both dialogs share the fields and the bindings; the feature applies to both.
  **Acceptance Criteria**: All acceptance scenarios hold in the add dialog too.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: In the add/edit dialog, Up/Down MUST cycle the field focus through Name → Category → Quantity → Mode (wrapping), except in the Category field where Up/Down MUST keep moving the dropdown highlight.
- **FR-002**: The free-text fields (Name, Quantity) MUST display a visible text cursor at the insertion position while focused; the cursor MUST advance on input and retreat on deletion; it MUST NOT be shown while another field is focused.
- **FR-003**: With the Quantity field focused, Left/Right MUST increase/decrease the quantity by one, clamped at the minimum of 1; typing digits MUST keep working.
- **FR-004**: With the Category field focused, Left/Right MUST keep cycling the category selection (unchanged).
- **FR-005**: Tab MUST remain available as a field-cycling alias in all fields.
- **FR-006**: The dialog's hint line and the help screen MUST document the final binding set, and the extraction tests MUST cover it.

### Key Entities

- **Dialog field cycle**: The four-field focus ring (Name, Category, Quantity, Mode) navigated by Up/Down and aliased by Tab.
- **Text cursor**: The visible insertion-position marker in free-text fields; a readability aid at the end of the text.
- **Quantity stepping**: Left/Right adjusting the numeric quantity by ±1 within its valid range (minimum 1).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of the dialog's fields are reachable with Up/Down from any field (with the Category exception per EC-001, where Tab covers it).
- **SC-002**: The text cursor is visible in 100% of renders where a free-text field is focused, and absent in 100% of renders where it is not.
- **SC-003**: 100% of Left/Right presses on the Quantity field change the value by exactly one, never below 1.
- **SC-004**: The dialog's hints and help agree with its behavior, enforced by the extraction tests.
- **SC-005**: All existing dialog behavior (Tab cycling, category cycling, mode toggle, save/cancel) passes its tests unchanged.

## Assumptions

- The uniform arrow model (spec 014) governs: vertical = move between things, horizontal = adjust the thing in focus.
- The text cursor is append-only (insertion at the end), matching the current editing model; in-text cursor movement is out of scope (EC-003).
- The quantity's valid range in the dialog is ≥ 1 (the existing minimum); no upper bound is introduced beyond the existing one.
- Tab remains a working alias; removing it is not requested.
- The dialog's hint line has room for the updated entries at the supported minimum size (the hint rows reflow automatically per the existing hint system).
