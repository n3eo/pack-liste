use serde::{Deserialize, Serialize};

use crate::models::activity::Activity;
use crate::models::travel_style::TravelStyle;
use crate::models::weather::WeatherInput;
use crate::models::weekdays::Weekdays;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityCalculationInput {
    pub weekdays: Weekdays,
    pub travel_style: TravelStyle,
    pub weather: WeatherInput,
    pub activities: Vec<Activity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityCalculationOutput {
    pub item_quantities: Vec<u32>,
    pub total_items: u32,
    pub calculation_notes: String,
}
