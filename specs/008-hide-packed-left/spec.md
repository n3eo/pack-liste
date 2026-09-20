# Feature Specification: No Packed State on the Left Panel

**Feature Branch**: `008-hide-packed-left`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description: "do not show on the left side if a element was packed"

> **Re-invoked 2026-09-20** (three phrasings, one intent):
> 1. "do not show on the left side if a element was packed"
> 2. "Do not show the already packed marker on the left split"
> 3. "don't show on the left side if a item was packed or not"
>
> **Re-invoked again 2026-09-20** (fourth phrasing, with a screenshot):
> 4. "remove the second ○ on the left side showing if it was packed"
>
> All four target the same thing: the left panel's per-item packed
> indicator (the `✓`/`○` symbol and its green/red coloring at
> src/ui/app.rs:3905–3918). The third phrasing ("packed or not") makes
> explicit that the *entire* packed-state column goes — not just the
> packed case. The fourth phrasing confirms it against the current
> rendering: the left panel shows two symbols per item ("✓ ○ T-Shirt
> x5" — selection, then packed); the second symbol is the packed
> indicator this spec removes. The spec's original model (hide packed
> items entirely) is **superseded by the marker-removal model**: items
> stay visible on the left panel regardless of packed state; only the
> indicator is removed. The user's own words control: "if a item was
> packed or not" — the left panel no longer shows which it was.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The left panel shows no packed state (Priority: P1)

A user works through their packing list. The left panel (the all-items tree) shows each item's name, quantity, and selection state — but nothing about its packed state. Whether an item is packed or not is simply not shown on the left; the compiled list on the right remains the single place where packed state is visible and managed. The user's own words define the outcome: the left side does not show "if a item was packed or not".

**Why this priority**: This is the entire request, stated three times with increasing clarity. The left panel's job is selecting what goes into the trip; packed state belongs to the working list on the right, and showing it on the left duplicates state where it is not managed.

**Independent Test**: Pack and unpack items on the right panel; verify the left panel's rendering of every item is identical in both cases — no symbol, color, or any other visual difference tracks packed state.

**Acceptance Scenarios**:

1. **Given** an item is unpacked, **When** the user views the left panel, **Then** the item row shows its selection symbol, name, and quantity, and no packed indicator.
2. **Given** an item is packed, **When** the user views the left panel, **Then** the item row is rendered exactly as the unpacked case — same symbols, same colors, same layout; nothing distinguishes it.
3. **Given** the user toggles an item's packed state on the right panel, **When** the left panel redraws, **Then** the item's left-panel rendering is unchanged by the toggle.

---

### User Story 2 - The right panel keeps the full packed state (Priority: P1)

The compiled list on the right continues to show every item with its packed indicator (`✓`/`○`), the category packing indicators, and the progress summary — exactly as today. Packed state remains visible, toggleable, and persisted there; removing the left panel's indicator removes no functionality, only the duplicate display.

**Why this priority**: The feature must not lose the packed-state feature itself; the right panel is where it lives and stays.

**Independent Test**: Toggle packed states on the right panel; verify the indicators, category markers, and progress summary update exactly as before this feature.

**Acceptance Scenarios**:

1. **Given** any trip state, **When** the user views the right panel, **Then** each item shows its packed indicator as before.
2. **Given** the user toggles packed on the right panel, **When** the list redraws, **Then** the indicator, category markers, and progress summary update as before.
3. **Given** the user exits and resumes, **When** the trip reloads, **Then** packed states are restored and shown on the right panel as before.

---

### User Story 3 - Left-panel interactions are unaffected (Priority: P2)

Everything the left panel does keeps working identically: navigation visits every item row (packed or not), selection toggles apply to all items, category collapse/expand and category selection behave as before, and the cursor never jumps because no rows are hidden. Removing the indicator is purely visual.

**Why this priority**: The change must be invisible to the left panel's behavior; any navigation or selection regression would break the primary workflow.

**Independent Test**: Navigate the left panel end to end with a mix of packed and unpacked items; toggle selections and category toggles; verify behavior is identical to before the feature.

**Acceptance Scenarios**:

1. **Given** a mix of packed and unpacked items, **When** the user navigates the left panel, **Then** the cursor visits every item row in the same order as before.
2. **Given** packed items exist, **When** the user toggles a category's selection on the left, **Then** all its items — packed and unpacked — toggle together as before.
3. **Given** packed items exist, **When** the user collapses/expands a category, **Then** the behavior is unchanged (no rows are hidden by packed state).

