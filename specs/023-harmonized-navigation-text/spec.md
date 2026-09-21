# Feature Specification: Harmonized Navigation Text

**Feature Branch**: `023-harmonized-navigation-text`

**Created**: 2026-09-21

**Status**: Draft

**Input**: User description: "Harmonize the navigation text at the bottom of the screens, only specify everything once"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One binding, one wording, everywhere (Priority: P1)

A user reads the navigation hint line at the bottom of any screen and sees the same key described with the same word on every screen where it appears. Today the same binding is explained with different wording per surface (for example the vertical arrows are "Navigate" on the main panels but the horizontal arrows' wording differs between the welcome screen and the add/edit dialog, and Tab is described with surface-specific phrasing). After this feature, each key has exactly one canonical action text; a surface's hint line lists only the bindings that apply there, but every listed binding uses the canonical wording. The help screen agrees with every hint line.

**Why this priority**: The user's core complaint is that the same thing is specified multiple times with different words. One canonical wording per key is the essence of the feature; everything else builds on it.

**Independent Test**: For every key that appears on more than one surface, collect its action text from each surface's hint line and from the help screen — all occurrences are identical.

**Acceptance Scenarios**:

1. **Given** a key that appears on two or more surfaces (e.g. the vertical arrows), **When** its action text is collected from each surface's hint line, **Then** every occurrence uses the same wording.
2. **Given** the help screen, **When** a binding's description is compared with the corresponding hint line entry, **Then** the key and its meaning agree (the help screen may be more verbose, but never contradicts the hint wording).
3. **Given** any surface's hint line, **When** it is rendered, **Then** it contains no binding that the surface does not actually respond to, and it contains every binding the surface does respond to (the surface's complete binding set).

---

### User Story 2 - Each surface states its bindings once (Priority: P1)

A user moves between screens and never sees the same explanation repeated within one hint line, and never sees a binding explained twice on the same screen. Each surface's hint line is the single place where that surface's bindings are stated; shared bindings (quit, help, config) appear once per surface, not duplicated across panels or footers on the same screen. Where two panels on the same screen share a binding (e.g. both main panels support the recalculate keys), the shared entries are stated once, not once per panel.

**Why this priority**: "Only specify everything once" is the second half of the user's request — the duplication itself must go, not just the wording drift.

**Independent Test**: On each screen, no (key, action) pair appears more than once across all hint lines visible on that screen.

**Acceptance Scenarios**:

1. **Given** the main screen with both panels visible, **When** their hint lines are inspected, **Then** no (key, action) pair appears in both panels' lines (shared bindings are stated once).
2. **Given** any single hint line, **When** it is inspected, **Then** no key appears twice with the same or overlapping meaning.
3. **Given** a screen with a footer hint and a popup hint (e.g. the config screen with the modifier editor open), **When** both are visible, **Then** the popup's line lists only the popup-specific bindings; bindings shared with the underlying screen are not restated.

---

### User Story 3 - The wording stays harmonized (Priority: P2)

A developer changes a binding's behavior or adds a new binding, and the hint text and help text cannot drift apart again: the canonical action text for each key is defined in exactly one place, and every surface's hint line and the help screen derive from that single definition. A wording change is a one-line change that propagates everywhere.

**Why this priority**: This protects the feature's value after delivery — without it, the harmonization decays with the next change.

**Independent Test**: The canonical action text for each key exists in exactly one definition; all hint lines and the help screen reference it. A test fails if a surface's rendered hint line contains an action text that differs from the canonical definition for that key.

**Acceptance Scenarios**:

1. **Given** the canonical definition of a key's action text, **When** any surface's hint line is rendered, **Then** the rendered text equals the canonical definition.
2. **Given** a new binding is added to a surface, **When** its hint entry is written, **Then** the canonical wording is reused (or a new canonical entry is created), and no surface-specific paraphrase is introduced.

---

### Edge Cases

- What happens when a key means different things on different surfaces (e.g. Space selects on the left panel but marks packed on the right panel)?
  **Answer**: The canonical wording describes the key's meaning at the level both share ("Space" = the primary action of the focused panel); where the meanings genuinely differ, each surface keeps its own canonical entry, but each distinct meaning still has exactly one wording used on every surface where that meaning applies.
- What happens when a surface's complete binding set does not fit the available width?
  **Answer**: The existing adaptive packing and collapse behavior (packing entries into rows that fit, collapsing to the short help pointer when tight) continues to apply; harmonization changes the wording, not the layout rules.
- What happens to the help screen, which is intentionally more verbose than a hint line?
  **Answer**: The help screen keeps its longer descriptions but must not contradict the hint wording; the key tokens and the canonical action words agree.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every key that appears in more than one surface's navigation hint MUST have exactly one canonical action text used verbatim on every surface where the key appears.
- **FR-002**: Each surface's hint line MUST state its complete binding set — every binding the surface responds to, and no binding it does not respond to.
- **FR-003**: No (key, action) pair MAY appear more than once within a single hint line, and no key MAY be restated across the hint lines visible on the same screen.
- **FR-004**: The help screen MUST agree with the hint lines: the same key tokens, and descriptions that do not contradict the canonical action text.
- **FR-005**: The canonical action text for each key MUST be defined in exactly one place; all hint lines and the help screen MUST derive from that definition.
- **FR-006**: The harmonization MUST NOT change any key's actual behavior — only the displayed text changes.
- **FR-007**: The existing adaptive hint layout (row packing to fit the width, collapse to the short help pointer when space is tight) MUST continue to work unchanged.

### Key Entities

- **Canonical binding**: A (key, action text) pair defined once; the single source that every surface's hint line and the help screen reference.
- **Surface binding set**: The complete list of bindings a screen or popup responds to; the hint line for a surface renders exactly this set.
- **Hint line**: The navigation explanation rendered at the bottom of a screen or popup, composed of canonical bindings.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For every key appearing on two or more surfaces, all occurrences of its action text across all hint lines are identical (zero wording variants).
- **SC-002**: Zero (key, action) pairs appear more than once within any single hint line or across the hint lines visible on the same screen.
- **SC-003**: Every surface's hint line matches its actual binding set exactly (no missing, no extra bindings).
- **SC-004**: A single change to a canonical action text propagates to every surface that shows that key, with no other edits required.
- **SC-005**: All existing behavior is unchanged: the full test suite passes and manual navigation on every screen behaves identically to before.

## Assumptions

- The set of surfaces is the existing one: the two main panels, the welcome screen, the config screen, the add/edit dialog, the delete/edit confirmations, the modifier editor, and the raw editor.
- Keys whose meanings genuinely differ per surface (e.g. Space on the left vs. right panel) keep per-meaning canonical entries; the feature harmonizes wording, not semantics.
- The help screen remains the verbose reference; only contradiction with hint wording is in scope, not its level of detail.
- The existing hint layout rules (row packing, collapse) are out of scope for change; only the entry wording and duplication are addressed.
- No key bindings are added, removed, or reassigned by this feature.
