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

```
+-----------------------------------------------------+
| Welcome to Pack-Liste                               |
|-----------------------------------------------------|
| +-------------------------------+                   |
| | Trip Duration (Weekdays)      |                   |
| |                               |                   |
| |  Current: 7 days              |                   |
| |                               |                   |
| |  Use +/- to adjust            |                   |
| +-------------------------------+                   |
| +-------------------------------+                   |
| | Travel Style                 |                   |
| |                               |                   |
| |  Current: Standard            |                   |
| |                               |                   |
| |  Use ↑/↓ to cycle through     |                   |
| +-------------------------------+                   |
| +-------------------------------+                   |
| | Weather                      |                   |
| |                               |                   |
| |  Current: Mixed              |                   |
| |                               |                   |
| |  Use ↑/↓ to cycle through     |                   |
| +-------------------------------+                   |
| +-------------------------------+                   |
| | Activities                   |                   |
| |                               |                   |
| |  Current: None               |                   |
| |                               |                   |
| |  Use ↑/↓ to toggle            |                   |
| +-------------------------------+                   |
| Tab to navigate between fields                       |
| Enter to continue to packing list                     |
| q to quit                                            |
+-----------------------------------------------------+
```

### Navigation

- **Tab**: Move focus to next field
- **Shift+Tab**: Move focus to previous field
- **Focused field** is highlighted with a yellow border

### Value Adjustment

- **Weekdays**: Use **+** or **-** to increase/decrease the number of days (1-30)
- **Travel Style**: Use **↑** or **↓** to cycle through options
- **Weather**: Use **↑** or **↓** to cycle through options
- **Activities**: Use **↑** or **↓** to toggle activities

### Continue to Main Screen

- **Enter**: After configuring all parameters, press Enter to continue to the main packing list screen

## Screen 2: Main Screen

The main screen shows categories and items on the left, and the compiled packing list on the right.

```
+---------------------+---------------------+
| Categories & Items  | Compiled Packing List|
|---------------------+---------------------|
| ▼ Clothing          | ▼ Clothing          |
|   ✓ T-Shirts x 7   |   ✓ T-Shirts x 7   |
|   ○ Pants x 5       |   ○ Pants x 5       |
|   ○ Socks x 7       |   ○ Socks x 7       |
|   ○ Underwear x 7   |   ○ Underwear x 7   |
|                     |                     |
| ▼ Toiletries        | ▼ Toiletries        |
|   ✓ Toothbrush x 1 |   ✓ Toothbrush x 1 |
|   ✓ Toothpaste x 1 |   ✓ Toothpaste x 1 |
|   ✓ Shampoo x 1    |   ✓ Shampoo x 1    |
|                     |                     |
| ▼ Documents         | ▼ Documents         |
|   ✓ Passport x 1   |   ✓ Passport x 1   |
|   ✓ Tickets x 1    |   ✓ Tickets x 1    |
|                     |                     |
|                     | Total Items: 9      |
|                     | Packed: 6           |
|                     | Unpacked: 3         |
|                     | Progress: 67%       |
|                     |                     |
| Tab to switch panels|                     |
| C to collapse/expand|                     |
| Space to mark done  |                     |
+---------------------+---------------------+
```

### Features

1. **Collapsible Categories**: Each category can be collapsed or expanded
   - **▼**: Category is expanded (items visible)
   - **▶**: Category is collapsed (items hidden)

2. **Item Status**: Each item shows its packed status
   - **✓**: Item is marked as packed
   - **○**: Item is not packed

3. **Live Quantities**: Item quantities are calculated based on your trip parameters from the welcome screen

### Navigation

- **Tab**: Switch between left panel (categories/items) and right panel (compiled list)
- **↑/↓**: Navigate up and down through categories and items
- **Current focus** is highlighted

### Actions

- **C**: Collapse or expand the current category
- **Space**: Mark the current item or all items in the current category as done/undone
- **Enter**: Select (action depends on context)

### Statistics

The right panel shows:
- **Total Items**: Total number of items in the packing list
- **Packed**: Number of items marked as packed
- **Unpacked**: Number of items not yet packed
- **Progress**: Percentage of items packed

## Key Bindings Summary

### Welcome Screen

| Key | Action |
|-----|--------|
| Tab | Next field |
| Shift+Tab | Previous field |
| + | Increase weekdays |
| - | Decrease weekdays |
| ↑ | Cycle up (travel style, weather, activities) |
| ↓ | Cycle down (travel style, weather, activities) |
| Enter | Continue to main screen |
| q | Quit application |

### Main Screen

| Key | Action |
|-----|--------|
| Tab | Switch between left and right panels |
| Shift+Tab | Switch between left and right panels |
| ↑ | Navigate up |
| ↓ | Navigate down |
| C | Collapse/expand current category |
| Space | Mark item/category as done/undone |
| Enter | Select |
| q | Quit application |

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
11. Press **Enter** to continue

### Managing Your Packing List

1. You'll see the Main Screen with categories on left and compiled list on right
2. Use **Tab** to switch to the right panel
3. Use **C** to collapse a category
4. Use **C** again to expand it
5. Use **Space** to mark all items in a category as packed
6. Use **Space** again to mark them as unpacked
7. Use **Tab** to switch back to left panel
8. Press **q** to quit when done

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

## Future Enhancements

- [ ] Proper dropdown menus for selection in welcome screen
- [ ] Individual item selection (not just category-wide)
- [ ] Edit individual item quantities
- [ ] Save/load packing list configurations
- [ ] Multiple packing list management
- [ ] Search/filter items
- [ ] Export packing list to PDF/print
