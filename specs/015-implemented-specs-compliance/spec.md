# Feature Specification: Implemented-Specs Compliance Review

**Feature Branch**: `015-implemented-specs-compliance`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"Critically review all already implemented user stories and features for compliance against the current product; identify gaps and create new user stories and tasks to implement"

> **Review note (2026-09-20)**: All five implemented specs (001–005) were
> verified claim-by-claim against the current code. Most claims are
> compliant. This spec captures the gaps found: requirements that were
> aligned away but never re-planned as work, silent-fallback behavior that
> contradicts its own spec, and product-quality deficits (untested input
> handling, incomplete help). Each gap becomes a user story below; the
> tasks to implement them are generated from this spec.

## Review Results

| # | Spec | Claim | Verdict |
|---|------|-------|---------|
| 1 | 001 FR-008 | Add/edit/delete items via TUI | Compliant |
| 2 | 001 SC-003 | Real-time list updates | Compliant |
| 3 | 001 input pt. 3 | Items can be in multiple categories | **GAP** — one category per item |
| 4 | 001 FR-006 | All selected by default on new list | Compliant |
| 5 | 002 FR-001 | Collapse/expand with cursor to header | Compliant |
| 6 | 002 FR-004 | Modifier screen from any screen | Compliant |
| 7 | 002 SC-007 | Modifiers persist | Compliant |
| 8 | 003 FR-002 | Fixed-quantity items, set once, editable | **PARTIAL** — no TUI toggle; config-file only |
| 9 | 003 input pt. 4 | Category marked if not all items packed | **GAP** — aligned away, never re-planned |
| 10 | 003 FR-006 | Progress summary | Compliant |
| 11 | 004 FR-003 | Remembered trip file, resume prompt | Compliant |
| 12 | 004 FR-007 | Path resolution (CWD, ~) | Compliant |
| 13 | 004 reconcile | Drop unknown, match categories | Compliant |
| 14 | 005 FR-005 | Real-time quantity updates | Compliant |
| 15 | 005 FR-007 | Trip length 1–30, base cap 20 | Compliant |
| 16 | 005 FR-010 | Unknown modifiers fall back to 1.0 | **PARTIAL** — unknown activities silently dropped |
| 17 | — | Input-handling test coverage | **GAP** — zero tests for key handling |
| 18 | — | Help screen completeness | **PARTIAL** — 7+ bindings missing |
| 19 | — | Welcome screen shows current values | Compliant |
| 20 | — | No TODO/FIXME/unimplemented | Compliant |

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Toggle Auto/Fixed Quantity in the TUI (Priority: P1)

The user marks an item as fixed-quantity (or back to automatic) directly in the TUI, in the existing item edit dialog. Today the auto/fixed flag can only be changed by hand-editing the configuration file; the edit dialog edits the quantity but never the mode, and a quantity edited on an automatic item is silently overwritten by the next recalculation. With this story, the mode is a first-class field of the edit dialog: toggling to fixed keeps the current quantity as the fixed value; toggling back to automatic lets the calculation take over again.

**Why this priority**: Spec 003 FR-002 promises fixed-quantity items "set once, editable" — the behavior exists but is unreachable from the TUI, and the silent overwrite of edited auto quantities is a data-loss-like surprise. This is the largest functional gap in the shipped product.

**Independent Test**: Open the edit dialog for an automatic item, toggle it to fixed, save, change trip parameters, and verify the quantity stays; toggle back to automatic, change parameters, and verify it recalculates.

**Acceptance Scenarios**:

1. **Given** the edit dialog of an automatic item, **When** the user toggles the quantity mode to fixed and saves, **Then** the item's current quantity becomes its fixed value and survives all trip-parameter changes.
2. **Given** the edit dialog of a fixed item, **When** the user toggles the mode back to automatic and saves, **Then** the quantity recalculates from the trip parameters immediately.
3. **Given** an automatic item, **When** the user edits only its quantity in the dialog, **Then** the mode toggle shows "automatic" and the edit does not silently stick (the recalculation rule for auto items remains authoritative).

---

### User Story 2 - Category Packing Indicator (Priority: P1)

Each category header on the compiled list shows whether all of its items are packed: a distinct marker when every item of the category is packed, a different marker when some are unpacked. This restores the original spec 003 input point 4 ("each category also is marked visually if not all items are packed"), which was aligned away during the spec review but never re-planned as work.

**Why this priority**: It was an explicit original requirement, dropped silently. During packing, the category-level signal is the at-a-glance answer to "which areas still have work" — the per-item symbols require scanning every row.

**Independent Test**: Pack all items of one category and leave another partially unpacked; verify the headers show the all-packed and partially-packed markers respectively, and that the markers update immediately on every toggle.

**Acceptance Scenarios**:

1. **Given** a category with all items packed, **When** the compiled list is viewed, **Then** its header carries the all-packed marker.
2. **Given** a category with at least one unpacked item, **When** the compiled list is viewed, **Then** its header carries the partially-packed marker.
3. **Given** the user toggles the last unpacked item of a category, **When** the list redraws, **Then** the category header switches to the all-packed marker immediately.

---

### User Story 3 - Unknown Modifier Fallback Is Uniform (Priority: P2)

Unknown modifier names behave the same everywhere: a selected but unknown travel style, weather, or activity falls back to a neutral multiplier of 1.0, per spec 005 FR-010. Today unknown activities are silently dropped from the multiplier list, which changes the calculation's structure (an empty activity list hands control back to the weather multiplier) instead of applying the specified neutral fallback.

**Why this priority**: The current behavior contradicts the implemented spec's own fallback rule and is invisible to the user — the same input produces different results depending on which modifier kind is unknown. Uniformity makes the calculation predictable and the spec honest.

**Independent Test**: Select an activity that is not in the configured modifier list; verify the calculation treats it as multiplier 1.0 (the activity path stays active) rather than falling back to the weather multiplier.

**Acceptance Scenarios**:

1. **Given** a selected activity with no configured multiplier, **When** quantities are calculated, **Then** it contributes 1.0 to the activity average (the activity path remains in effect).
2. **Given** an unknown travel style or weather, **When** quantities are calculated, **Then** the multiplier is 1.0 (existing behavior, unchanged).
3. **Given** a mix of known and unknown selected activities, **When** quantities are calculated, **Then** the average includes the unknown ones at 1.0.

---

### User Story 4 - Help Screen Lists Every Binding (Priority: P2)

The help screen is complete and accurate: every key binding the application responds to is listed, and nothing listed is stale. Today at least seven bindings are missing (quit via Ctrl+C, trip-file edit via F, BackTab reverse cycling, Up/Down on the config screen, Home/End in the raw editor, the modifier editor's keys, Esc's screen-level roles) and one entry is duplicated.

**Why this priority**: The help screen is the only in-app documentation of the key-driven interface; missing bindings are undiscoverable, and the application's own constitution requires documented behavior.

**Independent Test**: Extract the binding list from the help screen and compare it against the actual key handlers; verify a one-to-one match with no missing or stale entries.

**Acceptance Scenarios**:

1. **Given** the help screen, **When** the user reads it, **Then** every active binding is listed, grouped by screen or context.
2. **Given** the help screen, **When** a binding is added or removed in the code, **Then** the help screen is updated in the same change (verified by the extraction test).
3. **Given** the help screen content, **When** checked for duplicates, **Then** each binding appears exactly once.

---

### User Story 5 - Input Handling Is Under Test (Priority: P2)

The application's key handling — screen routing, input modes, navigation, collapse/cursor logic, the flows of the other user stories — is covered by automated tests. Today the 11 existing tests cover only serialization, reconciliation, path resolution, and the quantity calculation; zero tests exercise any key handler, so regressions in the most-touched code are invisible to the suite.

**Why this priority**: The constitution requires tested features, and every user story in this spec changes key handling; without an input-level test net, those changes cannot be made safely. This story is the net the other stories land on.

**Independent Test**: Run the test suite; verify it contains tests that drive the input handlers (key events in, state out) and that they fail when a binding is broken.

**Acceptance Scenarios**:

1. **Given** the test suite, **When** a navigation binding is broken (e.g., Up no longer moves the cursor), **Then** at least one test fails.
2. **Given** the test suite, **When** the flows of user stories 1–4 are exercised, **Then** each has at least one passing test driving it through key events.
3. **Given** any future key-handling change, **When** the suite runs, **Then** the input-level tests provide the regression net.

---

### User Story 6 - Multi-Category Items (Priority: P3)

An item can belong to more than one category, restoring spec 001's input point 3. The item appears under each of its categories in the tree, and selecting or deselecting any of its category occurrences selects or deselects the item as a whole (one underlying item, shown in multiple places). The edit dialog allows managing the item's category memberships.

**Why this priority**: It is the remaining unfulfilled original input point, but it restructures the data model (single category field today) and the selection semantics; the value is real but the cost is the highest of the gaps, and the single-category model covers the primary workflow.

**Independent Test**: Give an item two categories; verify it appears under both in the tree, that deselecting it under one category deselects it everywhere, and that the compiled list shows it once.

**Acceptance Scenarios**:

1. **Given** an item with two categories, **When** the tree is viewed, **Then** the item appears under both category headers.
2. **Given** an item shown under two categories, **When** the user deselects it under one category, **Then** the item is deselected (it disappears from the compiled list and is shown deselected under both headers).
3. **Given** the edit dialog, **When** the user changes the item's categories, **Then** the tree reflects the new memberships after saving.

### Edge Cases

