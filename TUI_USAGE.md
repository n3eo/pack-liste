# TUI Usage Guide - Pack-Liste

## Overview

Pack-Liste is a terminal-based packing list application with dynamic quantity calculation. The application has these screens:
1. **Welcome Screen**: Configure trip parameters and the packing list file
2. **Main Screen**: View and manage the packing list (all items left, compiled list right)
3. **Help Screen** (`?`): Full navigation reference
4. **Configuration Screen** (`O`): Manage travel style and weather modifiers
5. **Raw Config Editor** (`Ctrl+E`): Edit the config file in-app

## Running the Application

### In a Terminal

To run the interactive TUI:

```bash
cargo run
```

**Note**: All logging has been moved to stderr, so the TUI display is clean.

### Force TUI Mode

If you're in a non-interactive environment but want to test the TUI:

```bash
cargo run -- --force-tui
```

The packing list file can be passed as the first argument:

```bash
cargo run -- my-trip.yaml
```

## Global Keys (any screen)

| Key | Action |
|-----|--------|
| Ctrl+C | Quit immediately (saves config and trip file) |
| Ctrl+E | Open the raw config file editor |
| q | Quit (saves config and trip file) |
| ? | Help screen (from Main and Welcome) |
| O | Configuration screen (from any screen except Config/ConfigEditor) |

## Screen 1: Welcome Screen

The welcome screen allows you to configure your trip parameters and the packing list file before viewing the packing list.

### Fields

- **Packing List File** (required): the path where the compiled packing list is persisted; the first field and required to continue
- **Trip Duration**: number of weekdays (1-30)
- **Travel Style**: e.g. Minimalist, Standard, Luxury, Backpacker
- **Weather**: e.g. Hot, Cold, Rainy, Snowy, Mixed

### Navigation

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate between fields |
| ←/→ | Adjust the focused value (weekdays ±1, cycle style/weather) |
| +/- | Adjust weekdays |
| Enter | Continue (edit file name on the file field) |
| F | Edit the trip file path |
| Esc | Quit |

## Screen 2: Main Screen

The main screen shows all items grouped by category on the left, and the compiled packing list on the right. Both panels show a navigation hint line at the bottom at every supported terminal size.

### Terminal Sizes

The layout adapts to the terminal:

| Size | Layout |
|------|--------|
| 80 columns and wider (at least 24 rows) | Two panels side by side |
| 40–79 columns (at least 24 rows) | Stacked: both panels full-width, one above the other |
| Below 40 columns or 24 rows | A resize notice only — no data is lost; resize back to continue |

Long item names, categories, and file paths truncate with a visible `…` indicator — nothing is cut off silently. When the full hint lines would squeeze the lists, they collapse to a single `? Help` line; `?` opens the help screen at every size. The same key bindings work at every size.

### Fullscreen Compiled List

Press `F` on the main screen to toggle a fullscreen view of the compiled packing list. All compiled-list actions (navigate, toggle packed, recalculate) work there; `F` switches back. The view is presentation-only — quitting from fullscreen saves the trip exactly as from the two-panel view.

### The Two-Layer Category Tree

Categories are displayed as a two-level tree: every category string is split on the first ": " into a top-layer group and a sub-category. "Ausland: Karten" renders as the group "Ausland" with the indented sub-category "Karten" beneath it, and the items beneath that. Categories sharing a top layer appear under one group header; categories without the separator render as a group with their items directly beneath. The stored category strings are unchanged — the nesting is display-only.

### Left Panel: All Items

- Shows the two-layer tree: group headers, indented sub-category headers, items
- Both layers can be **collapsed** (▶) or **expanded** (▼): a group hides its whole sub-tree, a sub-category hides its items
- `Ctrl+Space` on a group header selects/deselects everything beneath the group
- Each item shows its selection state (✓ selected / ○ not selected), name, and quantity
- All items are **preselected** by default on a new list

### Right Panel: Compiled Packing List

- Shows the selected items in the same two-layer tree
- Group and sub-category headers show a packing indicator: ✓ (all packed) / ○ (partially packed)
- Each item shows its packed state (✓ packed / ○ unpacked)
- Shows statistics at the bottom (total, packed, unpacked, progress)

