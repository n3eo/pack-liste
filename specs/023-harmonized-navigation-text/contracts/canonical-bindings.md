# Contract: Canonical Navigation Bindings

The user-facing contract of the harmonized navigation text: what each key is called and what it is said to do, everywhere.

## The canonical table

Each entry is the single wording for that (key, meaning). A surface's hint line shows a subset; the help screen shows all with longer descriptions.

| Key | Action | Meaning / where it applies |
|-----|--------|---------------------------|
| `↑/↓` | Navigate | Move the cursor / focus (all list and field surfaces) |
| `←/→` | Adjust | Change the focused value (welcome fields, dialog category/quantity) |
| `Tab` | Switch panel | Move focus between the main panels |
| `Tab` | Switch section | Move between config screen sections |
| `Tab` | Next field | Move to the next dialog field |
| `Space` | Select | Toggle an item's selection (left panel) |
| `Space` | Packed | Toggle an item's packed state (right panel) |
| `Space` | Toggle | Cycle the focused dialog field's value (Mode/Buffer/Category) |
| `Enter` | Confirm | Apply a confirmation popup |
| `Enter` | Save | Save the add/edit dialog |
| `Enter` | Continue | Leave the welcome screen for the main screen |
| `Enter` | New line | Insert a line in the raw editor |
| `Esc` | Cancel | Close a popup or editor without applying |
| `Esc` | Back | Leave the config screen |
| `F` | Fullscreen | Toggle the fullscreen compiled list (main screen) |
| `F` | Edit path | Edit the trip file path (welcome screen) |
| `R` | Recalc item | Recalculate the current item |
| `Ctrl+R` | Recalc category | Recalculate the current category |
| `Ctrl+Shift+R` | Recalc all | Recalculate every automatic item |
| `A` | Add | Add a new item / modifier |
| `E` | Edit | Edit the item / modifier under the cursor |
| `D` | Delete | Delete the item / modifier under the cursor |
| `C` | Collapse | Collapse/expand the category under the cursor |
| `Ctrl+Space` | Category | Toggle the whole category's selection (left panel) |
| `?` | Help | Open the help screen |
| `O` | Config | Open the config screen |
| `Ctrl+E` | Raw editor | Open the raw config editor |
| `Ctrl+S` | Save & apply | Save and apply the raw editor's content |
| `q` | Quit | Quit the application |
| `Ctrl+C` | Quit | Quit immediately |
| `Home/End` | Line start/end | Jump the raw editor cursor |
| `Backspace` | Delete | Delete backward (raw editor) |

## Surface compositions (which entries each surface shows)

- **LeftPanel**: `↑/↓ Navigate`, `Space Select`, `Ctrl+Space Category`, `A Add`, `E Edit`, `D Delete`, `C Collapse`
- **RightPanel**: `↑/↓ Navigate`, `Space Packed`, `R Recalc item`, `Ctrl+R Recalc category`, `Ctrl+Shift+R Recalc all`, `Tab Switch panel`, `F Fullscreen`, `? Help`
- **Welcome**: `↑/↓ Navigate`, `←/→ Adjust`, `+/- Weekdays`, `Enter Continue`, `F Edit path`, `O Config`, `Ctrl+E Raw editor`, `q Quit`
- **Config**: `Tab Switch section`, `↑/↓ Navigate`, `A Add`, `E Edit`, `D Delete`, `Ctrl+E Raw editor`, `Esc Back`
- **AddEditPopup**: `Tab Next field`, `↑/↓ Navigate`, `←/→ Adjust`, `Space Toggle`, `Enter Save`, `Esc Cancel`
- **DeletePopup / ConfirmEditPopup**: `Enter Confirm`, `Esc Cancel`
- **ModifierEditor**: `Tab Next field`, `Enter Save`, `Esc Cancel`
- **RawEditor**: `↑/↓/←/→ Navigate`, `Home/End Line start/end`, `Enter New line`, `Backspace Delete`, `Ctrl+S Save & apply`, `Esc Cancel`

## De-duplication rule (the main screen)

The shared pairs (`↑/↓`, `Tab`, `F`, `R`, `Ctrl+R`, `Ctrl+Shift+R`, `?`) appear in the RIGHT panel's line only; the left panel's line carries only the left-specific bindings. No (key, action) pair is visible twice on the same screen.

## Help screen agreement

Every key the help screen lists is a canonical key above; each help description contains its key's canonical action word. The help screen may add detail (scopes, conditions) but never contradicts the canonical wording.

## Invariants (pinned by tests)

1. Every (key, meaning) has exactly one action text in the source.
2. No surface composition contains a duplicate pair.
3. The two main panels' compositions are pair-disjoint.
4. Every canonical entry is used by at least one composition or the help screen.
5. No key behavior changes — the contract is display text only.
