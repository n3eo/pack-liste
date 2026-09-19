# Data Model: Quantity Calculation

**Feature**: Quantity Calculation
**Date**: 2026-09-19
**Spec**: [spec.md](../spec.md)

## Entities

### PackingList
Represents a named collection of items with quantities and packed status.

**Fields**:
- `id: String` (UUID) - Unique identifier for the packing list.
- `name: String` - User-defined name for the packing list.
- `items: Vec<Item>` - List of items in the packing list.
- `created_at: DateTime<Utc>` - Timestamp when the packing list was created.
- `updated_at: DateTime<Utc>` - Timestamp when the packing list was last updated.

**Validation Rules**:
- `name` must not be empty.
- `items` must not exceed a reasonable limit (e.g., 1000 items).

**State Transitions**:
- Created → Updated (when items are added/removed or quantities changed).
- Updated → Exported (when the packing list is exported to a file).

---

### Item
Represents a specific packing item with a quantity and packed status.

**Fields**:
- `id: String` (UUID) - Unique identifier for the item.
- `name: String` - Name of the item (e.g., "T-shirt").
- `quantity: u32` - Number of this item to pack.
- `packed: bool` - Whether the item has been packed.
- `category: String` - Category the item belongs to (e.g., "Clothing").
- `excluded: bool` - Whether the item is excluded from the packing list (subtractive selection).

**Validation Rules**:
- `name` must not be empty.
- `quantity` must be non-negative.
- `category` must match an existing category.

**State Transitions**:
- Created → Updated (when quantity or packed status changes).
- Updated → Excluded (when deselected by the user).

---

### Weekdays
Represents the number of weekdays for a trip.

**Fields**:
- `days: u32` - Number of weekdays.

**Validation Rules**:
- `days` must be non-negative.

---

### TravelStyle
Represents the user's travel style.

**Fields**:
- `style: String` - Travel style (e.g., "minimalist", "luxury", "backpacker").

**Validation Rules**:
- `style` must be one of the predefined values.

---

### WeatherInput
Represents the weather conditions for a trip.

**Fields**:
- `condition: String` - Weather condition (e.g., "hot", "cold", "rainy").

**Validation Rules**:
- `condition` must be one of the predefined values.

---

### Activity
Represents the user's planned activities.

**Fields**:
- `activity: String` - Activity (e.g., "hiking", "beach", "city tour").

**Validation Rules**:
- `activity` must be one of the predefined values.

---

### QuantityCalculation
Represents the algorithm to determine item quantities based on inputs.

**Fields**:
- `weekdays: Weekdays` - Number of weekdays.
- `travel_style: TravelStyle` - User's travel style.
- `weather: WeatherInput` - Weather conditions.
- `activities: Vec<Activity>` - Planned activities.

**Validation Rules**:
- At least one of `weekdays`, `travel_style`, `weather`, or `activities` must be provided.

**State Transitions**:
- Initialized → Calculated (when inputs are provided and quantities are computed).
