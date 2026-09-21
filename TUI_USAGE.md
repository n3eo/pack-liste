# TUI Usage Guide - Pack-Liste

## Overview

Pack-Liste is a terminal-based packing list application with dynamic quantity calculation. The application has these screens:
1. **Welcome Screen**: Configure trip parameters and the packing list file
2. **Main Screen**: View and manage the packing list (all items left, compiled list right)
3. **Help Screen** (`?`): Full navigation reference
4. **Configuration Screen** (`Ctrl+O`): Manage travel style and weather modifiers
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
| Ctrl+O | Configuration screen (from any screen except Config/ConfigEditor) |

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

The main screen shows all items grouped by label on the left, and the compiled packing list on the right. Both panels show a navigation hint line at the bottom at every supported terminal size.

### Terminal Sizes

The layout adapts to the terminal:

| Size | Layout |
|------|--------|
| 80 columns and wider (at least 24 rows) | Two panels side by side |
| 40–79 columns (at least 24 rows) | Stacked: both panels full-width, one above the other |
| Below 40 columns or 24 rows | A resize notice only — no data is lost; resize back to continue |

Long item names, labels, and file paths truncate with a visible `…` indicator — nothing is cut off silently. The quantity always stays on the row: the name truncates first, then the label chips. When the full hint lines would squeeze the lists, they collapse to a single `? Help` line; `?` opens the help screen at every size. The same key bindings work at every size.

### Fullscreen Compiled List

Press `F` on the main screen to toggle a fullscreen view of the compiled packing list. All compiled-list actions (navigate, toggle packed, recalculate) work there; `F` switches back. The view is presentation-only — quitting from fullscreen saves the trip exactly as from the two-panel view.

### Labels and the Flat Grouping

Items are grouped by **labels** — a flat, single-level grouping. Every item carries one or more labels (plain strings, no hierarchy, no `": "` separator). The panels show one header per distinct label (sorted alphabetically), with the items carrying that label beneath it. An item with multiple labels appears under each of its labels' groups — it is still one item: one quantity, one edit, counted once in the statistics.

Behind every item's name its labels render as small grey chips (e.g. ` [Ausland] [Karten]`): dark grey when the occurrence's toggle state matches the item's own, light grey when it differs.

### Singletons

A **singleton** is an item whose toggles are synchronized across all its labels: toggling it — selecting it for the trip, or marking it packed — under any of its labels flips it everywhere. This is the model for "one physical thing appearing in several groups" (one toothbrush listed under two labels is ONE toothbrush). A non-singleton item keeps independent toggle states per label occurrence: toggling it under one label changes only that occurrence, and the other labels' chips turn light grey to show the divergence.

### Left Panel: All Items

- Shows the flat label grouping: one header per label, the carrying items beneath
- Labels can be **collapsed** (▶) or **expanded** (▼): a collapsed label hides its items
- `Ctrl+Space` on a label header selects/deselects every item carrying that label
- Each item shows its occurrence's selection state (✓ selected / ○ not selected), name, label chips, and quantity
- All items are **preselected** by default on a new list

### Right Panel: Compiled Packing List

- Shows the selected items in the same flat label grouping
- Label headers show a packing indicator: ✓ (all packed) / ○ (partially packed)
- Each item shows its occurrence's packed state (✓ packed / ○ unpacked)
- Shows statistics at the bottom (total, packed, unpacked, progress) — items are counted once, not per label occurrence

### Navigation

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate the focused panel's rows |
| ←/→ | Nothing (no value to adjust on the main screen) |
| Tab / Shift+Tab | Switch panel |
| Space | Select the item occurrence (left) / Packed toggle (right) |
| Ctrl+Space | Label: toggle the label's selection (left panel) |
| C | Toggle collapse/expand of the label under the cursor |
| A | Add a new item |
| E | Edit the item under the cursor |
| D | Delete the item under the cursor (with confirmation) |
| R | Recalc item: the current item (on a label header: every item carrying that label) |
| Ctrl+R | Recalc label: every item carrying the cursor's label |
| Ctrl+Shift+R | Recalc all: every automatic item |
| ? | Help screen |
| Esc | Quit |

## Add/Edit Item Dialog

| Key | Action |
|-----|--------|
| Tab | Next field: Name → Labels → Singleton → Quantity → Mode → Rate → Buffer → Name |
| ↑/↓ | Navigate fields |
| ←/→ | Cycle the highlighted label (Labels field) / step the quantity ±1 (Quantity field) |
| Space | Toggle the focused field's value (Labels: the highlighted label's membership / Singleton / Mode / Buffer) |
| Enter | Save (the edit dialog asks for confirmation) |
| Esc | Cancel |

The **Labels** field is a multi-select: the picker lists every existing label, `←/→` moves the highlight, `Space` toggles membership, and free text adds a new label. At least one label is required to save. The **Singleton** field (`Space` toggles yes/no) sets whether the item's toggles synchronize across all its labels.

## Configuration Screen (Ctrl+O)

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

1. Use **↑/↓** to navigate through labels and items
2. Press **C** to collapse a label; **C** again to expand it
3. Press **Space** on the left panel to toggle an item occurrence's selection
4. Press **Tab** to switch to the right panel
5. Press **Space** on the right panel to mark an item occurrence as packed (✓)
6. Press **A** to add a new item (the cursor's label is preselected)
7. Press **E** to edit the item under the cursor
8. Press **D** to delete the item under the cursor (confirmation required)
9. Press **R** to recalculate the current item, **Ctrl+R** for its
   label, or **Ctrl+Shift+R** for everything. Quantities never
   change on their own: manual edits are authoritative, and only the
   recalculate keys recalculate (fixed items are never touched).
10. Press **q** to quit when done (config and trip file are saved)

## Configuration

The application configuration (items, labels, trip parameters, modifiers) is stored in `~/.config/pack-liste/config.yaml` as YAML. The compiled packing list is persisted to the packing list file path set on the welcome screen (or passed as the first argument), in a human-readable YAML format that can also be edited by hand.

An item in the config file:

```yaml
items:
- name: Führerschein international
  labels:
  - Ausland
  - Karten
  singleton: true
  auto_quantity: false
  quantity: 1
- name: T-Shirt
  labels:
  - Ausland
  - Handgepäck
  auto_quantity: true
```

- `labels`: one or more plain strings — the item's only grouping data; no `": "` separator, no hierarchy
- `singleton`: optional (absent = false) — the item's toggles synchronize across all its labels
- `quantity`: only present for fixed-quantity items (`auto_quantity: false`); automatic items are recalculated on every load
- Duplicate entries (same name AND same labels) load as one item — the first occurrence wins; same name with different labels are distinct items

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

