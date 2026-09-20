# Contract: Harmonized Hints and Bindings

**Feature**: Harmonized Navigation, UI, and UX Across Views
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This contract supersedes the binding table in
`specs/015-implemented-specs-compliance/contracts/input-and-help.md`
where they conflict (the main-screen Left/Right removal and the dialog's
Left/Right addition). It defines the final binding set, the hint style,
and the hint↔help agreement guarantee.

## The binding contract (final)

### Global (any screen, any mode)

| Key | Action |
|-----|--------|
| Ctrl+C | Quit immediately (saves config and trip file) |
| Ctrl+E | Open the raw config file editor |
| q | Quit (saves config and trip file) |
| ? | Help screen (from Main and Welcome, normal mode) |
| O | Configuration screen (from any screen except Config/ConfigEditor) |

### Welcome screen

| Key | Action |
|-----|--------|
| ↑/↓ | Move focus between fields (file, weekdays, style, weather, activities) |
| ←/→ | Adjust the focused value (weekdays ±1, cycle style/weather, move activity cursor) |
| +/- | Adjust weekdays |
| Space | Toggle the activity under the cursor |
| Enter | Edit file name (on file field) / continue to main screen |
| F | Edit the trip file path |
| Esc | Quit |

### Main screen (normal mode)

| Key | Action |
|-----|--------|
| ↑/↓ | Move the cursor through the focused panel's rows |
| Tab / BackTab | Switch focus between panels |
| Space | Toggle selection (left) / toggle packed (right) |
| Ctrl+Space | Toggle the category's selection (left panel) |
| C | Toggle collapse/expand of the category under the cursor |
| A | Add a new item |
| E | Edit the item under the cursor |
| D | Delete the item under the cursor (with confirmation) |
| Esc | Quit |

**Removed by this feature**: Left/Right collapse/expand. Left/Right do
nothing on the main screen (nothing is adjustable there).

### Add/Edit item dialog

| Key | Action |
|-----|--------|
| Tab | Cycle fields: Name → Category → Quantity → Mode → Name |
| ↑/↓ | Move the category dropdown highlight (category field) |
| ←/→ | Cycle the category selection (category field) — **new** |
| Space | Toggle the quantity mode (Mode field) / toggle category membership (category field) |
| Enter | Save |
| Esc | Cancel |

### Config screen, modifier editor, raw editor, resume prompt

Unchanged from the spec 015 contract (see
`specs/015-implemented-specs-compliance/contracts/input-and-help.md`).

## The hint style contract

Every hint line in the application is produced by one constructor and
follows:

- Entries: `KEY Action` pairs, keys in the highlight color (yellow,
  bold), actions plain.
- Separator: ` | ` between entries.
- Glyphs: ↑/↓/←/→ for arrows (never "Up/Down"/"Left/Right" words).
- Placement: footer area at the bottom of the screen or panel; popups
  keep their in-popup hint block, restyled.
- Content: only the surface's own bindings.

## The hint↔help agreement guarantee (FR-006)

- `hint_entries(surface)` is the single data source for hint rendering.
- Every key in `hint_entries` appears in `help_binding_tokens`.
- The extraction test cross-checks both directions for each surface's
  binding set and fails on any drift (missing, stale, or duplicated).
- Any binding change updates the handler, `hint_entries`,
  `help_binding_tokens`, and `help_content` in the same change.

## Notes

- Spec 017 (remove activities, Draft), if implemented, removes the
  welcome screen's activity entries from this contract in the same
  change as its handler removal.
- Spec 018 (recalculate, Draft), if implemented, adds its key to the
  main-screen set and both list panels' hint entries.
