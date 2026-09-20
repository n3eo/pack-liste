# Feature Specification: Recalculate Amounts on an Existing List

**Feature Branch**: `018-recalculate-action`

**Created**: 2026-09-20

**Status**: Draft (amended 2026-09-20)

**Input**: User description:
"Add option to recalculate the amounts on existing list"

Amended 2026-09-20 with:
"fix that individual items quantity can be overwritten manually
currently the calculation always trumps any manual quantity
re calculations shall only trigger when pressing 'R'"

Amended again 2026-09-20 with:
"recalculations using 'r' shall only recalculate the current item, use space + 'r' for the group and space + shift + 'r' for everything"

> **Scope note (amended)**: The original feature added an explicit
> `R` (Recalculate) action on top of an always-recalculate model:
> quantities recalculated automatically on every trip-parameter change,
> config save, and item add/edit, and a manually edited quantity on an
> automatic item was overwritten by the next recalculation. The
> amendments **reverse that model**: manual quantity edits are
> authoritative and stick; recalculation happens **only** when the user
> explicitly presses a recalculate key — never as an automatic side
> effect of parameter changes, config saves, or item edits. The
> recalculate action itself gains three scopes: the current item, the
> current item's category (group), and everything.
>
> **Binding note**: The second amendment requested "space + r" and
> "space + shift + r". Terminals report only Shift, Ctrl, and Alt as
> key modifiers — Space cannot be detected as a held modifier, so those
> chords are not implementable. The user chose the equivalent
> modifier-based scheme: **R** (current item), **Ctrl+R** (the item's
> category), **Ctrl+Shift+R** (everything).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Manual quantity edits are authoritative (Priority: P1)

The user edits an item's quantity — through the edit dialog, or by any future direct means — and the value they set sticks. No automatic recalculation overwrites it: changing trip parameters (weekdays, style, weather), saving the config, adding items, or editing other items leaves the manually set quantity untouched. The calculation is no longer a background authority; it is a tool the user invokes explicitly.

**Why this priority**: This is the core amendment — "currently the calculation always trumps any manual quantity" is a data-loss defect from the user's perspective: they set a value, the application silently replaces it. Manual edits must win.

**Independent Test**: Set an item's quantity to a manual value; change every trip parameter; add and edit other items; save the config; verify the manually set quantity is unchanged throughout.

**Acceptance Scenarios**:

1. **Given** an item whose quantity was manually set to 3, **When** the user changes the weekdays, travel style, or weather, **Then** the item's quantity remains 3.
2. **Given** an item whose quantity was manually set to 3, **When** the user adds another item or edits a different item, **Then** the item's quantity remains 3.
3. **Given** an item whose quantity was manually set to 3, **When** the configuration is saved (including through the raw config editor), **Then** the item's quantity remains 3.
4. **Given** a fresh list or a resumed trip, **When** it loads, **Then** quantities are calculated once for items without a manual value (the existing load behavior), and this initial calculation does not overwrite quantities the trip file restored.

---

### User Story 2 - Recalculation happens only on explicit key press (Priority: P1)

Nothing recalculates quantities as a side effect. The automatic recalculation triggers that exist today — on every trip-parameter change, on config saves, on modifier add/delete, on item add/edit — are removed. The only events that recalculate quantities are the three recalculate keys. Parameter changes still update the parameters; they just do not touch quantities until the user asks.

**Why this priority**: The user's words are explicit: "re calculations shall only trigger when pressing 'R'." Automatic triggers are the mechanism by which manual values were being trumped; removing them is the fix.

**Independent Test**: Change every trip parameter and perform every save/edit action; verify no item's quantity changes at any point. Then press a recalculate key and verify the quantities update.

**Acceptance Scenarios**:

1. **Given** any list, **When** the user changes weekdays, travel style, or weather, **Then** no item's quantity changes.
2. **Given** any list, **When** the user adds an item, edits an item, or saves the config, **Then** no other item's quantity changes.
3. **Given** any list, **When** the user presses one of the three recalculate keys, **Then** the quantities in the key's scope recalculate (per US3).

---

### User Story 3 - Three recalculate scopes (Priority: P1)

The recalculate action has three scopes, one key each:

- **R** — recalculate the **current item** (the item under the cursor). Only that item's quantity is recalculated from the current trip parameters and its per-item configuration.
- **Ctrl+R** — recalculate the **current item's category** (the group). Every item in the category the cursor is in recalculates.
- **Ctrl+Shift+R** — recalculate **everything**. Every item in the list recalculates.