---

### User Story 4 - The help screen and hints stay accurate (Priority: P3)

The documentation of the two panels matches the new reality: the left panel is described as selection-only (no packed state), and packed state is documented under the right panel only. No hint or help text suggests the left panel shows packed state.

**Why this priority**: The help screen is the in-app documentation; it must not describe a display that no longer exists.

**Independent Test**: Read the help screen and every on-panel hint; verify none attributes packed-state display to the left panel.

**Acceptance Scenarios**:

1. **Given** the help screen, **When** the user reads the panel descriptions, **Then** packed state is attributed to the right panel only.
2. **Given** the left panel's hints, **When** read, **Then** they describe selection and navigation only.

### Edge Cases

- Does the selection symbol (✓/○) on the left panel stay?
  **Answer**: Yes — the selection symbol is the left panel's own state (what goes into the trip) and is unaffected. Only the packed indicator (the second symbol column with its green/red coloring) is removed.
  **Acceptance Criteria**: The left panel still shows the selection symbol, name, and quantity per item.
- Does the row layout change when the indicator is removed?
  **Answer**: The freed space is not repurposed; the row simply loses the indicator column. Name and quantity positions shift left accordingly; no other layout change.
  **Acceptance Criteria**: The left panel's rows show selection symbol, name, and quantity with no gap where the indicator was.
- Does the left panel's Space key still toggle packed state?
  **Answer**: No — and it never did; Space toggles selection on the left and packed on the right (existing binding, unchanged). With no packed display on the left, the left panel's Space remains selection-only, consistent with what it shows.
  **Acceptance Criteria**: Space on the left toggles selection exactly as before.
- Does removing the indicator affect the category packing indicators on the right panel?
  **Answer**: No — the right panel's per-item indicators and category-header markers (spec 015) are untouched.
  **Acceptance Criteria**: The right panel renders identically to before this feature.
- Does the trip file or any persisted data change?
  **Answer**: No — the change is presentation-only on the left panel; packed state is still persisted via the trip file exactly as before.
  **Acceptance Criteria**: Trip files saved with this feature are identical in content to those saved without it.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The left panel MUST NOT display any per-item packed indicator — no symbol, color, or other visual distinction based on packed state.
- **FR-002**: The left panel MUST continue to show every item (packed and unpacked) with its selection symbol, name, and quantity.
- **FR-003**: The right panel MUST keep its full packed-state display: per-item indicators, category packing markers, and the progress summary, unchanged.
- **FR-004**: Left-panel navigation, selection toggles, category selection, and collapse/expand MUST behave exactly as before (no rows are hidden by packed state).
- **FR-005**: The change MUST be presentation-only: selection state, packed state, trip file contents, and quantity calculations MUST be unaffected.
- **FR-006**: The help screen and on-panel hints MUST NOT attribute packed-state display to the left panel.

### Key Entities *(include if feature involves data)*

- **Left panel (all-items tree)**: The selection view of the item catalog grouped by category; with this feature, it displays selection state only — packed state is not shown.
- **Packed state**: Per-item boolean, displayed, toggled, and persisted exclusively via the right panel (compiled list) and the trip file.
- **Packed indicator (removed)**: The former `✓`/`○` symbol column on the left panel; superseded by this feature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of left-panel item rows render identically for packed and unpacked items — zero visual packed-state distinctions.
- **SC-002**: 100% of items remain visible on the left panel regardless of packed state.
- **SC-003**: The right panel's packed-state display (indicators, category markers, progress) is unchanged, verified against pre-feature behavior.
- **SC-004**: Trip files saved with this feature are identical in content to those saved without it (no data change).
- **SC-005**: All existing left-panel behaviors (navigation, selection, category toggles, collapse) pass their existing tests unchanged.

## Assumptions

- Packed state is toggled on the right panel (the compiled list); the left panel's packed indicator was a duplicate display of state managed elsewhere and is removed.
- The right panel always shows the full compiled list, packed and unpacked, as the single source of truth for packed state.
- The removal is automatic and always active on the left panel; no toggle or setting is added.
- The selection symbol on the left panel is separate state (trip membership) and stays.
- Existing key bindings and screens are unchanged; only what the left panel renders changes.
