# Quantity Calculation Documentation

## Overview

The Quantity Calculation feature dynamically adjusts packing list item quantities based on multiple input factors:

- **Weekdays**: Number of days for the trip
- **Travel Style**: User's preferred packing style (minimalist, standard, luxury, backpacker)
- **Weather Input**: Expected weather conditions (hot, cold, rainy, snowy, mixed)
- **Activities**: Planned activities (hiking, beach, city tour, business, skiing, snowboarding, camping)

## Architecture

### Module Structure

```
src/
├── models/
│   ├── weekdays.rs      # Weekdays input validation
│   ├── travel_style.rs  # Travel style enum and multipliers
│   ├── weather.rs       # Weather input enum and multipliers
│   ├── activity.rs      # Activity enum and multipliers
│   ├── item.rs          # Item model with quantity and packed status
│   └── packing_list.rs  # Packing list model
├── quantity/
│   ├── model.rs         # Quantity calculation input/output models
│   ├── calculation.rs   # Core calculation logic
│   └── error.rs         # Quantity calculation errors
├── services/
│   ├── quantity_service.rs  # Quantity calculation service
│   ├── packing_service.rs   # Packing list management service
│   └── item_service.rs       # Item management service
└── ui/
    └── views/
        ├── input.rs      # Input view for parameters
        ├── list.rs       # Packing list view
        └── summary.rs    # Summary view
```

## Calculation Algorithm

### Base Calculation

1. Start with base quantities based on weekdays
2. Apply travel style multiplier
3. Apply weather multiplier
4. Apply activities multiplier (average of all selected activities)

### Multipliers

| Factor | Value | Multiplier |
|--------|-------|------------|
| Travel Style | Minimalist | 0.5x |
| Travel Style | Standard | 1.0x |
| Travel Style | Luxury | 1.5x |
| Travel Style | Backpacker | 1.2x |
| Weather | Hot | 0.7x |
| Weather | Cold | 1.5x |
| Weather | Rainy | 1.2x |
| Weather | Snowy | 1.8x |
| Weather | Mixed | 1.0x |
| Activity | Hiking | 1.5x |
| Activity | Beach | 0.8x |
| Activity | City Tour | 1.0x |
| Activity | Business | 1.2x |
| Activity | Skiing | 2.0x |
| Activity | Snowboarding | 2.0x |
| Activity | Camping | 1.8x |

## Usage

### CLI

```bash
# Run the application
cargo run

# With debug logging
cargo run -- --debug
```

### Configuration

Packing lists are stored in `~/.config/pack-liste/` directory as JSON files.

### Example Packing List

```json
{
  "id": "list_123456789",
  "name": "Summer Trip 2026",
  "items": [
    {
      "id": "item_1",
      "name": "T-shirts",
      "category": "Clothing",
      "base_quantity": 5,
      "quantity": 7,
      "packed": false,
      "editable": true
    }
  ],
  "file_path": null
}
```

## Error Handling

All errors use `thiserror` for clear error types and `anyhow` for error propagation.

### Error Types

- `WeekdaysError`: Invalid weekdays input
- `QuantityError`: Quantity calculation errors
- `ConfigError`: Configuration errors
- `UiError`: UI rendering errors

## Testing

Run tests with:

```bash
cargo test
```

## Performance

- Real-time updates: <500ms for quantity recalculations
- Memory usage: <100MB
- No unsafe Rust code

## Constraints

- Single-user terminal application
- Offline-capable (no external API dependencies)
- Rust 1.98 (2024 edition)
