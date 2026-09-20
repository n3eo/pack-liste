# Feature Specification: Harmonized Navigation, UI, and UX Across Views

**Feature Branch**: `016-harmonized-ui-navigation`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"Review the navigation, ui and ux and harmonize across all views"

Refined in the same session with:
"add navigation explanation to list views"

> **Scope note**: A review of every screen found concrete
> inconsistencies, listed below. This spec harmonizes them: one
> navigation model, one hint system, one visual language. The fourth
> request ("add navigation explanation to list views") is absorbed here
> as the hint-system story — the right panel currently shows no
> navigation explanation at all, and the left panel's is conditional
> and buggy.

## Review findings (verified in code)

| # | Finding | Where |
|---|---------|-------|
| 1 | Right panel (compiled list) shows **no navigation hints at all** | draw_selected_items_panel has a stats footer but no hint line |
| 2 | Left panel hints render **only with 5+ spare rows**, silently vanish otherwise; positioned inline below items, not as a footer | draw_all_items_panel |
| 3 | Add/edit popup hint text is **corrupted**: "Select fields.category" (refactor artifact) | draw_add_item_popup / draw_edit_item_popup |
| 4 | Hint **separator styles mix** " - " and " | " within and across screens | all hint lines |
| 5 | Hint **placement varies**: footer block (welcome, config), inline-below-list (left panel), in-popup block (dialogs), none (right panel) | all screens |
| 6 | **Key glyph styles vary**: "Up/Down" vs "↑/↓", "Left/Right" vs "←/→" across screens | all hint lines |
| 7 | Left/Right **collapse/expand on the main screen** contradicts the uniform arrow model (spec 014, Draft) | main screen handler |
| 8 | Panel titles use **inconsistent focus indication** (color-only) | both panels |
| 9 | Help screen (rebuilt in spec 015) is complete but its **grouping differs from on-screen hints** | help vs hints |

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One navigation model everywhere (Priority: P1)

On every screen, the arrows mean the same thing: Up/Down navigate (cursor, field focus, list selection), Left/Right adjust the focused value (cycle options, step numbers). No screen repurposes the arrows. This implements spec 014's model: the main screen's Left/Right collapse/expand binding is removed (collapse stays on its dedicated `C` key), and the add/edit dialog's category field cycles with Left/Right.

**Why this priority**: Navigation is the most-used interaction; one meaning everywhere removes the need to remember which screen you are on. The main screen is the only remaining violator of the model the welcome screen already follows.

**Independent Test**: On each screen, press Up/Down and verify only position changes; press Left/Right and verify only values change (or nothing, where nothing is adjustable); verify collapse/expand still works via `C`.

**Acceptance Scenarios**:

1. **Given** the main screen, **When** the user presses Left/Right, **Then** nothing collapses or expands.
2. **Given** the main screen, **When** the user presses `C` on a category, **Then** it collapses/expands exactly as before (cursor to header included).
3. **Given** the add/edit dialog with the category field focused, **When** the user presses Left/Right, **Then** the category selection cycles.
4. **Given** the welcome screen, **When** the user presses Up/Down then Left/Right, **Then** field focus moves, then the focused value adjusts (existing behavior, now the model everywhere).

---

### User Story 2 - Every list view shows its navigation explanation (Priority: P1)

Both list views — the all-items tree (left) and the compiled list (right) — show a navigation explanation line, always. The hints are a fixed footer area of the panel (not conditional on spare rows), so they never silently vanish; the compiled list gains the hints it never had. The hint content matches the panel's own bindings: left panel (navigate, select, category toggle, add/edit/delete, collapse, switch, help), right panel (navigate, toggle packed, switch, help).

**Why this priority**: This is the explicit "add navigation explanation to list views" request. The left panel's hints currently disappear below 5 spare rows, and the right panel has none — the two list views, the core of the application, are the least documented screens.

**Independent Test**: Open the main screen at several terminal sizes, including small; verify both panels show their hint line at every size and the hint content matches each panel's actual bindings.

**Acceptance Scenarios**:

1. **Given** the main screen at any supported size, **When** it renders, **Then** both panels show a navigation hint line.
2. **Given** a deep list filling the whole panel, **When** the screen renders, **Then** the hint line remains visible (the list viewport shrinks, not the hints).
3. **Given** the left panel's hints, **When** read, **Then** they describe the left panel's bindings (select, category, add/edit/delete, collapse).
4. **Given** the right panel's hints, **When** read, **Then** they describe the right panel's bindings (navigate, toggle packed, switch panel).

---

### User Story 3 - One hint style and placement system (Priority: P1)

All screens use one hint system: a footer area at the bottom of the screen or panel, one line per context where space allows, with a single separator style (" | " between entries), a single key-glyph style (the arrow glyphs ↑/↓/←/→ already used on the main and config screens), keys in the existing highlight color, and actions in plain text. The corrupted popup hint ("Select fields.category") is fixed. No screen's hints contradict another's.

**Why this priority**: The current mix (" - " vs " | ", "Up/Down" vs "↑/↓", footer vs inline vs none) makes the hints look broken rather than styled; one system makes them readable and trustworthy.

**Independent Test**: Compare every screen's hint line: same separator, same glyph style, same placement pattern, no corrupted text.

**Acceptance Scenarios**:

1. **Given** any screen's hint line, **When** read, **Then** entries are separated by " | " and keys use the ↑/↓/←/→ glyph style consistently.
2. **Given** the add/edit dialog, **When** the hints render, **Then** the category line reads correctly (no "fields." artifact).
3. **Given** all screens, **When** their hint placements are compared, **Then** each uses the footer pattern (bottom of screen or panel).
4. **Given** the help screen and the on-screen hints, **When** compared, **Then** they describe the same bindings in the same terms.

