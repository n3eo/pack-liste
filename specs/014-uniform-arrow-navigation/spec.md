# Feature Specification: Uniform Arrow-Key Navigation Model

**Feature Branch**: `014-uniform-arrow-navigation`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"Navigation shall always use up and down
for category change or number increase/decrease use left and right"

> **Scope note**: The application's arrow keys currently mean different
> things per screen: on the main screen Left/Right collapse/expand the
> category under the cursor, while on the trip setup screen Up/Down move
> between fields and Left/Right adjust the focused value. This feature
> establishes one uniform model across all screens: **Up/Down always
> navigate** (move the cursor through rows, fields, or list entries) and
> **Left/Right always change the focused value** (cycle a category-like
> choice, or increase/decrease a number). No screen repurposes the arrows.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Up/Down always navigate (Priority: P1)

On every screen and in every mode, Up and Down move the user's position: the cursor in the item lists, the focused field on the trip setup screen, the selected entry in the configuration lists, the cursor position in the raw config editor, and the highlighted choice in dropdowns. Up/Down never change a value and never collapse or expand anything.

**Why this priority**: This is half of the uniform model. Navigation is the most frequent action; giving it one meaning everywhere means the user never has to remember which screen they are on to know what the arrows do.

**Independent Test**: On each screen, press Up/Down and verify only the position (cursor, field focus, or selection) changes; no value, quantity, category, or collapse state changes anywhere.

**Acceptance Scenarios**:

1. **Given** the main screen, **When** the user presses Up/Down, **Then** the cursor moves through the visible rows of the focused list and nothing else changes.
2. **Given** the trip setup screen, **When** the user presses Up/Down, **Then** the focus moves between the fields (file, weekdays, travel style, weather, activities) and no value changes.
3. **Given** the configuration screen, **When** the user presses Up/Down, **Then** the cursor moves through the modifier list of the focused section.
4. **Given** the raw config editor, **When** the user presses Up/Down, **Then** the text cursor moves between lines (existing behavior, unchanged).
5. **Given** a dropdown in the add/edit item dialog, **When** the user presses Up/Down, **Then** the highlighted choice moves (existing behavior, unchanged).

---

### User Story 2 - Left/Right always change the focused value (Priority: P1)

On every screen, Left and Right change the value at the user's current position: cycling the travel style or weather option, increasing/decreasing the weekday number, moving the activity cursor through the activity list, and cycling the category choice in the add/edit item dialog. Left/Right never move the cursor between rows or fields.

**Why this priority**: This is the other half of the model. Value adjustment gets the horizontal arrows everywhere, matching the mental model "horizontal = adjust the thing I'm on, vertical = move between things".

**Independent Test**: On the trip setup screen, focus each field and press Left/Right; verify the focused value changes (or the activity cursor moves) and the field focus never changes. In the add/edit dialog's category field, verify Left/Right cycles the category selection.

**Acceptance Scenarios**:

1. **Given** the trip setup screen with the weekdays field focused, **When** the user presses Right/Left, **Then** the weekday number increases/decreases within its valid range.
2. **Given** the trip setup screen with the travel style (or weather) field focused, **When** the user presses Right/Left, **Then** the selection cycles through the configured options.
3. **Given** the trip setup screen with the activities field focused, **When** the user presses Right/Left, **Then** the activity cursor moves through the activity list (the existing Left/Right behavior for this field, now consistent with the model).
4. **Given** the add/edit item dialog with the category field focused, **When** the user presses Right/Left, **Then** the selected category cycles through the available categories.

---

### User Story 3 - Collapse/expand moves off the arrows (Priority: P1)

On the main screen, Left/Right no longer collapse or expand the category under the cursor. Collapsing and expanding remain available through the existing dedicated key, and the behavior (cursor to the header when collapsing, items hidden, Ctrl+Space selection-follows-collapse) is unchanged — only the arrow binding is removed.

**Why this priority**: The arrows cannot mean "adjust value" on the main screen while meaning collapse/expand; the conflict is the core inconsistency this feature removes. The dedicated toggle key already exists, so nothing is lost.

**Independent Test**: On the main screen, press Left/Right on a category header and verify nothing collapses or expands; press the collapse toggle key and verify it still collapses/expands with the existing cursor behavior.

**Acceptance Scenarios**:

1. **Given** the main screen with the cursor on a category, **When** the user presses Left/Right, **Then** no collapse or expand happens.
2. **Given** the main screen with the cursor on a category, **When** the user presses the collapse toggle key, **Then** the category collapses (or expands) exactly as before, with the cursor on the header when collapsed.
3. **Given** the main screen, **When** the user toggles a category's selection with Ctrl+Space, **Then** the follow-selection collapse/expand behavior is unchanged.

---

### User Story 4 - Hints and help match the model (Priority: P2)

Every on-screen hint and the help screen describe the new model: Up/Down navigate, Left/Right adjust. No hint or help text tells the user that Left/Right collapse/expand or that Up/Down change values.

**Why this priority**: Hints that contradict the behavior are worse than no hints; the model is only uniform if the documentation of it is uniform too.

