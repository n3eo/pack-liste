# Feature Specification: Responsive Small Terminal UI

**Feature Branch**: `006-responsive-small-terminals`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description: "Make the UI more responsive and work better with smaller terminal sizes"

Refined 2026-09-20 with:
"show the navigation explanation on the list screen or for small sizes hide behind '?' shortcut"

> **Refinement note**: The main list screen currently shows its navigation
> explanation (key hints) only when there is spare vertical space, and
> silently drops it otherwise. The refinement makes this a defined
> behavior: the hints are shown on the list screen when space allows, and
> when the terminal is too small they collapse to a single short hint
> pointing at the existing `?` help shortcut, so the explanation is always
> reachable but never breaks the layout.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Main packing screen stays usable at reduced sizes (Priority: P1)

A user opens the packing list in a small terminal (for example a terminal multiplexer pane or a half-width window). The two lists — all items on one side, the selected trip items on the other — must both stay readable and navigable. When the terminal is too narrow for a comfortable side-by-side arrangement, the layout switches to a vertical stacked arrangement — the two lists arranged top and bottom, each using the full terminal width — so the user can still review items, select them, and mark them as packed without horizontal scrolling or cut-off text.

**Why this priority**: The main screen is where users spend nearly all their time. If it breaks at small sizes, the application is effectively unusable in the environments that motivated this feature.

**Independent Test**: Open the main screen at 80x24 and at a narrower size; verify both lists render completely, the cursor is always visible, and selection/packing actions work at every size.

**Acceptance Scenarios**:

1. **Given** the main screen is open in an 80-column terminal, **When** the user navigates both lists, **Then** every row, header, and hint is fully visible with no truncated or overlapping text.
2. **Given** the main screen is open in a terminal narrower than the side-by-side threshold, **When** the layout switches to the vertical stacked arrangement, **Then** both lists remain visible and navigable, each using the full terminal width.
3. **Given** a long item or category name, **When** the terminal is narrow, **Then** the name is handled gracefully (truncated with a visible indicator or wrapped) without breaking the row layout.

---

### User Story 2 - Every screen fits a small terminal (Priority: P2)

A user navigates through all screens — welcome/trip setup, help, configuration, and the raw config editor — in a small terminal. Each screen must fit within the available area: no cut-off titles, no overlapping blocks, no borders broken by long content (such as the packing list file path on the welcome screen).

**Why this priority**: Broken rendering on any screen blocks the task the user was trying to complete on that screen, even if the main screen works.

**Independent Test**: Visit every screen at 80x24 and at one smaller size; verify each renders completely and all its functions are reachable.

**Acceptance Scenarios**:

1. **Given** the welcome screen with a long file path configured, **When** the terminal is 80 columns wide or narrower, **Then** the path display and all other blocks fit without breaking the layout.
2. **Given** the help screen, **When** the terminal is small, **Then** the full help content is accessible (scrolling if needed) rather than cut off.
3. **Given** the configuration and raw editor screens, **When** the terminal is small, **Then** lists, line numbers, and hints remain readable and the cursor stays visible.

---

### User Story 3 - Popups and overlays fit the terminal (Priority: P3)

A user opens any popup — add item, edit item, delete confirmation, modifier editor, or the resume-previous-trip prompt — in a small terminal. Every popup must fit inside the terminal area, scale down when needed, and keep all its fields and actions reachable.

**Why this priority**: Popups gate core actions (adding, editing, deleting items). A popup that overflows the terminal makes those actions impossible at small sizes.

**Independent Test**: Trigger each popup at 80x24 and at a smaller size; verify each fits within the terminal and can be completed or cancelled.

**Acceptance Scenarios**:

1. **Given** the add-item popup, **When** the terminal is small, **Then** the popup fits inside the terminal with all three fields usable.
2. **Given** the delete confirmation or resume prompt, **When** the terminal is very small, **Then** the prompt remains fully visible and answerable.
3. **Given** a popup is open, **When** the terminal is resized smaller, **Then** the popup re-fits without losing the user's in-progress input.

---

### User Story 4 - Graceful behavior below the minimum size and on resize (Priority: P4)

A user shrinks the terminal below the smallest usable size (or resizes it while working). The application must not crash, corrupt its display, or lose data. Below the minimum it shows a clear, readable notice (for example "terminal too small") instead of a broken screen; when the terminal grows again, the previous screen returns intact with all state preserved.

**Why this priority**: Robustness at the extremes protects the user's work; it matters less than day-to-day usability but prevents the worst outcomes.

**Independent Test**: Resize the terminal across a range of sizes, including below the minimum, while lists are scrolled and a trip is in progress; verify no crash, no data loss, and correct re-rendering.

**Acceptance Scenarios**:

1. **Given** any screen, **When** the terminal is resized larger or smaller, **Then** the display adapts within a short time without restart and without losing selections, packed states, or scroll positions.
2. **Given** a terminal below the minimum usable size, **When** the user views the application, **Then** a readable notice appears instead of broken content, and no data is lost.
3. **Given** the terminal returns to a usable size, **When** the notice was showing, **Then** the previous screen reappears with its state intact.

### User Story 5 - Navigation hints adapt to the available space (Priority: P2)

