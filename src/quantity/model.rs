use serde::{Deserialize, Serialize};

use crate::models::item::QuantityBuffer;
use crate::models::weekdays::Weekdays;

/// How the quantity of one item is determined.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum ItemQuantitySpec {
    /// Recalculated from trip parameters and the item's own modifiers
    /// (per-day rate and buffer, spec 010)
    Auto { rate: f32, buffer: QuantityBuffer },
    /// Fixed quantity, never recalculated (e.g. toothbrush)
    Fixed(u32),
}

/// Input for the quantity calculation.
///
/// The modifier names are carried for documentation purposes; the multipliers
/// are resolved from the config file by the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QuantityCalculationInput {
    pub(crate) weekdays: Weekdays,
    pub(crate) item_specs: Vec<ItemQuantitySpec>,
    pub(crate) travel_style: String,
    pub(crate) travel_style_multiplier: f32,
    pub(crate) weather: String,
    pub(crate) weather_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The calculated quantities, one per input item, plus the total and a
/// human-readable note describing the parameters used.
pub(crate) struct QuantityCalculationOutput {
    pub(crate) item_quantities: Vec<u32>,
    pub(crate) total_items: u32,
    pub(crate) calculation_notes: String,
}
