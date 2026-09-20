# Data Model: Remove Activity Options

**Feature**: Remove Activity Options
**Date**: 2026-09-20
**Spec**: [spec.md](spec.md)

This feature deletes a dimension; no entity is added. This document
records the exact shape of the changed structs and enums so the
implementation has an exact target.

## Removed entities and fields

| Item | File | Change |
|------|------|--------|
| `Focus::Activities` | `src/ui/app.rs` | Variant deleted; `next()`/`prev()` arms removed |
| `ConfigSection::Activities` | `src/ui/app.rs` | Variant deleted; `title()`/`next()`/`prev()` arms removed |
| `AppDrawState.selected_activities` | `src/ui/app.rs` | Field deleted |
| `AppDrawState.activity_cursor` | `src/ui/app.rs` | Field deleted |
| `AppConfig.selected_activities` | `src/ui/app.rs` | Field deleted (config file no longer carries it) |
| `TripParams.selected_activities` | `src/ui/app.rs` | Field deleted |
| `TripFile.selected_activities` | `src/config/trip_file.rs` | Field deleted (trip file no longer carries it) |
| `ModifiersConfig.activities` | `src/config/modifiers.rs` | Field deleted |
| `default_activities()` | `src/config/modifiers.rs` | Function deleted |
| `ModifiersConfig::activity_multiplier()` | `src/config/modifiers.rs` | Method deleted |
| `QuantityCalculationInput.activities` | `src/quantity/model.rs` | Field deleted |
| `QuantityCalculationInput.activity_multipliers` | `src/quantity/model.rs` | Field deleted |
| `toggle_activity` | `src/ui/app.rs` | Method deleted |
| Activity-priority branch in `calculate` | `src/quantity/calculation.rs` | Branch deleted |

## Surviving shapes (after the change)

```rust
// src/ui/app.rs
pub enum Focus { TripFile, Weekdays, TravelStyle, Weather }
// next(): TripFile -> Weekdays -> TravelStyle -> Weather -> TripFile
// prev(): the reverse

enum ConfigSection { TravelStyles, Weather }
// next(): TravelStyles <-> Weather; prev(): the reverse

struct AppDrawState {
    // ... unchanged fields ...
    // selected_activities, activity_cursor: gone
}

struct AppConfig {
    categories: Vec<String>,
    items: Vec<SavedItem>,
    weekdays: u32,
    travel_style: String,
    weather: String,
    // selected_activities: gone
    modifiers: ModifiersConfig,
    last_trip_file: Option<String>,
}

struct TripParams {
    weekdays: Weekdays,
    travel_style: String,
    weather: String,
    // selected_activities: gone
    modifiers: ModifiersConfig,
    last_trip_file: Option<String>,
}
```

```rust
// src/config/modifiers.rs
pub struct ModifiersConfig {
    #[serde(default = "default_travel_styles")]
    pub travel_styles: Vec<ModifierConfig>,
    #[serde(default = "default_weather")]
    pub weather: Vec<ModifierConfig>,
    // activities: gone
}
// activity_multiplier(): gone; travel_style_multiplier and
// weather_multiplier unchanged
```

```rust
// src/quantity/model.rs
pub struct QuantityCalculationInput {
    pub weekdays: Weekdays,
    pub item_specs: Vec<ItemQuantitySpec>,
    pub travel_style: String,
    pub travel_style_multiplier: f32,
    pub weather: String,
    pub weather_multiplier: f32,
    // activities, activity_multipliers: gone
}
```

```rust
// src/config/trip_file.rs
pub struct TripFile {
    pub weekdays: u32,
    pub travel_style: String,
    pub weather: String,
    // selected_activities: gone
    #[serde(default)]
    pub items: Vec<TripItem>,
}
```

## The revised calculation (FR-002)

```
base        = min(weekdays, 20)
combined    = travel_style_multiplier * weather_multiplier
auto qty    = max(round(base * combined), 1)
fixed qty   = unchanged (never recalculated)
notes       = "Calculated for {weekdays} weekdays, {style} style, {weather} weather"
```

The weather multiplier always applies (EC-001); no input or state can
suppress it.

## File formats after the change

**Config file** (written): `categories`, `items`, `weekdays`,
`travel_style`, `weather`, `modifiers` (with `travel_styles` and
`weather` only), `last_trip_file`. No activity keys.

**Trip file** (written): `weekdays`, `travel_style`, `weather`,
`items`. No activity keys.

**Legacy tolerance** (load): serde ignores unknown fields by default, so
files that still carry `selected_activities` or
`modifiers.activities` parse without error; the values are dropped.
This is not version-compatibility machinery — no code reads them, and
the next save rewrites the file without them (EC-005).

## State transitions

None beyond the removal: the welcome focus cycle and the config
section cycle each lose one member; all remaining transitions are
unchanged. No persisted state machine changes (the removed fields were
plain data).
