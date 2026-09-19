# TUI Usage Guide - Pack-Liste

## Overview

Pack-Liste is a terminal-based packing list application with dynamic quantity calculation. The application has two screens:
1. **Welcome Screen**: Configure trip parameters
2. **Main Screen**: View and manage packing list with categories

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

## Screen 1: Welcome Screen

The welcome screen allows you to configure your trip parameters before viewing the packing list.

### Features
- **Trip Duration**: Set number of weekdays (1-30)
- **Travel Style**: Choose from Minimalist, Standard, Luxury, Backpacker
- **Weather**: Choose from Hot, Cold, Rainy, Snowy, Mixed
- **Activities**: Toggle activities on/off

### Navigation
- **Tab**: Move focus to next field
- **Shift+Tab**: Move focus to previous field
- **Focused field** is highlighted with a yellow border

### Value Adjustment
- **Weekdays**: Use **+** or **-** to increase/decrease
- **Travel Style/Weather/Activities**: Use **↑** or **↓** to cycle through options

### Continue
- **Enter**: Continue to main packing list screen
- **q**: Quit application

## Screen 2: Main Screen

The main screen shows categories and items on the left, and the compiled packing list on the right.

### Left Panel: Categories & Items
- Shows all categories with their items
- Categories can be **collapsed** (▶) or **expanded** (▼)
- Items show their packed status: **✓** (packed) or **○** (not packed)
- All items are **preselected** by default
- Shows item name and calculated quantity

### Right Panel: Compiled Packing List
- Shows the same items grouped by category
- Categories can be collapsed/expanded independently
- Shows statistics at the bottom

### Features

1. **Collapsible Categories**
   - **▼**: Category is expanded (items visible)
   - **▶**: Category is collapsed (items hidden)
   - Press **C** to toggle collapse/expand

2. **Item Status**
   - **✓**: Item is marked as packed
   - **○**: Item is not packed
   - Press **Space** to toggle packed status

3. **Item Selection**
   - All items are preselected by default
   - Press **X** to exclude/include entire category
   - Press **Enter** to toggle individual item selection

4. **Add/Remove Items**
   - Press **A** to add a new item to the current category
   - Press **D** to remove the selected item

5. **Live Quantities**
   - Item quantities are calculated based on trip parameters
   - Quantities update automatically

### Navigation
- **Tab**: Switch between left and right panels
- **↑/↓**: Navigate up and down through categories and items
- **Current selection** is highlighted with yellow

### Actions
| Key | Action |
|-----|--------|
| Tab | Switch between left and right panels |
| ↑/↓ | Navigate through categories/items |
| C | Collapse/expand current category |
| Space | Toggle packed status of selected item |
| Enter | Toggle selection of selected item |
| A | Add new item to current category |
| D | Remove selected item |
| X | Toggle all items in category as selected/excluded |
| q | Quit application |

### Statistics

The right panel shows:
- **Total Items**: Total number of items in the packing list
- **Packed**: Number of items marked as packed
- **Unpacked**: Number of items not yet packed
- **Progress**: Percentage of items packed

## Example Workflow

### Setting Up Your Trip

1. Start the application: `cargo run`
2. You'll see the Welcome Screen
3. Use **Tab** to navigate to Weekdays field
4. Use **+** to set weekdays to 10
5. Use **Tab** to navigate to Travel Style
6. Use **↓** to set to Luxury
7. Use **Tab** to navigate to Weather
8. Use **↓** to set to Cold
9. Use **Tab** to navigate to Activities
10. Use **↓** to add Hiking activity
11. Press **Enter** to continue to main screen

### Managing Your Packing List

1. You'll see the Main Screen with categories on left and compiled list on right
2. Use **↑/↓** to navigate through categories
3. Press **C** to collapse a category
4. Press **C** again to expand it
5. Use **↑/↓** to navigate to an item
6. Press **Space** to mark the item as packed (✓)
7. Press **Space** again to mark it as unpacked (○)
8. Press **A** to add a new item to the current category
9. Press **D** to remove the selected item
10. Press **X** to exclude all items in a category
11. Press **Tab** to switch to the right panel
12. Press **q** to quit when done

## Key Bindings Summary

### Welcome Screen
| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Navigate between fields |
| + / - | Adjust weekdays |
| ↑ / ↓ | Cycle through options |
| Enter | Continue to main screen |
| q | Quit |

### Main Screen
| Key | Action |
|-----|--------|
| Tab | Switch between panels |
| ↑ / ↓ | Navigate |
| C | Collapse/expand category |
| Space | Toggle packed status |
| Enter | Toggle item selection |
| A | Add item |
| D | Remove item |
| X | Toggle category selection |
| q | Quit |

## Configuration

Packing lists are stored in `~/.config/pack-liste/` directory as JSON files.

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

## Implementation Notes

### Features Implemented

✅ **Welcome Screen**
- Trip parameter configuration
- No packing list shown on right
- Clean layout with instructions

✅ **Main Screen**
- Split screen layout (categories left, compiled list right)
- All items preselected
- Categories and items displayed

✅ **Collapsible Categories**
- Each category can be collapsed/expanded
- ▼ symbol for expanded
- ▶ symbol for collapsed
- Press C to toggle

✅ **Navigation**
- Tab to switch between panels
- ↑/↓ to navigate through categories/items
- Current selection highlighted

✅ **Mark as Packed**
- Space to toggle individual item packed status
- ✓ for packed, ○ for unpacked
- Progress percentage calculated

✅ **Add/Remove Items**
- A to add new item to current category
- D to remove selected item
- Items are added with default quantity of 1

✅ **Category Selection**
- X to toggle all items in category as selected/excluded
- Excluded items are hidden from the display

### Features Not Yet Implemented

- Individual item quantity editing
- Save/load packing list configurations
- Multiple packing list management
- Export packing list to file
- Import packing list from file
- Search/filter items
- Custom item creation with name/quantity
