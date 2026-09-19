use anyhow::Result;

use crate::models::activity::Activity;
use crate::models::travel_style::TravelStyle;
use crate::models::weather::WeatherInput;
use crate::models::weekdays::Weekdays;
use crate::quantity::model::{QuantityCalculationInput, QuantityCalculationOutput};

pub struct QuantityCalculation;

impl QuantityCalculation {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(&self, input: &QuantityCalculationInput) -> Result<QuantityCalculationOutput> {
        // Base calculation based on weekdays
        let mut quantities = vec![input.weekdays.get(); 10];
        
        // Apply travel style multiplier
        let style_multiplier = input.travel_style.multiplier();
        quantities = quantities.iter().map(|&q| (q as f32 * style_multiplier).round() as u32).collect();
        
        // Apply weather multiplier
        let weather_multiplier = input.weather.clothing_multiplier();
        quantities = quantities.iter().map(|&q| (q as f32 * weather_multiplier).round() as u32).collect();
        
        // Apply activities multiplier
        if !input.activities.is_empty() {
            let activities_multiplier: f32 = input.activities.iter().map(|a| a.gear_multiplier()).sum();
            let avg_multiplier = activities_multiplier / input.activities.len() as f32;
            quantities = quantities.iter().map(|&q| (q as f32 * avg_multiplier).round() as u32).collect();
        }
        
        let total_items: u32 = quantities.iter().sum();
        let notes = format!(
            "Calculated for {} weekdays, {:?} style, {:?} weather, {:?} activities",
            input.weekdays.get(),
            input.travel_style,
            input.weather,
            input.activities
        );
        
        Ok(QuantityCalculationOutput {
            item_quantities: quantities,
            total_items,
            calculation_notes: notes,
        })
    }
}
