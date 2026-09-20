# TUI Contract: Quantity Calculation

**Feature**: Quantity Calculation
**Date**: 2026-09-19
**Spec**: [spec.md](../spec.md)

## Overview
The TUI contract defines the user interface and interactions for the quantity calculation feature. It specifies the views, navigation, and state management.

## Views

### Main Application View
The main view of the application, which includes:
- A tree of categories and items on the left side.
- A packing list on the right side.
- Input fields for weekdays, travel style, weather, and activities at the top.
- A summary of packing progress at the bottom.

**Navigation**:
- Use arrow keys to navigate between categories/items.
- Use `Enter` to select/deselect items.
- Use `Tab` to switch focus between the tree and the packing list.
- Use `Ctrl+Q` to quit the application.

---

### Input View
The input view allows users to specify trip details:
- **Weekdays**: Number of weekdays for the trip.
- **Travel Style**: Dropdown or input field for selecting a travel style.
- **Weather**: Dropdown or input field for selecting weather conditions.
- **Activities**: Multi-select input for planned activities.

**Navigation**:
- Use `Tab` to move between input fields.
- Use arrow keys to select options in dropdowns.
- Use `Enter` to confirm selections.

---

### Packing List View
The packing list view displays the items selected for the trip:
- Items are grouped by category.
- Each item shows its name, quantity, and packed status.
- Unpacked items are highlighted in red.
- Fully packed categories are highlighted in green.
- Partially packed categories are highlighted in yellow.

**Navigation**:
- Use arrow keys to navigate between items.
- Use `Space` to toggle the packed status of an item.
- Use `E` to edit the quantity of an item.

---

### Summary View
The summary view provides an overview of packing progress:
- Total number of items.
- Number of items packed.
- Number of items remaining.
- Percentage of items packed.

**Navigation**:
- Use `Tab` to return to the main view.

---

## State Management

### Application State
The application state includes:
- Current packing list.
- Selected categories/items.
- Input values (weekdays, travel style, weather, activities).
- Calculated quantities.
- Packed status of items.

### State Transitions
- **Input Changed**: Recalculate quantities and update the packing list.
- **Item Selected/Deselected**: Update the packing list and summary.
- **Quantity Edited**: Update the item's quantity and recalculate totals.
- **Packed Status Toggled**: Update the item's packed status and category state.

---

## Error Handling

### User Feedback
- Invalid inputs (e.g., negative weekdays) display an error message at the bottom of the screen.
- Conflicting inputs (e.g., "hot weather" and "snowboarding activity") display a warning message.
- File operations (export/import) display success or error messages.

### Recovery
- Users can correct invalid inputs and retry.
- Users can cancel file operations and return to the main view.

---

## Examples

### Main View
```
┌───────────────────────────────────────────────────────────────┐
│ Weekdays: 7 | Travel Style: Minimalist | Weather: Hot        │
│ Activities: [Hiking] [Beach]                                  │
├───────────────────────────────────────────────────────────────┤
│ Categories                                                  │
│ ├─ Clothing                                                  │
│ │  ├─ T-shirt (5)                                           │
│ │  ├─ Pants (2)                                             │
│ │  └─ Jacket (1)                                            │
│ ├─ Electronics                                               │
│ │  └─ Laptop (1)                                            │
│ └─ Toiletries                                               │
│    └─ Toothbrush (1)                                        │
├───────────────────────────────────────────────────────────────┤
│ Packing List                                                │
│ Clothing                                                    │
│ ├─ T-shirt (5) [ ]                                          │
│ ├─ Pants (2) [ ]                                            │
│ └─ Jacket (1) [ ]                                           │
│ Electronics                                                 │
│ └─ Laptop (1) [ ]                                           │
│ Toiletries                                                  │
│ └─ Toothbrush (1) [ ]                                       │
├───────────────────────────────────────────────────────────────┤
│ Packed: 0/9 | Remaining: 9 | Progress: 0%                   │
└───────────────────────────────────────────────────────────────┘
```

### Input View
```
┌───────────────────────────────────────────────────────────────┐
│ Weekdays: [7]                                                │
│ Travel Style: [Minimalist ▼]                                │
│ Weather: [Hot ▼]                                             │
│ Activities: [Hiking □] [Beach □] [City Tour □]              │
├───────────────────────────────────────────────────────────────┤
│ Press Enter to confirm                                       │
└───────────────────────────────────────────────────────────────┘
```

### Packing List View
```
┌───────────────────────────────────────────────────────────────┐
│ Clothing                                                    │
│ ├─ T-shirt (5) [ ]                                          │
│ ├─ Pants (2) [X]                                            │
│ └─ Jacket (1) [ ]                                           │
│ Electronics                                                 │
│ └─ Laptop (1) [X]                                           │
│ Toiletries                                                  │
│ └─ Toothbrush (1) [ ]                                       │
├───────────────────────────────────────────────────────────────┤
│ Packed: 2/9 | Remaining: 7 | Progress: 22%                  │
└───────────────────────────────────────────────────────────────┘
```

> **Alignment note (2026-09-20)**: This contract predates the shipped
> implementation and is superseded. Trip parameters are entered on the
> welcome/trip-setup screen (Up/Down navigate, Left/Right adjust values),
> not on the main screen. Selection uses Space (not Enter); the application
> quits with q or Esc (not Ctrl+Q). See the help screen in the application
> for the shipped key bindings.
