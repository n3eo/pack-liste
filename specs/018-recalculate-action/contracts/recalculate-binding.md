# Contract: Recalculate Binding and Hints

**Feature**: Recalculate Amounts on an Existing List
**Date**: 2026-09-20
**Spec**: [spec.md](../spec.md)

This contract amends the spec 016 hint/binding contract with one new
main-screen binding. Everything else is unchanged.

## The binding (final)

### Main screen (normal mode) — after this feature

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
| **R** | **Recalculate quantities on the current list** |
| Esc | Quit |

**Added by this feature**: `R` — one key press, no focus-side guard,
active on the main screen in normal mode only. On every other screen
or mode the key is unbound (EC-002).

## Action semantics

- Recalculates every **automatic** item's quantity from the current
  in-memory trip parameters (weekdays, travel style, weather) and
  per-item configuration.
- **Fixed** items never change (FR-002).
- Idempotent: repeated presses with unchanged inputs produce
  identical quantities (FR-003).
- Touches no packed, selection, or collapse state; writes no file
  (FR-004).
- Feedback is the redrawn quantities alone (EC-005).
- Uses the in-memory state; external file changes enter memory only
  through the raw config editor's apply flow (EC-003).

## Hint surfaces (after)

| Surface | Entries |
|---------|---------|
| Left panel | ↑/↓ Navigate, Space Select, Ctrl+Space Category, A Add, E Edit, D Delete, C Collapse, **R Recalculate**, Tab Switch, ? Help |
| Right panel | ↑/↓ Navigate, Space Packed, **R Recalculate**, Tab Switch, ? Help |

## Hint/help agreement guarantee (unchanged mechanism, extended data)

- `hint_entries` remains the single data source; both list surfaces
  gain `("R", "Recalculate")`.
- `help_content`'s MAIN SCREEN section documents `R`.
- `help_binding_tokens` gains `"R"`;
  `help_screen_text_covers_all_handler_bindings` gains `"R"` to its
  handler list; `hint_entries_match_the_contract_binding_sets` gains
  the entry in both list panels' expected sets.
- Any later change that drops or corrupts the `R` entry in hints,
  help, or tokens is a test failure (US2/AC3, SC-004).

## Unchanged

All other bindings, surfaces, and file formats — identical to
`specs/016-harmonized-ui-navigation/contracts/hints-and-bindings.md`
(as amended by spec 017's contract if 017 has landed).