A user works on the main list screen. The navigation explanation (which keys do what) is shown on the screen when there is room for it. When the terminal is too small to show the full explanation without squeezing the lists, the explanation is replaced by a single short hint that points to the existing help shortcut (`?`), which opens the full navigation explanation on demand. The user never loses access to the explanation, and the lists never lose space to it at small sizes.

**Why this priority**: The hints teach the key-driven interface; hiding them entirely at small sizes leaves a user stranded with no visible clue. A one-line pointer to the help screen keeps the explanation reachable at every size at minimal cost.

**Independent Test**: Open the main screen at a large size and verify the navigation explanation is visible; shrink the terminal and verify the explanation collapses to the short `?` hint while the lists keep their space; press `?` and verify the full explanation opens.

**Acceptance Scenarios**:

1. **Given** the main screen in a terminal with ample space, **When** the list is rendered, **Then** the navigation explanation is shown on the screen.
2. **Given** the main screen in a terminal too small for the full explanation, **When** the list is rendered, **Then** only the short hint (pointing to `?` for help) is shown, and the lists keep the space the full explanation would have taken.
3. **Given** the main screen at any size, **When** the user presses `?`, **Then** the full navigation explanation opens as the help screen, and closing it returns to the list screen unchanged.
4. **Given** the terminal is resized from small to large (or back) while on the main screen, **When** the next frame renders, **Then** the explanation switches between full and short form to match the new size.

### Edge Cases

- What happens when the terminal is resized while a popup is open or text is being typed?
- What happens at extreme sizes (for example 20 columns or 10 rows), including a zero-size terminal during tiling transitions?
- How are very long item names, category names, and file paths displayed at narrow widths?
- What happens to the cursor and scroll position when a list is deep-scrolled and the terminal shrinks (fewer visible rows)?
- What happens when the terminal grows during the "too small" notice — is state fully restored?
- At which size exactly does the navigation explanation switch to the short `?` hint, and is the switch stable (no flicker when the terminal sits exactly at the threshold)?
- Is the short hint shown when the list content itself needs all the rows (deep list in a mid-size terminal), or only when the terminal is small?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every screen MUST render without truncated, overlapping, or broken content at 80 columns by 24 rows.
- **FR-002**: The main screen MUST keep both item lists visible and navigable below the side-by-side width threshold by switching to a vertical stacked arrangement, with each list using the full terminal width.
- **FR-003**: All popups and overlays MUST fit within the current terminal area, scale down as needed, and keep every field and action reachable.
- **FR-004**: The application MUST handle live terminal resizing without restart, data loss, or display corruption, and re-render the active screen promptly.
- **FR-005**: Below the minimum usable size, the application MUST show a readable notice instead of broken content and MUST NOT lose any user data.
- **FR-006**: Long text values (item names, categories, file paths) MUST be displayed at narrow widths without breaking the layout, using truncation with a visible indicator or safe wrapping.
- **FR-007**: All keyboard functions MUST remain available at small sizes with the same key bindings as at large sizes.
- **FR-008**: The application MUST remain fully usable down to 80 columns by 24 rows; below that size, only the readable notice and data safety requirements (FR-005) apply.
- **FR-009**: The main list screen MUST show the navigation explanation when the terminal has room for it without squeezing the lists.
- **FR-010**: When the terminal is too small for the full navigation explanation, the main list screen MUST show a single short hint pointing to the `?` help shortcut instead, and MUST NOT let the explanation or hint crowd out the lists.
- **FR-011**: The `?` shortcut MUST open the full navigation explanation (help screen) from the main list screen at every supported size, and closing it MUST return to the list screen with its state unchanged.

### Key Entities *(include if feature involves data)*

- **Terminal viewport**: The current width and height available for rendering; changes at runtime and drives layout decisions.
- **Layout mode**: The arrangement currently in effect for the main screen — side-by-side, vertical stacked, or minimum-size notice — derived from the terminal viewport.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At 80x24, a user can complete the full core workflow — configure a trip, select items, mark items packed, and exit with the list saved — without any visual defect.
- **SC-002**: At 80x24, every screen is readable and every function is reachable; no screen requires horizontal scrolling to see its content.
- **SC-003**: Terminal resizes take effect on screen within 1 second, without restart or data loss.
- **SC-004**: 100% of keyboard functions available at large sizes remain available at the minimum size.
- **SC-005**: Below the minimum size, 100% of sessions show the readable notice (never broken content), and no session loses data.
- **SC-006**: At every supported size, the navigation explanation is reachable within one key press (`?`) from the main list screen.
- **SC-007**: At small sizes, the short hint replaces the full explanation without reducing the number of visible list rows below what the layout mode provides.

## Assumptions

- Users run the application in standard terminal emulators; mouse or touch interaction is out of scope.
- 80x24 is the minimum fully supported size; below it the application degrades to the readable notice with data safety guaranteed.
- Existing functionality, key bindings, and saved data formats are unchanged — only presentation adapts.
- Terminals below 80x24 are out of scope for full usability; only the clear notice and data safety apply there.
- Adaptation is automatic; no user-facing settings are added for layout.
- The application is already usable at large sizes; this feature only concerns reduced sizes and resizing.
- The `?` help shortcut already exists on the main and welcome screens; this feature relies on it rather than adding a new screen or command.
- The full navigation explanation is the help screen's content; the on-list hints are a condensed form of it, and the short hint is a pointer to that screen.
