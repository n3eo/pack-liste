# Quantity Calculation Contract

**Feature**: Quantity Calculation
**Date**: 2026-09-19
**Spec**: [spec.md](../spec.md)

## Overview
The quantity calculation contract defines the interface for calculating item quantities based on user inputs (weekdays, travel style, weather, activities). It specifies the inputs, outputs, and validation rules.

## Inputs

### Weekdays
- **Type**: `u32`
- **Description**: Number of weekdays for the trip.
- **Validation**: Must be non-negative.
- **Default**: `0`

### TravelStyle
- **Type**: `String`
- **Description**: User's travel style (e.g., "minimalist", "luxury", "backpacker").
- **Validation**: Must be one of the predefined values.
- **Default**: `"minimalist"`

### WeatherInput
- **Type**: `String`
- **Description**: Weather conditions for the trip (e.g., "hot", "cold", "rainy").
- **Validation**: Must be one of the predefined values.
- **Default**: `"moderate"`

### Activities
- **Type**: `Vec<String>`
- **Description**: Planned activities for the trip (e.g., "hiking", "beach", "city tour").
- **Validation**: Must be one of the predefined values.
- **Default**: `[]`

---

## Outputs

### CalculatedQuantities
- **Type**: `HashMap<String, u32>`
- **Description**: A map of item names to their calculated quantities.
- **Example**: `{"T-shirt": 5, "Pants": 2, "Laptop": 1}`

---

## Calculation Logic

### Algorithm
1. **Base Quantities**: Start with a base quantity for each item based on the number of weekdays.
   - Example: `base_quantity = weekdays * 0.5` (rounded up).

2. **Travel Style Adjustment**: Adjust quantities based on travel style.
   - **Minimalist**: Reduce quantities by 30%.
   - **Luxury**: Increase quantities by 30%.
   - **Backpacker**: No adjustment.

3. **Weather Adjustment**: Adjust quantities based on weather.
   - **Hot**: Increase quantities of summer clothing by 20%.
   - **Cold**: Increase quantities of winter clothing by 20%.
   - **Rainy**: Increase quantities of rain gear by 20%.

4. **Activity Adjustment**: Adjust quantities based on activities.
   - **Hiking**: Increase quantities of hiking gear by 50%.
   - **Beach**: Increase quantities of beachwear by 50%.
   - **City Tour**: No adjustment.

5. **Conflict Resolution**: If conflicting inputs are provided (e.g., "hot weather" and "snowboarding activity"), prioritize the activity input and display a warning message.

---

## Error Handling

### Errors
- **InvalidInput**: Returned when any input fails validation.
- **CalculationError**: Returned when the calculation fails (unlikely, but included for completeness).

### Error Messages
- **InvalidInput**: "Invalid input: {input_name}. Please provide a valid value."
- **CalculationError**: "Failed to calculate quantities. Please try again."

---

## Examples

### Example 1: Minimalist Traveler (7 Weekdays, Hot Weather)
**Inputs**:
- Weekdays: `7`
- Travel Style: `"minimalist"`
- Weather: `"hot"`
- Activities: `[]`

**Calculation**:
1. Base quantities: `T-shirt: 4`, `Pants: 2`, `Jacket: 1`
2. Minimalist adjustment: `T-shirt: 3`, `Pants: 1`, `Jacket: 1`
3. Hot weather adjustment: `T-shirt: 4`, `Pants: 2`, `Jacket: 1`

**Output**: `{"T-shirt": 4, "Pants": 2, "Jacket": 1}`

---

### Example 2: Luxury Traveler (10 Weekdays, Cold Weather, Hiking)
**Inputs**:
- Weekdays: `10`
- Travel Style: `"luxury"`
- Weather: `"cold"`
- Activities: `["hiking"]`

**Calculation**:
1. Base quantities: `T-shirt: 5`, `Pants: 3`, `Jacket: 2`, `Hiking Boots: 1`
2. Luxury adjustment: `T-shirt: 7`, `Pants: 4`, `Jacket: 3`, `Hiking Boots: 2`
3. Cold weather adjustment: `T-shirt: 7`, `Pants: 4`, `Jacket: 4`, `Hiking Boots: 2`
4. Hiking activity adjustment: `T-shirt: 7`, `Pants: 4`, `Jacket: 4`, `Hiking Boots: 3`

**Output**: `{"T-shirt": 7, "Pants": 4, "Jacket": 4, "Hiking Boots": 3}`

---

### Example 3: Conflicting Inputs (Hot Weather + Snowboarding)
**Inputs**:
- Weekdays: `5`
- Travel Style: `"minimalist"`
- Weather: `"hot"`
- Activities: `["snowboarding"]`

**Calculation**:
1. Base quantities: `T-shirt: 3`, `Pants: 1`, `Winter Jacket: 1`, `Snowboard: 1`
2. Minimalist adjustment: `T-shirt: 2`, `Pants: 1`, `Winter Jacket: 1`, `Snowboard: 1`
3. Activity adjustment (snowboarding): `T-shirt: 2`, `Pants: 1`, `Winter Jacket: 1`, `Snowboard: 1`

**Warning**: "Conflicting inputs detected. Prioritizing activity input."

**Output**: `{"T-shirt": 2, "Pants": 1, "Winter Jacket": 1, "Snowboard": 1}`

> **Alignment note (2026-09-20)**: This contract predates the shipped
> implementation and is superseded. Weekdays are validated to 1-30 with a
> default of 7 (not non-negative with default 0). Modifier names are
> user-configurable (spec 002), not a fixed predefined set, and unknown
> names fall back to a neutral multiplier of 1.0. The calculation rule is
> documented in spec 005 (FR-006 to FR-010) and unit-tested in
> src/quantity/calculation.rs.
