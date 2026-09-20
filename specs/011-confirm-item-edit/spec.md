# Feature Specification: Confirm Before Saving Item Edits

**Feature Branch**: `011-confirm-item-edit`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"Ask for confirmation before saving an edited item to the config.yaml"

> **Re-invoked 2026-09-20** with: "in the edit field ask for
> confirmation before saving" — the same request, phrased against the
> edit dialog's save action. The existing spec already covers it
> exactly (US1: a confirmation prompt between the edit dialog and the
> write; declining discards the edit with nothing written). No spec
> change was needed; this phrasing is recorded as interpretation
> evidence.

> **Scope note**: Editing an item through the TUI (name, category,
> quantity) currently writes the change to the configuration file
> immediately when the edit dialog is confirmed — there is no second
> chance before the persisted catalog changes. This feature adds a
> confirmation step between the edit dialog and the write, mirroring the
> existing delete flow which already asks "Delete <item>?" before it
> acts. Adding an item keeps its current behavior (no confirmation); only
> edits are confirmed.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Confirm before an edit is written to the configuration (Priority: P1)

The user edits an item in the edit dialog (changes name, category, or quantity) and presses the dialog's save key. Instead of the change being written to the configuration file immediately, a confirmation prompt appears showing what is about to be saved. Only when the user confirms does the edit apply and the configuration file update; cancelling returns to the list with the item unchanged and nothing written.

**Why this priority**: This is the entire request. The configuration file is the user's persisted item catalog; an accidental key press in the edit dialog currently rewrites it with no way back. The confirmation makes the write an explicit, deliberate act.

**Independent Test**: Edit an item, save the dialog, decline the confirmation; verify the item is unchanged in the list and the configuration file was not modified. Repeat and accept; verify the item changed and the file contains the new values.

**Acceptance Scenarios**:

1. **Given** the user has edited an item in the edit dialog, **When** they confirm the dialog, **Then** a confirmation prompt appears describing the pending change before anything is saved.
2. **Given** the confirmation prompt is shown, **When** the user confirms, **Then** the edit is applied to the list and written to the configuration file.
3. **Given** the confirmation prompt is shown, **When** the user cancels, **Then** the edit is discarded, the item keeps its previous values, and the configuration file is unchanged.

---

### User Story 2 - The confirmation shows what will change (Priority: P2)

The confirmation prompt is not a blind yes/no: it shows the item's new values (and, where space allows, the previous ones) so the user can verify the edit before committing. The prompt names the item and the fields about to change.

**Why this priority**: A confirmation without content forces the user to remember what they typed; showing the pending change makes the confirmation meaningful and reduces wrong confirms.

**Independent Test**: Edit an item's name and quantity, save the dialog, and verify the prompt shows the new name and quantity before confirming.

**Acceptance Scenarios**:

1. **Given** an edit that changes the item's name and quantity, **When** the confirmation prompt appears, **Then** it shows the new name and the new quantity.
2. **Given** an edit that leaves a field unchanged, **When** the confirmation prompt appears, **Then** the unchanged field is presented as staying the same (or shown identically in old/new form).
3. **Given** the confirmation prompt, **When** the terminal is small, **Then** the prompt fits the terminal and remains answerable (consistent with the small-terminal behavior of other popups).

---

### User Story 3 - Cancelling at any stage loses nothing (Priority: P2)

The user can back out of an edit at every stage without side effects: Esc in the edit dialog cancels the edit (existing behavior), and cancel at the confirmation prompt discards the edit without writing. In both cases the list, the item, and the configuration file are exactly as before the edit started.

**Why this priority**: The confirmation is only trustworthy if "cancel" is guaranteed to be side-effect-free; otherwise users cannot safely explore the edit dialog.

**Independent Test**: Start an edit, change fields, save the dialog, cancel the confirmation; verify the item and configuration file are byte-identical to before the edit.

**Acceptance Scenarios**:

1. **Given** an item being edited, **When** the user cancels at the confirmation prompt, **Then** the item's values and the configuration file are identical to before the edit dialog was opened.
2. **Given** the confirmation prompt is open, **When** the user presses Esc, **Then** the prompt closes, the edit is discarded, and the user is back on the list screen.

---

### User Story 4 - Other save paths keep their behavior (Priority: P3)

Adding a new item, deleting an item (which already has its own confirmation), and editing the configuration through the raw config editor are unchanged. The new confirmation applies only to editing an existing item through the TUI edit dialog.

**Why this priority**: The request targets item edits specifically; widening the confirmation to other flows would add friction where the user did not ask for it. Delete already confirms; add and raw-editor saves are deliberate multi-step actions.

**Independent Test**: Add an item and verify no new confirmation appears; delete an item and verify the existing delete confirmation still appears exactly once.

