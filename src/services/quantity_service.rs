use anyhow::Result;

use crate::models::activity::Activity;
use crate::models::travel_style::TravelStyle;
use crate::models::weather::WeatherInput;
use crate::models::weekdays::Weekdays;

pub struct QuantityService;

impl QuantityService {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_base_quantities(&self, weekdays: Weekdays) -> Result<Vec<u32>> {
        Ok(vec![weekdays.get(); 10])
    }

    pub fn calculate_travel_style(&self, quantities: Vec<u32>, style: TravelStyle) -> Result<Vec<u32>> {
        let multiplier = style.multiplier();
        Ok(quantities.iter().map(|&q| (q as f32 * multiplier).round() as u32).collect())
    }

    pub fn calculate_weather(&self, quantities: Vec<u32>, weather: WeatherInput) -> Result<Vec<u32>> {
        let multiplier = weather.clothing_multiplier();
        Ok(quantities.iter().map(|&q| (q as f32 * multiplier).round() as u32).collect())
    }

    pub fn calculate_activities(&self, quantities: Vec<u32>, activities: &[Activity]) -> Result<Vec<u32>> {
        let multiplier: f32 = activities.iter().map(|a| a.gear_multiplier()).sum();
        let avg_multiplier = if activities.is_empty() { 1.0 } else { multiplier / activities.len() as f32 };
        Ok(quantities.iter().map(|&q| (q as f32 * avg_multiplier).round() as u32).collect())
    }
}
