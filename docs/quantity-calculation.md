# Quantity Calculation Documentation

## Overview

The quantity calculation dynamically adjusts packing list item quantities
based on the trip parameters:

- **Weekdays**: Number of days for the trip
- **Travel Style**: User's preferred packing style (e.g. minimalist,
  standard, luxury, backpacker)
- **Weather**: Expected weather condition (e.g. hot, cold, rainy, mixed)

Travel styles and weather conditions are user-configurable (see the
configuration screen, `O` from any screen); each carries a multiplier.

## Architecture

### Module Structure

```
src/
├── models/
│   ├── item.rs          # Item model with quantity, packed, and selection state
│   ├── packing_list.rs  # The packing list (item catalog)
│   └── weekdays.rs      # Weekdays input validation
├── config/
│   ├── modifiers.rs     # ModifierConfig: named multipliers (travel style, weather)
│   └── trip_file.rs     # The persisted trip file (compiled packing list)
└── quantity/
    ├── model.rs         # Quantity calculation input/output models
    └── calculation.rs   # Core calculation logic
```

The UI layer (`src/ui/`) drives the calculation: `state.rs` holds the trip
parameters and calls `recalculate_quantities`, which resolves the
configured multipliers and delegates to `QuantityCalculation::calculate`.

### Calculation Rule

For each item:

- **Fixed items** (`auto_quantity: false`, e.g. a toothbrush) keep their
  quantity; it is never recalculated; their stored modifiers are
  inactive while fixed.
- **Automatic items** get (spec 010's per-item rule):

```
rate amount = round(min(weekdays, 20) * style_multiplier * weather_multiplier * rate)
quantity    = rate amount + reserve          (flat buffer)
          or rate amount * buffer_multiplier (scaling buffer)
```

clamped to 1..=99. The base is capped at 20 days (spec 005 EC-003);
the weather multiplier always applies (the pre-activity rule, spec
017). The trip multipliers touch only the rate-based part — the
reserve is flat, the multiplier scales the rate amount (spec 010
EC-006).

**Per-item modifiers** (spec 010), configured in the add/edit dialog:

- **Rate** (`rate`): how many of this item one trip day needs
  (0.05–10.0, default 1.0 = one per day; 0.15 ≈ one per week).
- **Buffer** — exactly one kind per item (never both):
  - **Reserve** (`reserve`): a flat amount (0–10) added on top,
    independent of trip length and multipliers (e.g. +2 spare
    underwear).
  - **Multiplier** (`buffer_multiplier`): a scaling factor (1.0–3.0)
    applied to the rate amount (e.g. 1.25 = pack a quarter more).

Items without modifiers (rate 1.0, no buffer) calculate exactly as
before spec 010; pre-feature config files load with those defaults.

Unknown travel style or weather names fall back to a multiplier of 1.0.

### Recalculation Triggers

Quantities recalculate when trip parameters change (weekdays, travel
style, weather on the welcome screen) and on the explicit `R` key on the
main screen (spec 018). Manual edits to a fixed item's quantity are
authoritative; only `R` recalculates.

## Storage

The input configuration (`~/.config/pack-liste/config.yaml`) stores the
item catalog with per-item `auto_quantity` flags; fixed items persist
their quantity, automatic items do not (they recalculate on load).

The compiled packing list is persisted as a separate human-editable YAML
trip file (the path chosen on the welcome screen or passed as the first
CLI argument). It stores the trip parameters and the selected items with
their packed state **and their quantities** (spec 012) — the file is a
complete, self-contained snapshot of the trip. On resume the saved
quantities are restored as-is, not recalculated: the file is
authoritative for its own trip, and hand edits to the quantities in the
file are honored. On load it is reconciled against the current input
config: items gone from the config are dropped, categories are remapped
to the catalog item's primary category; quantities survive for items
that reconcile.

## Error Handling

The calculation returns `anyhow::Result`; failures propagate to the UI,
where they are shown in the status line (a footer message on the main
screen) instead of terminating the session.
