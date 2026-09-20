# Feature Specification: Add Dialog Quantity and Mode

**Feature Branch**: `021-add-dialog-quantity-mode`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"when adding a field support changing the default quantity and automatic calc mode"

> **Scope note**: The add-item dialog and the edit-item dialog share
> their field layout, but not their field set: the edit dialog has a
> Quantity field and a Mode field (automatic/fixed, added by spec
> 015), while the add dialog has only Name and Category. A new item is
> always created as automatic with its quantity taken from the typed
> value (falling back to 1) — the user cannot set a fixed quantity at
> creation time and cannot choose the calculation mode. This feature
> gives the add dialog the same two fields: a Quantity field whose
> value becomes the item's starting quantity, and a Mode field
> (automatic/fixed) that decides whether that quantity is a fixed
> value or a starting point for the automatic calculation. The two
> dialogs end up with identical fields and identical bindings.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The add dialog has a Quantity field (Priority: P1)

The user opens the add dialog and sees the Quantity field between Category and Mode, exactly as in the edit dialog. They can type a quantity (and, once spec 020 lands, step it with Left/Right); the typed value becomes the new item's quantity. An empty quantity keeps the existing default of 1. The field behaves identically to the edit dialog's Quantity field.

**Why this priority**: This is half the request — today the typed quantity is silently reinterpreted (an automatic item's quantity is recalculated away on the next parameter change), so the user cannot reliably set what they typed.

**Independent Test**: Add an item with quantity 5, save, and verify the item shows 5; add another with an empty quantity and verify it shows the calculated/default value.

**Acceptance Scenarios**:

1. **Given** the add dialog, **When** it opens, **Then** it shows the fields Name, Category, Quantity, and Mode in the edit dialog's order.
2. **Given** the add dialog with quantity 5 typed, **When** the user saves, **Then** the new item's quantity is 5.
3. **Given** the add dialog with an empty quantity, **When** the user saves, **Then** the new item's quantity is 1 (the existing default).

---

### User Story 2 - The add dialog has a Mode field (Priority: P1)

The add dialog's Mode field shows the calculation mode for the new item — automatic or fixed — and Space toggles it, exactly as in the edit dialog. The mode decides what the typed quantity means: fixed means the item keeps the typed quantity forever (never recalculated); automatic means the typed quantity is the starting value and the trip-parameter calculation governs from the next recalculation on. The default mode for a new item is automatic (the existing behavior).

**Why this priority**: This is the other half of the request — without the Mode field, a user who wants a fixed-quantity item (a toothbrush that stays at 1) must add it as automatic and then edit it a second time to fix its quantity.

**Independent Test**: Add an item with mode fixed and quantity 3, save, change trip parameters, and verify the quantity stays 3; add another with mode automatic and quantity 3, change parameters, and verify it recalculates.

**Acceptance Scenarios**:

1. **Given** the add dialog, **When** it opens, **Then** the Mode field shows "automatic" (the default).
2. **Given** the add dialog with mode toggled to fixed and quantity 3, **When** the user saves, **Then** the new item is fixed with quantity 3 and no later recalculation changes it.
3. **Given** the add dialog with mode automatic and quantity 3, **When** the user saves and trip parameters change, **Then** the item's quantity recalculates (the typed 3 was the starting value only).
4. **Given** the add dialog's Mode field focused, **When** the user presses Space, **Then** the mode toggles (existing edit-dialog behavior).

---

### User Story 3 - Both dialogs share one field model (Priority: P2)

The add and edit dialogs have the same four fields, the same focus order, the same bindings, and the same hint line. A user who knows one dialog knows the other; the only differences are the title and what saving does (create vs. update). The dialog's hint and the help screen document the shared field set once.

**Why this priority**: The request is about parity ("when adding... support changing"); parity is only real if the two dialogs do not drift apart again.

**Independent Test**: Open both dialogs; verify the fields, their order, the focus cycling, and the hint line are identical (modulo title and pre-filled values).

**Acceptance Scenarios**:

1. **Given** the add dialog and the edit dialog, **When** their field sets are compared, **Then** both show Name, Category, Quantity, Mode in the same order.
2. **Given** either dialog, **When** the user cycles fields, **Then** the cycle and bindings are the same.
3. **Given** the help screen's dialog section, **When** read, **Then** it documents the field set once, applying to both dialogs.

---

### Edge Cases

- **EC-001**: What does the typed quantity mean when the mode is automatic?
  **Answer**: It is the item's starting quantity; the next recalculation (parameter change, explicit recalculate) replaces it with the calculated value. This matches the edit dialog's semantics for automatic items.
  **Acceptance Criteria**: An automatic item added with quantity 5 recalculates on the next parameter change.

- **EC-002**: What does the typed quantity mean when the mode is fixed?
  **Answer**: It is the item's permanent quantity, never recalculated. An empty quantity with fixed mode yields 1 (the existing minimum).
  **Acceptance Criteria**: A fixed item added with quantity 3 keeps 3 through all recalculation triggers.

- **EC-003**: Does the new item appear selected with the right quantity in the list?
  **Answer**: Yes — the existing post-add behavior (selection moves to the new item, quantities update, config saves) is unchanged; only the item's initial quantity and mode come from the dialog.
  **Acceptance Criteria**: After adding, the list shows the new item with its dialog quantity/mode and the cursor on it.

- **EC-004**: Is the Mode field's Space toggle the same control as the edit dialog's?
  **Answer**: Yes — one Mode field concept, one toggle binding; the add dialog starts at automatic, the edit dialog starts at the item's current mode.
  **Acceptance Criteria**: Space in either dialog's Mode field toggles the mode.

- **EC-005**: What if the user adds an item while spec 020 (dialog navigation) has not landed?
  **Answer**: The fields work with the bindings that exist at implementation time (Tab cycling, typed quantity); the field set is independent of the navigation model. The two specs compose without conflict.
  **Acceptance Criteria**: The Quantity and Mode fields are usable under the current bindings.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The add dialog MUST offer the same four fields as the edit dialog — Name, Category, Quantity, Mode — in the same order.
- **FR-002**: The typed quantity MUST become the new item's quantity; an empty quantity MUST default to 1.
- **FR-003**: The Mode field MUST set the new item's calculation mode: fixed items keep their quantity permanently; automatic items have their typed quantity as the starting value and recalculate thereafter. The default MUST be automatic.
- **FR-004**: Space MUST toggle the Mode field, identical to the edit dialog's Mode field.
- **FR-005**: The add dialog's field bindings MUST match the edit dialog's (shared field model); the hint line and help screen MUST document the shared set.
- **FR-006**: The existing post-add behavior (selection on the new item, quantity update, config save) MUST be unchanged.

### Key Entities

- **Add dialog**: The item-creation dialog; gains the Quantity and Mode fields, ending with the same field model as the edit dialog.
- **Starting quantity**: The typed quantity's role for an automatic item — the value until the next recalculation.
- **Calculation mode**: The existing automatic/fixed distinction (spec 015's Mode field), now settable at creation time.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of items added through the TUI are created with the dialog's quantity and mode (empty quantity → 1, default mode → automatic).
- **SC-002**: 100% of fixed items added through the dialog keep their quantity through all recalculation triggers.
- **SC-003**: The add and edit dialogs' field sets, order, and bindings are identical.
- **SC-004**: The dialog's hints and help document the shared field set, enforced by the extraction tests.
- **SC-005**: All existing add/edit behavior passes its tests unchanged.

## Assumptions

- The Mode field reuses the edit dialog's existing mode concept and toggle binding (spec 015); no new mode semantics are introduced.
- The typed quantity's meaning follows the existing auto/fixed semantics (EC-001/EC-002); no new quantity semantics are introduced.
- The add dialog's defaults (empty name → "New Item", empty category → the current category, empty quantity → 1) are unchanged except that the quantity and mode are now visible and editable.
- Spec 020 (dialog navigation) and this spec compose: the field set is independent of the navigation bindings in effect.
- The dialog's hint line reflows to fit the updated entries at the supported minimum size (the existing hint system's behavior).