### Navigation

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate the focused panel's rows |
| ←/→ | Nothing (no value to adjust on the main screen) |
| Tab / Shift+Tab | Switch panel |
| Space | Select the item (left) / Packed toggle (right) |
| Ctrl+Space | Category: toggle the category's selection (left panel) |
| C | Toggle collapse/expand of the category under the cursor |
| A | Add a new item |
| E | Edit the item under the cursor |
| D | Delete the item under the cursor (with confirmation) |
| R | Recalc item: the current item (on a header: its scope — sub-category, or the whole group on a group header) |
| Ctrl+R | Recalc category: the cursor's sub-category (the whole group on a group header) |
| Ctrl+Shift+R | Recalc all: every automatic item |
| ? | Help screen |
| Esc | Quit |

## Add/Edit Item Dialog

| Key | Action |
|-----|--------|
| Tab | Next field: Name → Category → Quantity → Mode → Rate → Buffer → Name |
| ↑/↓ | Navigate fields |
| ←/→ | Change the category (category field) / step the quantity ±1 (quantity field) |
| Space | Toggle the focused field's value (Mode / Buffer / Category) |
| Enter | Save (the edit dialog asks for confirmation) |
| Esc | Cancel |

## Configuration Screen (O)

Manage travel style and weather modifiers.

| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Switch section (travel styles / weather) |
| ↑/↓ | Navigate the modifier list |
| A | Add a modifier |
| E | Edit the modifier under the cursor |
| D | Delete the modifier under the cursor |
| Esc | Close the modifier editor if open; otherwise leave the config screen |

## Raw Config Editor (Ctrl+E)

Edit the config file in-app.

| Key | Action |
|-----|--------|
| ↑/↓/←/→ | Navigate the text cursor |
| Home/End | Cursor to line start / end |
| Enter | New line: insert one |
| Backspace | Delete backward |
| Ctrl+S | Save and apply (validates YAML first) |
| Esc | Discard edits and return |

## Example Workflow

### Setting Up Your Trip

1. Start the application: `cargo run`
2. Set the packing list file path (Enter on the file field opens its editor)
3. Use **↑/↓** to move focus to the Weekdays field
4. Use **←/→** or **+/-** to set weekdays to 10
5. Use **↑/↓** to move focus to Travel Style, **←/→** to cycle to Luxury
6. Use **↑/↓** to move focus to Weather, **←/→** to cycle to Cold
7. Press **Enter** to continue to the main screen

### Managing Your Packing List

1. Use **↑/↓** to navigate through categories and items
2. Press **C** to collapse a category; **C** again to expand it
3. Press **Space** on the left panel to toggle an item's selection
4. Press **Tab** to switch to the right panel
5. Press **Space** on the right panel to mark an item as packed (✓)
6. Press **A** to add a new item (the current category is preselected)
7. Press **E** to edit the item under the cursor
8. Press **D** to delete the item under the cursor (confirmation required)
9. Press **R** to recalculate the current item, **Ctrl+R** for its
   category, or **Ctrl+Shift+R** for everything. Quantities never
   change on their own: manual edits are authoritative, and only the
   recalculate keys recalculate (fixed items are never touched).
10. Press **q** to quit when done (config and trip file are saved)

## Configuration

The application configuration (items, categories, trip parameters, modifiers) is stored in `~/.config/pack-liste/config.yaml` as YAML. The compiled packing list is persisted to the packing list file path set on the welcome screen (or passed as the first argument), in a human-readable YAML format that can also be edited by hand.

## Troubleshooting

### "Failed to enable raw mode" Error

This error occurs when the application is run in a non-interactive environment (not a terminal).

**Solutions:**
1. Run the application in a real terminal
2. Use `--force-tui` flag if you're sure your environment supports it
3. Check that your terminal supports ANSI escape codes

### Terminal is Unresponsive

If the TUI starts but you can't interact with it:
1. Press **q** or **Esc** to quit
2. If that doesn't work, press **Ctrl+C** to force quit
3. Make sure you're running in a proper terminal emulator

### No Display or Garbled Output

If the screen is blank or shows garbled characters:
1. Ensure your terminal supports ANSI colors
2. Try a different terminal emulator
3. Check your terminal's character encoding (should be UTF-8)

## Supported Terminals

The application should work in:
- Linux: GNOME Terminal, Konsole, xterm, Alacritty, etc.
- macOS: Terminal.app, iTerm2
- Windows: Windows Terminal, PowerShell, CMD (with ANSI support)
- WSL: Windows Subsystem for Linux terminal

## Performance

- Real-time updates: <500ms for quantity recalculations
- Memory usage: <100MB
- No unsafe Rust code