In all scopes, items marked fixed keep their fixed values (the existing auto/fixed distinction); only automatic items recalculate. The actions are idempotent per scope: repeating a press with unchanged inputs produces the same numbers.

**Why this priority**: The user's second amendment defines the scopes; the binding is the chosen equivalent of the requested space-chords. Scope control makes recalculation safe to use on a list where some quantities were manually set — the user recalculates exactly what they want, leaving manual values elsewhere untouched.

**Independent Test**: With a list containing three categories and a manually set quantity in each, place the cursor on an item in the first category: press R and verify only that item changed; press Ctrl+R and verify the whole first category changed and the others did not; press Ctrl+Shift+R and verify every automatic item changed and the fixed items did not.

**Acceptance Scenarios**:

1. **Given** the cursor on an automatic item, **When** the user presses R, **Then** only that item's quantity recalculates; every other item is unchanged.
2. **Given** the cursor on an item of category "Clothing", **When** the user presses Ctrl+R, **Then** every automatic item in "Clothing" recalculates; items in other categories are unchanged.
3. **Given** any cursor position, **When** the user presses Ctrl+Shift+R, **Then** every automatic item in the list recalculates.
4. **Given** a fixed item in any scope, **When** the scope recalculates, **Then** the fixed item's quantity is unchanged.
5. **Given** the cursor on a category header (not an item), **When** the user presses R or Ctrl+R, **Then** the category's items recalculate (the header identifies the group; R and Ctrl+R are equivalent there).

---

### User Story 4 - The bindings are discoverable and documented (Priority: P2)

The three recalculate keys are shown in the list views' hint lines and in the help screen, grouped with the other list actions, each with its scope. The hint and help entries are covered by the extraction tests so they cannot drift.

**Why this priority**: Three scoped actions are undiscoverable without documentation; the hint system (spec 016) and the help contract (spec 015) are the established discovery surfaces.

**Independent Test**: Read the list views' hint lines and the help screen; verify all three keys appear with their scopes; remove one and verify the extraction test fails.

**Acceptance Scenarios**:

1. **Given** the main screen's hint lines, **When** read, **Then** the recalculate keys and their scopes are listed.
2. **Given** the help screen, **When** read, **Then** all three keys are documented under the main screen's actions.
3. **Given** the extraction tests, **When** a recalculate binding is removed from hints or help, **Then** the test fails.

---

### User Story 5 - Persistence and load behavior (Priority: P2)