- **EC-001**: What marker pair is used for the category packing indicator?
  **Answer**: A check-type symbol for all-packed and an open-type symbol for partially packed, in the same color language as the per-item state symbols (green for complete, red/neutral for incomplete), consistent with the existing symbol vocabulary.
  **Acceptance Criteria**: The two states are visually distinct from each other and from the collapse arrow.

- **EC-002**: Does the category indicator appear on the left (all-items) panel too?
  **Answer**: No — packed state is a property of the compiled trip; the indicator appears on the compiled list's category headers only. (Spec 008, if implemented later, hides packed items from the left panel entirely.)
  **Acceptance Criteria**: The left panel's headers are unchanged.

- **EC-003**: When toggling auto→fixed, what becomes the fixed value?
  **Answer**: The item's current calculated quantity at the moment of the toggle; the user can then edit it like any fixed quantity.
  **Acceptance Criteria**: After toggling to fixed, the quantity equals the pre-toggle calculated value until edited.

- **EC-004**: Can the mode toggle produce an invalid state (fixed with quantity 0)?
  **Answer**: No — fixed quantities keep the existing minimum-1 rule; the toggle itself never changes the quantity, and subsequent edits validate as today.
  **Acceptance Criteria**: No fixed item can hold a quantity below 1.

- **EC-005**: For unknown activities at 1.0, does the calculation note still say activities take priority?
  **Answer**: Yes — the activity path is genuinely in effect (average including the 1.0 entries); the note's wording stays truthful.
  **Acceptance Criteria**: With unknown activities selected, the note reflects the activity path, not the weather path.

- **EC-006**: Does multi-category selection interact with category-level selection toggles?
  **Answer**: Yes — toggling a category's selection applies to all items shown under it, which for a multi-category item means the item as a whole; the item's other occurrences reflect the same state.
  **Acceptance Criteria**: Category toggles and item toggles produce one consistent selection state per item.

- **EC-007**: How are input-level tests structured without a live terminal?
  **Answer**: The key handlers are exercised against the application state directly — construct the state, dispatch key events through the handler, assert the resulting state — without spawning the terminal backend. Tests that need the full loop use the existing pty-driven approach where practical.
  **Acceptance Criteria**: The input tests run in the normal test suite without a TTY.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The item edit dialog MUST offer a quantity-mode toggle (automatic/fixed); toggling to fixed MUST keep the current quantity as the fixed value, and toggling to automatic MUST recalculate immediately.
- **FR-002**: The compiled list's category headers MUST indicate whether all of the category's items are packed (all-packed marker) or not (partially-packed marker), updating immediately on every packed toggle.
- **FR-003**: Unknown modifier names MUST fall back to a neutral multiplier of 1.0 uniformly for travel styles, weather, and activities; unknown activities MUST NOT be silently dropped from the activity average.
- **FR-004**: The help screen MUST list every active key binding, grouped by context, with no missing, stale, or duplicated entries.
- **FR-005**: The key-handling flows of this feature's user stories MUST be covered by automated tests driving key events against the application state.
- **FR-006**: An item MUST be able to belong to multiple categories, appearing under each in the tree, with one shared selection and packed state across all occurrences.
- **FR-007**: The edit dialog MUST allow managing an item's category memberships for multi-category items.

### Key Entities *(include if feature involves data)*

- **Quantity mode**: An item's automatic/fixed flag, now editable in the TUI; fixed keeps a user-set quantity, automatic recalculates.
- **Category packing state**: Derived per category on the compiled list — all-packed vs. partially-packed — from its items' packed states; presentation-only.
- **Modifier fallback**: The uniform 1.0 multiplier applied to any selected but unknown modifier name.
- **Category membership**: The set of categories an item belongs to; one item, many memberships, one selection/packed state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of items can be toggled between automatic and fixed quantity from the TUI without touching the configuration file.
- **SC-002**: 100% of compiled-list categories show a correct packing indicator that updates within one render of any packed toggle.
- **SC-003**: Unknown modifiers of all three kinds produce identical, specified fallback behavior (1.0), verified by calculation tests.
- **SC-004**: The help screen matches the actual bindings 1:1 (verified by an extraction test with zero missing and zero stale entries).
- **SC-005**: The test suite fails when any covered key binding is broken, and covers the flows of user stories 1–4.
- **SC-006**: An item with multiple categories appears under each, with consistent selection state, verified by tests.

## Assumptions

- The category packing indicator uses the existing symbol/color vocabulary; no new color scheme is introduced.
- The quantity-mode toggle lives in the existing edit dialog (a fourth field or a dedicated key within the dialog), not in a new screen.
- The unknown-activity fix changes calculation behavior only for the unknown-name case; known-activity calculations are bit-identical.
- The help screen is extended in place; its layout must still fit the minimum supported terminal size (spec 006's constraints apply when both are implemented).
- Multi-category items keep one underlying identity: selection, packed state, quantity, and edit dialog operate on the item, not per occurrence.
- Input-level tests target the handler layer (state in, key event, state out); full-loop pty tests remain the exception, not the rule.