**Acceptance Scenarios**:

1. **Given** the user adds a new item, **When** the add dialog is confirmed, **Then** the item is saved as before, with no additional confirmation prompt.
2. **Given** the user deletes an item, **When** the delete key is pressed, **Then** the existing delete confirmation appears (unchanged), and confirming it applies the deletion as before.
3. **Given** the user saves the configuration through the raw config editor, **When** the save is applied, **Then** it behaves as before, with no additional confirmation.

### Edge Cases

- **EC-001**: What exactly does the confirmation show when only some fields changed?
  **Answer**: The prompt shows the item's resulting values (name, category, quantity) after the edit; fields the edit leaves untouched are shown as-is. A full old/new diff is not required.
  **Acceptance Criteria**: The prompt always identifies the item and its post-edit values.

- **EC-002**: Does the confirmation also apply when the edit dialog is saved with an empty field (existing behavior: empty fields keep the old value)?
  **Answer**: Yes — the confirmation shows the resulting values, which already reflect the keep-old-value rule; the user confirms exactly what will be written.
  **Acceptance Criteria**: An edit saved with an empty name confirms with the previous name shown as the result.

- **EC-003**: What happens if the user quits the application while the confirmation prompt is open?
  **Answer**: Quitting (or terminating) at the prompt discards the edit like a cancel; the configuration file never receives a half-confirmed edit. The exit-time save of the current state is unaffected.
  **Acceptance Criteria**: After quitting at the prompt, the item in the configuration file has its pre-edit values.

- **EC-004**: Does the confirmation appear for every edit, even trivial ones?
  **Answer**: Yes — the confirmation is unconditional for edits. Detecting "trivial" edits adds complexity and a hidden rule; the user asked for a confirmation before saving, so every edit save gets one.
  **Acceptance Criteria**: Every edit-dialog save shows the confirmation, regardless of how many fields changed.

- **EC-005**: Is the confirmation skipped when the edit changes nothing (same values re-saved)?
  **Answer**: No — the same unconditional rule applies; the prompt shows the (unchanged) resulting values and the user confirms. The write is idempotent, so confirming is harmless.
  **Acceptance Criteria**: A no-op edit still shows the confirmation and writes the same values.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When the user confirms the edit dialog for an existing item, the system MUST show a confirmation prompt before applying the edit or writing the configuration file.
- **FR-002**: The confirmation prompt MUST show the item's resulting values (name, category, quantity) so the user can verify the pending change.
- **FR-003**: The edit MUST be applied and the configuration file written only after the user confirms the prompt.
- **FR-004**: Cancelling the confirmation prompt MUST discard the edit entirely: the item keeps its previous values and the configuration file is not written.
- **FR-005**: Esc MUST cancel the confirmation prompt, equivalent to declining it.
- **FR-006**: The confirmation MUST be unconditional for edits made through the TUI edit dialog (no heuristic skipping).
- **FR-007**: Adding an item, deleting an item, and saving through the raw config editor MUST keep their existing behavior, including the existing delete confirmation.
- **FR-008**: The confirmation prompt MUST fit and remain answerable at the minimum supported terminal size, consistent with other popups.

### Key Entities *(include if feature involves data)*

- **Edit confirmation**: A prompt shown between the edit dialog and the save, presenting the item's post-edit values with confirm and cancel actions; declining discards the edit without any write.
- **Item edit**: The existing flow (edit dialog: name, category, quantity) ending, with this feature, in confirm-then-save instead of save-immediately.
- **Configuration file**: The persisted item catalog; the write target guarded by the confirmation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of item edits applied through the TUI are preceded by a confirmation prompt; 0% of edits are written without a confirmation.
- **SC-002**: 100% of cancelled confirmations leave the item and the configuration file unchanged.
- **SC-003**: The confirmation prompt shows the item's resulting name, category, and quantity in every case.
- **SC-004**: The add-item and delete-item flows behave exactly as before this feature (delete keeps its single existing confirmation).
- **SC-005**: The confirmation prompt is answerable at the minimum supported terminal size.

## Assumptions

- The edit dialog's field semantics (empty field keeps the old value; quantity parse failure keeps the old quantity) are unchanged; the confirmation presents the resulting values under those rules.
- The confirmation uses the same interaction pattern as the existing delete confirmation (confirm key to proceed, Esc to cancel), so users meet one consistent idiom.
- The configuration file write happens only after confirmation; the exit-time save of the current state is a separate, existing behavior and is not gated by this confirmation.
- Adding items is intentionally not confirmed: the request names edited items specifically, and the add flow already ends in an explicit dialog confirmation.
- The prompt content is the item's resulting values; a full old/new diff is a possible future refinement, not part of this feature.
