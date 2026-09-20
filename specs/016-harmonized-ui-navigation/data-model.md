# Data Model: Harmonized Navigation, UI, and UX Across Views

**Feature**: Harmonized Navigation, UI, and UX Across Views
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This feature is presentation-only: no persisted entities change, no
state machines change. This document records the code-level additions
(one helper, one data function) and the binding-set change so the
implementation has an exact target.

## Binding set change (US1)

**Removed**: Left/Right → collapse/expand on the main screen
(the handler block at src/ui/app.rs:1811–1821).

**Added**: Left/Right → cycle `category_index` within `category_list`
bounds in the add/edit dialog when the Category field is focused
(clamped, no wrap — same bounds logic as the Up/Down dropdown movement).

**Unchanged**: `C` (collapse toggle), Ctrl+Space (category selection +
follow-collapse), Up/Down everywhere, all other bindings.

## New presentation-only code units

### `hint_line(entries: &[(&'static str, &'static str)]) -> Line<'static>`

The single hint-style constructor: keys in `Color::Yellow` bold, actions
plain, " | " between entries, ↑/↓/←/→ glyphs in the key strings.

```text
hint_line(&[("↑/↓", "Navigate"), ("Space", "Select"), ...])
  -> "↑/↓ Navigate | Space Select | ..."
```

### `hint_entries(screen) -> &'static [(&'static str, &'static str)]`

The per-surface hint data, one source for rendering and the extraction
test:

| Surface | Entries (key, action) |
|---------|----------------------|
| Left panel | ↑/↓ Navigate, Space Select, Ctrl+Space Category, A Add, E Edit, D Delete, C Collapse, Tab Switch, ? Help |
| Right panel | ↑/↓ Navigate, Space Packed, Tab Switch, ? Help |
| Welcome | ↑/↓ Navigate fields, ←/→ Adjust values, +/- Weekdays, Space Toggle activity, Enter Edit file / Continue, F Edit path, O Config, Ctrl+E Raw editor, q Quit |
| Config | Tab Section, ↑/↓ Move, A Add, E Edit, D Delete, Ctrl+E Raw editor, Esc Back |
| Add/Edit popup | Tab Cycle fields, ↑/↓ Category dropdown, ←/→ Cycle category, Space Mode / Category, Enter Save, Esc Cancel |
| Delete popup | Enter Confirm, Esc Cancel |
| Modifier editor | Tab Switch field, Enter Save, Esc Cancel |
| Raw editor | ↑/↓/←/→ Cursor, Home/End Line start/end, Enter New line, Backspace Delete, Ctrl+S Save & apply, Esc Cancel |

(The welcome/config rows convert to glyphs per EC-004; the activity
entry remains until spec 017, if implemented, removes it.)

## Layout changes (US2)

| Panel | Split before | Split after |
|-------|--------------|-------------|
| Left (draw_all_items_panel) | [title 1, list min 10] + conditional inline hints | [title 1, list min 10, hints 1] — hints always |
| Right (draw_selected_items_panel) | [title 1, list min 10, stats 5] | [title 1, list min 10, hints 1, stats 5] — hints always |

The conditional `visible_rows + 5 < height` hint block is deleted; the
list viewport shrinks by one row (EC-001 priority: hints over rows).

## Focus indication (US4)

No data change: the existing color-switch mechanism is applied
uniformly; the cursor row highlight (Yellow on DarkGray) is already
identical on both panels and stays.

## Help content changes (US5)

- Remove: main-screen "←/→ Collapse/expand" entry.
- Add: dialog "←/→ Cycle category" entry.
- Convert: welcome entries to glyph style.
- `help_binding_tokens` updated to match; the extraction test extended
  to cross-check `hint_entries` keys against the tokens.

## State transitions

None. No persisted state, no runtime state machine changes. The only
behavioral change is the binding set above.
