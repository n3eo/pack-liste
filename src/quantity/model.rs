use serde::{Deserialize, Serialize};

use crate::models::weekdays::Weekdays;

/// How the quantity of one item is determined.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ItemQuantitySpec {
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
pub struct QuantityCalculationInput {
    pub weekdays: Weekdays,
    pub item_specs: Vec<ItemQuantitySpec>,
    pub travel_style: String,
    pub travel_style_multiplier: f32,
    pub weather: String,
    pub weather_multiplier: f32,
    pub activities: Vec<String>,
    pub activity_multipliers: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityCalculationOutput {
    pub item_quantities: Vec<u32>,
    pub total_items: u32,
    pub calculation_notes: String,
}