---

### User Story 4 - Consistent focus and state indication (Priority: P2)

Both panels indicate focus the same way, and item state is indicated consistently: the focused panel's title is highlighted (existing color convention kept, applied uniformly), the cursor row is highlighted identically on both panels, and selection/packed symbols use the established vocabulary (✓ green / ○ red) without per-panel deviations.

**Why this priority**: Focus indication is how the user knows which panel Space will act on; inconsistent indication creates wrong-key errors.

**Independent Test**: Switch focus between panels; verify the title highlight and cursor style follow one convention on both sides.

**Acceptance Scenarios**:

1. **Given** focus switches from left to right, **When** the screen redraws, **Then** the right panel's title is highlighted and the left's is not, per one convention.
2. **Given** the cursor on either panel, **When** the row renders, **Then** the cursor highlight style is identical on both panels.

---

### User Story 5 - Help screen mirrors the harmonized model (Priority: P2)

The help screen (rebuilt in spec 015) is updated to the harmonized bindings: the main screen's Left/Right collapse entry is removed (collapse documented under `C` only), the dialog's Left/Right category cycling is documented, and the help's extraction test is extended to cover the on-screen hint lines so hints and help cannot drift apart again.

**Why this priority**: The help screen is the reference; it must state the final binding set, and the drift guard must cover hints, not only help.

**Independent Test**: Run the extraction test; break a hint line deliberately and verify the test fails.

**Acceptance Scenarios**:

1. **Given** the help screen, **When** read, **Then** no binding attributes collapse/expand to Left/Right.
2. **Given** the hint-extraction test, **When** a hint line drops or corrupts a binding, **Then** the test fails.

### Edge Cases

- **EC-001**: What happens to the hints at very small terminal heights?
  **Answer**: The hint footer keeps priority over list rows: the list viewport shrinks first; below the minimum supported size the whole app shows the too-small notice (spec 006's rule). Hints never overlap or overwrite list content.
  **Acceptance Criteria**: At every supported size, hints and list content never overlap.

- **EC-002**: Do the popups (add/edit, delete confirm, modifier editor) also get footer hints?
  **Answer**: Popups keep their in-popup hint block (it is part of the dialog's content), but adopt the harmonized style: " | " separators, arrow glyphs, correct text. Placement harmonization applies to full screens and panels; popup hints stay inside the popup.
  **Acceptance Criteria**: Popup hints use the harmonized style; full-screen hints use footers.

- **EC-003**: Does removing Left/Right collapse break the Ctrl+Space follow-selection behavior?
  **Answer**: No — Ctrl+Space toggles a category's selection and collapses/expands per the existing rule; only the arrow binding is removed.
  **Acceptance Criteria**: Ctrl+Space behavior is unchanged.

- **EC-004**: Which glyph style wins — "Up/Down" or "↑/↓"?
  **Answer**: The arrow glyphs (↑/↓/←/→), already the majority style on the main and config screens. The welcome screen's text style ("Up/Down") is converted.
  **Acceptance Criteria**: No hint line uses the long-form words where glyphs fit.

- **EC-005**: Does the hint footer replace the right panel's stats block?
  **Answer**: No — the compiled list keeps its stats (total/packed/progress) and gains the hint line above or beside them within the same footer area; both remain visible at supported sizes.
  **Acceptance Criteria**: Both stats and hints are visible on the compiled list.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Up/Down MUST navigate and Left/Right MUST adjust values on every screen, per spec 014's model; the main screen's Left/Right collapse/expand binding MUST be removed, leaving `C` as the collapse toggle.
- **FR-002**: Both list views (all-items tree and compiled list) MUST show a navigation hint line at every supported terminal size, as a fixed footer area of the panel.
- **FR-003**: All hint lines MUST use one style: " | " entry separators, ↑/↓/←/→ key glyphs, keys highlighted, actions plain.
- **FR-004**: The corrupted popup hint text MUST be corrected.
- **FR-005**: Panel focus indication and cursor highlighting MUST follow one convention across both panels.
- **FR-006**: The help screen MUST reflect the harmonized bindings, and an extraction test MUST cover both the help content and the on-screen hint lines.
- **FR-007**: Hint footers MUST NOT overlap or displace list content; at reduced heights the list viewport shrinks first.

### Key Entities *(include if feature involves data)*

- **Navigation model**: The uniform arrow semantics (spec 014): vertical = move, horizontal = adjust.
- **Hint footer**: The per-panel bottom area showing that panel's bindings; always present on list views.
- **Hint style**: The single separator/glyph/color convention for all key hints.
- **Focus indication**: The uniform visual convention showing which panel receives actions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On 100% of screens, Up/Down only navigate and Left/Right only adjust (or do nothing where nothing is adjustable).
- **SC-002**: Both list views show a correct hint line at 100% of supported terminal sizes.
- **SC-003**: Zero hint-style deviations (separator, glyph, placement) across all screens; zero corrupted hint texts.
- **SC-004**: The help screen and all hint lines agree on every binding, enforced by the extraction test.
- **SC-005**: All existing navigation/selection/packed tests pass unchanged (except the removed arrow-collapse binding's coverage, which is updated with the change).

## Assumptions

- Spec 014's model is the target; this feature implements its main-screen and dialog requirements rather than duplicating them (014 remains the model's definition; this spec is its application plus the hint system).
- The hint footer takes at most one line per panel; content is the panel's own bindings only.
- The stats block on the compiled list stays; hints are added within the footer region.
- Popup hints stay inside their popups, restyled only.
- The `C` key remains the collapse toggle; Ctrl+Space's follow-selection behavior is untouched.
- The help extraction test (spec 015) is the drift guard; this feature extends it to hint lines.
