use anyhow::Result;

use crate::quantity::model::{
    ItemQuantitySpec, QuantityCalculationInput, QuantityCalculationOutput,
};

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
        let quantities: Vec<u32> = input
            .item_specs
            .iter()
            .map(|spec| match spec {
                ItemQuantitySpec::Fixed(qty) => *qty,
                ItemQuantitySpec::Auto => (base as f32 * combined).round().max(1.0) as u32,
            })
            .collect();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::weekdays::Weekdays;

    fn input(
        weekdays: u32,
        style: f32,
        weather: f32,
        activities: &[f32],
        specs: Vec<ItemQuantitySpec>,
    ) -> QuantityCalculationInput {
        QuantityCalculationInput {
            weekdays: Weekdays::new(weekdays).unwrap(),
            item_specs: specs,
            travel_style: "test-style".to_string(),
            travel_style_multiplier: style,
            weather: "test-weather".to_string(),
            weather_multiplier: weather,
            activities: activities.iter().map(|_| "activity".to_string()).collect(),
            activity_multipliers: activities.to_vec(),
        }
    }

    #[test]
    fn base_quantity_is_weekdays_times_multipliers() {
        // 7 days, standard (1.0), cold (1.5), no activities
        let specs = vec![ItemQuantitySpec::Auto; 3];
        let out = QuantityCalculation
            .calculate(&input(7, 1.0, 1.5, &[], specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![11, 11, 11]); // 7 * 1.5 = 10.5 -> 11
        assert_eq!(out.total_items, 33);
    }

    #[test]
    fn activities_replace_weather_multiplier() {
        // Activities take priority: average of [1.0, 2.0] = 1.5 replaces weather 0.7
        let specs = vec![ItemQuantitySpec::Auto; 3];
        let out = QuantityCalculation
            .calculate(&input(7, 1.0, 0.7, &[1.0, 2.0], specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![11, 11, 11]); // 7 * 1.5 = 10.5 -> 11
        assert!(out.calculation_notes.contains("take priority"));
    }

    #[test]
    fn quantity_is_capped_at_twenty() {
        // 30 days * 1.5 * 2.0 = 90 -> capped base 20, then 20 * 3.0 = 60
        let specs = vec![ItemQuantitySpec::Auto; 3];
        let out = QuantityCalculation
            .calculate(&input(30, 1.5, 2.0, &[], specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![60, 60, 60]);
    }

    #[test]
    fn quantity_never_drops_below_one() {
        // 1 day * 0.5 * 0.7 = 0.35 -> rounds to 0 -> clamped to 1
        let specs = vec![ItemQuantitySpec::Auto; 3];
        let out = QuantityCalculation
            .calculate(&input(1, 0.5, 0.7, &[], specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![1, 1, 1]);
    }

    #[test]
    fn fixed_items_keep_their_quantity() {
        // A toothbrush stays at 1 regardless of trip parameters
        let specs = vec![
            ItemQuantitySpec::Auto,
            ItemQuantitySpec::Fixed(1),
            ItemQuantitySpec::Fixed(3),
        ];
        let out = QuantityCalculation
            .calculate(&input(14, 1.5, 1.8, &[], specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![38, 1, 3]); // 14 * 1.5 * 1.8 = 37.8 -> 38
    }

    #[test]
    fn output_length_matches_item_count() {
        let specs = vec![ItemQuantitySpec::Auto; 3];
        let out = QuantityCalculation
            .calculate(&input(7, 1.0, 1.0, &[], specs))
            .unwrap();
        assert_eq!(out.item_quantities.len(), 3);
    }
}
