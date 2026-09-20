use serde::{Deserialize, Serialize};

use crate::models::weekdays::Weekdays;

/// How the quantity of one item is determined.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum ItemQuantitySpec {
    /// Recalculated from trip parameters on every change
    Auto,
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
pub(crate) struct QuantityCalculationOutput {
    pub(crate) item_quantities: Vec<u32>,
    pub(crate) total_items: u32,
    pub(crate) calculation_notes: String,
}