**Independent Test**: Read every screen's hint line and the help screen; verify each describes Up/Down as navigation and Left/Right as value adjustment wherever the arrows are mentioned.

**Acceptance Scenarios**:

1. **Given** the trip setup screen, **When** the user reads the hints, **Then** they state Up/Down for field navigation and Left/Right for value adjustment (matching current hints, verified consistent).
2. **Given** the main screen, **When** the user reads the hints and help, **Then** collapse/expand is documented under its dedicated key, not under Left/Right.
3. **Given** the add/edit item dialog, **When** the user reads the hints, **Then** the category cycling is documented under Left/Right (and the dropdown highlight under Up/Down).

### Edge Cases

- **EC-001**: What do Left/Right do on the main screen when a row is focused — is there a value to adjust?
  **Answer**: Nothing. The main screen's rows have no adjustable value bound to the arrows; Left/Right are reserved for value adjustment and simply have no effect there. No hidden alternative binding is introduced.
  **Acceptance Criteria**: Left/Right on the main screen change nothing.

- **EC-002**: Do Left/Right still adjust values while a text field is being edited (trip file name, item name, raw config editor)?
  **Answer**: No — text editing modes keep the arrows as text-cursor movement (raw editor) or ignore them (single-line inputs), as today. The uniform model applies to selection/navigation contexts, not to text editing.
  **Acceptance Criteria**: Editing text with the arrows behaves exactly as before this feature.

- **EC-003**: Does the weekday number still respect its valid range under Left/Right?
  **Answer**: Yes — the existing 1–30 validation and clamping apply unchanged; Left/Right only change which adjustment happens, not the validation.
  **Acceptance Criteria**: Left/Right cannot push the weekday number outside 1–30.

- **EC-004**: What happens to the +/- keys that currently also adjust the weekdays?
  **Answer**: They keep working as aliases for increase/decrease. Removing them is not requested; the model defines the arrows, it does not forbid aliases.
  **Acceptance Criteria**: +/- still adjust the focused weekday field.

- **EC-005**: In the add/edit dialog, do Left/Right cycle the category even when the category dropdown is not open?
  **Answer**: Yes — the category field's value is its selection; Left/Right adjust it whenever the category field is focused, which is the uniform model. Up/Down continue to move the dropdown highlight when it is open.
  **Acceptance Criteria**: With the category field focused, Left/Right change the selected category in both the open and closed dropdown states.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Up/Down MUST navigate on every screen: move the list cursor, move the field focus, or move the selection highlight, according to the screen; they MUST NOT change any value or collapse/expand anything.
- **FR-002**: Left/Right MUST change the focused value on every screen where a value is adjustable: cycle list-valued options (travel style, weather, category choice) and increase/decrease numeric values (weekdays), and move the activity cursor within the activities field.
- **FR-003**: Left/Right MUST NOT collapse or expand categories on the main screen; collapse/expand MUST remain available under its existing dedicated key with unchanged behavior.
- **FR-004**: The add/edit item dialog's category field MUST cycle its selection with Left/Right when focused.
- **FR-005**: Text editing contexts (raw config editor, text input fields) MUST keep their existing arrow behavior for text-cursor movement.
- **FR-006**: All on-screen hints and the help screen MUST describe the uniform model: Up/Down navigate, Left/Right adjust.
- **FR-007**: The existing validation of adjusted values (weekday range 1–30) MUST be unchanged.

### Key Entities *(include if feature involves data)*

- **Navigation (Up/Down)**: Moving the user's position — list cursor, field focus, or selection highlight — without side effects on values.
- **Value adjustment (Left/Right)**: Changing the value at the current position — cycling an option or stepping a number — without moving between rows or fields.
- **Collapse toggle**: The dedicated key that collapses/expands the category under the cursor; previously also bound to Left/Right, now the only binding.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On 100% of screens, Up/Down only navigate and Left/Right only adjust values (or have no effect where nothing is adjustable).
- **SC-002**: 100% of value adjustments that were possible with the arrows before remain possible with the arrows after (weekdays, travel style, weather, activities cursor, category choice).
- **SC-003**: Collapse/expand remains reachable in one key press (the dedicated toggle), with identical behavior to before.
- **SC-004**: 100% of arrow-key hints across all screens and the help screen match the uniform model.
- **SC-005**: All existing tests pass; navigation and adjustment behaviors carry tests reflecting the new bindings.

## Assumptions

- The uniform model is: vertical arrows move between things (rows, fields, entries); horizontal arrows change the thing in focus. Screens with nothing adjustable under Left/Right simply ignore them (EC-001).
- The existing dedicated collapse toggle key (C) remains the collapse/expand binding; no new key is introduced.
- The +/- aliases for weekday adjustment stay; the feature defines the arrow model, it does not remove existing aliases.
- Text editing (raw config editor cursor movement, text input) is out of the model's scope and keeps its current arrow behavior.
- The trip setup screen's current hints already describe the target model ("Up/Down to navigate fields, Left/Right to change values"); this feature extends that model to the main screen and the item dialogs, and updates the hints that contradict it.
