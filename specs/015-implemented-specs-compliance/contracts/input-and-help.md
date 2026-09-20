# Contract: Key Bindings and Help Screen

**Feature**: Implemented-Specs Compliance Review
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This is the contract this feature establishes: the complete, binding
key-map of the application, and the guarantee that the help screen and
the code stay in 1:1 agreement. US4 implements it; the extraction test
enforces it.

## The binding contract

Every binding the application responds to, grouped by context. The help
screen lists exactly this set, and only this set.

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
| Up/Down | Move focus between fields (file, weekdays, style, weather, activities) |
| Left/Right | Adjust the focused value (weekdays ±1, cycle style/weather, move activity cursor) |
| +/- | Adjust weekdays |
| Space | Toggle the activity under the cursor |
| Enter | Edit file name (on file field) / continue to main screen |
| F | Edit the trip file path |
| Esc | Quit |

### Resume prompt

| Key | Action |
|-----|--------|
| Y / Enter | Resume the previous trip file |
| N / Esc | Start a fresh list |

### Main screen (normal mode)

| Key | Action |
|-----|--------|
| Up/Down | Move the cursor through the focused panel's rows |
| Tab / BackTab | Switch focus between panels |
| Space | Toggle selection (left) / toggle packed (right) |
| Ctrl+Space | Toggle the category's selection (left panel) |
| C | Toggle collapse/expand of the category under the cursor |
| A | Add a new item |
| E | Edit the item under the cursor |
| D | Delete the item under the cursor (with confirmation) |
| Esc | Quit |

### Add/Edit item dialog

| Key | Action |
|-----|--------|
| Tab | Cycle fields: Name → Category → Quantity → Mode → Name |
| Up/Down | Move the category dropdown highlight (category field) |
| Left/Right | Cycle the category selection (category field) |
| Space | Toggle the quantity mode (Mode field) / toggle category membership (multi-category, category field) |
| Enter | Save |
| Esc | Cancel |

### Config screen (modifiers)

| Key | Action |
|-----|--------|
| Tab / BackTab | Switch section (travel styles / weather / activities) |
| Up/Down | Move the modifier cursor |
| A | Add a modifier |
| E | Edit the modifier under the cursor |
| D | Delete the modifier under the cursor |
| Esc | Close the modifier editor if open; otherwise leave the config screen |

### Modifier editor popup

| Key | Action |
|-----|--------|
| Tab | Switch between name and multiplier field |
| Enter | Save the modifier |
| Esc | Cancel the editor |

### Raw config editor

| Key | Action |
|-----|--------|
| Up/Down/Left/Right | Move the text cursor |
| Home/End | Cursor to line start / end |
| Enter | Insert a new line |
| Backspace | Delete backward |
| Ctrl+S | Save and apply (validates; error shown on failure) |
| Esc | Discard edits and return |

## The 1:1 guarantee (SC-004)

- The help screen's content is generated from this contract's grouping.
- An automated test extracts the binding tokens from the key-handler
  match arms and from the help screen's text, and fails on:
  - a handler binding with no help entry (missing),
  - a help entry with no handler (stale),
  - a duplicate entry.
- Any change that adds, removes, or rebinds a key must update this
  contract, the help screen, and the handlers in the same change; the
  extraction test is the reviewer that never forgets.

## Notes

- Spec 014 (uniform arrow navigation, Draft) will remove the
  Left/Right collapse binding from the main screen; when it is
  implemented, this contract and the help screen change with it in the
  same change.
- Spec 011 (confirm item edit, Draft) will insert a confirmation step
  after the edit dialog's Enter; the dialog's binding set is unchanged,
  so this contract needs no key changes for it.
