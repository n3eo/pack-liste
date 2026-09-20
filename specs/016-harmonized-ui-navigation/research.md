# Research: Harmonized Navigation, UI, and UX Across Views

**Feature**: Harmonized Navigation, UI, and UX Across Views
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This research resolves the design questions for the five harmonization
stories, grounded in the current code (line numbers from the working tree).

## US1 — Uniform arrow model on the main screen and dialog

### Decision: delete the Left/Right collapse handler; add Left/Right category cycling to the dialog

**Rationale**: The main-screen handler at src/ui/app.rs:1811–1821 is the
only violator of the model (welcome screen already follows it). Deleting
the block removes the binding; `C` (toggle_category_collapse, :1339 area)
and Ctrl+Space (follow-selection) are untouched, so no functionality is
lost. In the add/edit dialog, `handle_add_edit_input` (:2622) currently
handles only Char/Backspace; adding a `KeyCode::Left | KeyCode::Right`
arm that cycles `category_index` within `category_list` bounds (the same
clamp logic `move_cursor_up/down` use at :1178/:1218) gives the category
field its value-adjust binding per the model. The dialog's Up/Down
dropdown movement stays (it is navigation within the dropdown list).

**Alternatives considered**:
- Keeping Left/Right collapse as an alias: rejected — the spec's FR-001
  says MUST be removed; an alias is the contradiction the feature exists
  to remove.
- Moving collapse to Left/Right only when the cursor is on a header:
  rejected — conditional arrow meanings are worse than none.

### Test impact

The state-layer tests from spec 015 cover collapse via `C`
(`collapse_moves_cursor_to_header`); no test binds arrows to collapse, so
deletion is test-safe. New dialog tests assert Left/Right cycling and
bounds clamping.

## US2 — Always-visible hint footers on both list views

### Decision: reserve one footer row per panel via the layout split; hints render unconditionally

**Rationale**: `draw_all_items_panel` (:3799) splits its area into
[title(1), list(min 10)] and renders hints inline below the items only
when `visible_rows + 5 < height` (:3957) — the vanishing condition. The
fix: change the split to [title(1), list(min 10), hints(1)] and render
the hint line in the fixed hints area, deleting the conditional.
`draw_selected_items_panel` (:4004) splits [title(1), list(min 10),
stats(5)] — add a hints row: [title(1), list(min 10), hints(1),
stats(5)]. Both panels then always show hints; the list viewport shrinks
by one row, which is the spec's EC-001 priority (hints over rows).

**Alternatives considered**:
- Overlaying hints on the last list row: rejected — overlaps content,
  violating FR-007.
- A single shared hint bar under both panels: rejected — each panel has
  its own bindings (the spec's US-2 requires per-panel content), and a
  shared bar cannot express both.

### Hint content (per panel, from the actual bindings)

- Left: `↑/↓ Navigate | Space Select | Ctrl+Space Category | A Add | E Edit | D Delete | C Collapse | Tab Switch | ? Help`
- Right: `↑/↓ Navigate | Space Packed | Tab Switch | ? Help`

## US3 — One hint style

### Decision: a `hint_line(entries: &[(&str, &str)]) -> Line` helper; all screens use it

**Rationale**: Every hint line today is hand-built `Line::from(vec![...])`
with mixed separators (" - " in popups/help, " | " on panels) and mixed
glyphs ("Up/Down" on welcome vs "↑/↓" on main/config). One helper that
takes (key, action) pairs and renders `key action | key action` — keys in
`Color::Yellow` bold, actions plain, " | " between entries — makes the
style uniform by construction and gives the extraction test one place to
read. The corrupted popup text "Select fields.category" (:4336 area,
both popups) is replaced with the correct "Select category". The welcome
screen's "Up/Down"/"Left/Right" words convert to ↑/↓/←/→ glyphs (EC-004).

**Alternatives considered**:
- Fixing styles in place per screen: rejected — five hand-maintained
  copies is exactly how the styles drifted; the helper is the drift
  prevention.

### Placement

Full screens and panels use footer areas (welcome and config already do;
the two list panels gain them per US2). Popups keep their in-popup hint
blocks (EC-002), restyled via the helper.

## US4 — Focus and cursor indication

### Decision: keep the color convention, apply it uniformly; cursor style already uniform, verified

**Rationale**: Both panel titles already switch color on focus (left:
Green/White at :3822–3833; right: Cyan/White at :4022–4033) — the
convention exists but uses a different highlight color per panel. The
harmonization: the focused panel's title uses one highlight treatment
(bold + the panel's own accent color is kept — the spec's US-4 says
"existing color convention kept, applied uniformly", meaning the
mechanism, not identical colors). The cursor row highlight
(Yellow on DarkGray) is already identical on both panels (:3899–3903,
:4140–4144) — verified, no change needed there.

**Alternatives considered**:
- A `[FOCUS]` text marker in the title: rejected — adds a second
  indication mechanism; the color convention is sufficient when applied
  consistently.

## US5 — Help and extraction test

### Decision: update help_content + help_binding_tokens; extend the test to hint lines

**Rationale**: `help_content` (:3365) and `help_binding_tokens` (:3309)
are the single source the extraction test (:558–630) checks. Changes:
remove the main-screen Left/Right collapse entry; add the dialog's
Left/Right category cycling; the welcome entries keep their glyph
conversion. The test extension: the hint helper (US3) exposes each
screen's hint entries as data (`hint_entries(screen) -> &[(&str, &str)]`),
and the test asserts every hint entry's key appears in
`help_binding_tokens` and vice versa for the screens' binding sets —
hints and help cannot drift.

**Alternatives considered**:
- Generating the help screen from the hint data: attractive but a larger
  refactor of help_content's grouping; the data-driven hint entries +
  cross-check deliver the guarantee with a smaller diff.

## Dependency note

No new dependencies. US3's helper lands first (US2's footers use it);
US1's binding changes land before US5's help update so the help states
the final set once. The extraction test is updated in the same change as
every binding change (constitution Principle III).

## Verification approach

Per story: state-layer tests for the dialog cycling (US1), render-level
assertions via the hint data (US2/US3), the extended extraction test
(US5), and the full gate (`cargo check`, `cargo clippy`, `cargo clippy
--tests`, `cargo test`, `cargo fmt --check`) after every task. A pty
smoke run verifies the footers at two terminal sizes.
