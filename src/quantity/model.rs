use serde::{Deserialize, Serialize};

use crate::models::weekdays::Weekdays;

/// Input for the quantity calculation.
///
/// The modifier names are carried for documentation purposes; the multipliers
/// are resolved from the config file by the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityCalculationInput {
    pub weekdays: Weekdays,
    pub item_count: usize,
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
