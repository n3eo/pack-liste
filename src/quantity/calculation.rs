use anyhow::Result;

use crate::quantity::model::{QuantityCalculationInput, QuantityCalculationOutput};

/// Maximum quantity for any item, per spec 005 EC-003.
const MAX_QUANTITY: u32 = 20;

pub struct QuantityCalculation;

impl QuantityCalculation {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(&self, input: &QuantityCalculationInput) -> Result<QuantityCalculationOutput> {
        // Base quantity from trip length, capped per spec
        let base = input.weekdays.get().min(MAX_QUANTITY);

        // Travel style and weather multipliers
        let style_multiplier = input.travel_style_multiplier;
        let weather_multiplier = input.weather_multiplier;

        // Activities take priority over conflicting weather (FR-006):
        // when activities are selected, their average multiplier replaces
        // the weather multiplier
        let effective_weather_multiplier = if !input.activity_multipliers.is_empty() {
            let sum: f32 = input.activity_multipliers.iter().sum();
            sum / input.activity_multipliers.len() as f32
        } else {
            weather_multiplier
        };

        let combined = style_multiplier * effective_weather_multiplier;
        let quantities = vec![
            (base as f32 * combined).round().max(1.0) as u32;
            input.item_count
        ];

        let total_items: u32 = quantities.iter().sum();
        let notes = if input.activity_multipliers.is_empty() {
            format!(
                "Calculated for {} weekdays, {} style, {} weather",
                input.weekdays.get(),
                input.travel_style,
                input.weather,
            )
        } else {
            format!(
                "Calculated for {} weekdays, {} style, {} weather (activities [{}] take priority)",
                input.weekdays.get(),
                input.travel_style,
                input.weather,
                input.activities.join(", "),
            )
        };

        Ok(QuantityCalculationOutput {
            item_quantities: quantities,
            total_items,
            calculation_notes: notes,
        })
    }
}

impl Default for QuantityCalculation {
    fn default() -> Self {
        Self::new()
    }
}