Quantities — manual and recalculated alike — persist with the trip per the existing save/load behavior, and a resumed trip restores its saved quantities without recalculating (the existing resume behavior, now guaranteed by US2's no-side-effect rule). The initial load of a fresh list calculates quantities once for items without restored values.

**Why this priority**: The no-automatic-recalculation model must not lose data: what the user sees must survive restarts exactly as they left it.

**Independent Test**: Set manual quantities, recalculate a scope, exit, resume; verify the resumed quantities are exactly the pre-exit ones.

**Acceptance Scenarios**:

1. **Given** a list with manual and recalculated quantities, **When** the user exits and resumes, **Then** every quantity is restored as it was on exit.
2. **Given** a fresh trip (no trip file), **When** it starts, **Then** quantities are calculated once from the current parameters (the existing initial calculation).

### Edge Cases

- **EC-001**: What does "the current item" mean when the cursor is on a category header?
  **Answer**: The header identifies its category; R and Ctrl+R both recalculate that category's items (they are equivalent on a header). Ctrl+Shift+R is unaffected.
  **Acceptance Criteria**: All three keys work from any cursor position, including headers.

- **EC-002**: What does R do on the right panel (the compiled list)?
  **Answer**: The same as on the left — the current item is the item under the cursor; Ctrl+R uses that item's category. Both panels support all three scopes.
  **Acceptance Criteria**: The scopes work identically from both list views.

- **EC-003**: Does the edit dialog's save still recalculate the edited item?
  **Answer**: No — saving an edit applies the dialog's values (including the quantity the user typed) and nothing else; the edited item's quantity is exactly what the dialog says. If the user wants the item recalculated, they press R on it afterwards.
  **Acceptance Criteria**: After an edit save, the item's quantity equals the dialog's value; no other item changes.

- **EC-004**: Does adding an item still calculate the new item's quantity?
  **Answer**: The new item starts with the quantity from its dialog (spec 021) — no recalculation is triggered. The user can recalculate it (or its group, or everything) explicitly.
  **Acceptance Criteria**: After an add save, the new item's quantity equals its dialog value; no other item changes.

- **EC-005**: What happens to the recalculate-on-resume behavior?
  **Answer**: Resume restores the trip file's quantities without recalculating (the existing behavior); the load-time calculation applies only to items without a restored value (a fresh list).
  **Acceptance Criteria**: A resumed trip's quantities are exactly the saved ones until the user presses a recalculate key.

- **EC-006**: Do the recalculate keys work on the welcome screen or in dialogs?
  **Answer**: No — the keys are main-screen list-view actions (normal mode only), like the other list actions.
  **Acceptance Criteria**: The keys do nothing outside the main screen's list views.

- **EC-007**: Is there feedback that a recalculation happened?
  **Answer**: The updated quantities are the feedback (the existing model). No toast or dialog.
  **Acceptance Criteria**: The action's effect is visible in the quantities alone.

- **EC-008**: What about the raw config editor's apply flow, which recalculates today?
  **Answer**: Applying the raw config updates the in-memory configuration (modifiers, items) but does not recalculate quantities; the user presses a recalculate key to bring quantities in line with the new configuration.
  **Acceptance Criteria**: A raw-editor apply changes no quantity.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A manually set item quantity MUST be authoritative: no automatic process (parameter change, config save, item add/edit, raw-editor apply, load/resume of an existing trip) MAY overwrite it.
- **FR-002**: Recalculation MUST happen only when the user presses one of the three recalculate keys; the automatic recalculation triggers (parameter changes, config saves, modifier edits, item add/edit) MUST be removed.
- **FR-003**: The system MUST provide three recalculate scopes on the main screen's list views: **R** = the current item, **Ctrl+R** = the current item's category, **Ctrl+Shift+R** = every item.
- **FR-004**: In every scope, fixed-quantity items MUST NOT change; only automatic items recalculate.
- **FR-005**: Each scope's action MUST be idempotent for unchanged inputs.
- **FR-006**: The actions MUST NOT modify packed states, selection states, collapse states, or write any file.
- **FR-007**: The three keys MUST be listed in the list views' hint lines and the help screen, covered by the extraction tests.
- **FR-008**: A fresh list's initial load MUST calculate quantities once for items without restored values; a resumed trip MUST restore saved quantities without recalculating.
- **FR-009**: Saving an item edit MUST apply exactly the dialog's values (the edited item's quantity is the dialog's quantity); adding an item MUST apply exactly the dialog's values; neither MAY trigger a recalculation of any item.

### Key Entities *(include if feature involves data)*

- **Recalculate scopes**: The three explicit recalculation targets — current item, current category, everything — bound to R, Ctrl+R, Ctrl+Shift+R.
- **Manual quantity**: A quantity set by the user (edit dialog, or restored from a saved trip); authoritative until the user explicitly recalculates its scope.
- **Automatic recalculation triggers** (removed): The former side-effect paths that recalculated quantities on parameter changes, config saves, and item add/edit.
- **Quantity modes**: The existing automatic/fixed distinction; recalculate applies to automatic items only, in all scopes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of manual quantity edits survive every non-recalculate action (parameter changes, saves, add/edit of other items, raw-editor applies, restarts).
- **SC-002**: 0% of non-recalculate events change any item's quantity.
- **SC-003**: 100% of R presses recalculate exactly the current item; 100% of Ctrl+R presses exactly the current category; 100% of Ctrl+Shift+R presses exactly everything; fixed items unchanged in all cases.
- **SC-004**: The three bindings appear in the hints and help, enforced by the extraction tests.
- **SC-005**: The scope behavior (item, category, everything; fixed preserved; idempotent) is covered by tests.

## Assumptions

- The binding scheme (R / Ctrl+R / Ctrl+Shift+R) is the user's chosen replacement for the requested space-chords, which terminals cannot report (Space is not a reportable key modifier).
- "The current item's category" is the category of the item under the cursor; on a category header, it is that header's category (EC-001).
- The auto/fixed mode distinction (spec 015) is unchanged: recalculate touches automatic items only.
- Spec 021's add-dialog quantity/mode semantics compose with this spec: the add dialog's values are the new item's starting values, and no recalculation follows the add (EC-004).
- The trip file's persistence behavior is unchanged; only the recalculation triggers change.
- The hint lines reflow to fit the new entries at the supported minimum size (the existing hint system's behavior).
