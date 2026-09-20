use anyhow::Result;

use crate::models::item::QuantityBuffer;
use crate::quantity::model::{
    ItemQuantitySpec, QuantityCalculationInput, QuantityCalculationOutput,
};

/// Maximum base (trip days) for any item, per spec 005 EC-003.
const MAX_QUANTITY: u32 = 20;

/// Maximum final quantity for any item, per spec 010 EC-003.
const MAX_TOTAL_QUANTITY: u32 = 99;

/// The quantity calculation: trip length times the configured travel
/// style and weather multipliers, per item.
pub(crate) struct QuantityCalculation;

impl QuantityCalculation {
    /// Create the calculator (stateless).
    pub(crate) fn new() -> Self {
        Self
    }

    /// Calculate every item's quantity: fixed items keep their value,
    /// automatic items get round(min(weekdays, 20) * style * weather)
    /// clamped to a minimum of 1.
    pub(crate) fn calculate(
        &self,
        input: &QuantityCalculationInput,
    ) -> Result<QuantityCalculationOutput> {
        // Base quantity from trip length, capped per spec
        let base = input.weekdays.get().min(MAX_QUANTITY);

        // Travel style and weather multipliers; the weather multiplier
        // always applies (the pre-activity rule, spec 017)
        let combined = input.travel_style_multiplier * input.weather_multiplier;
        let quantities: Vec<u32> = input
            .item_specs
            .iter()
            .map(|spec| match spec {
                ItemQuantitySpec::Fixed(qty) => *qty,
                ItemQuantitySpec::Auto { rate, buffer } => {
                    // Per-item rule (spec 010): the rate-based amount is
                    // the trip multipliers applied to the item's own
                    // per-day rate; the buffer then adds a flat reserve
                    // or scales the amount — never both (EC-006: trip
                    // multipliers touch only the rate-based part).
                    let rate_amount = (base as f32 * combined * rate).round().max(1.0) as u32;
                    match buffer {
                        QuantityBuffer::None => rate_amount,
                        QuantityBuffer::Reserve(n) => rate_amount + n,
                        QuantityBuffer::Multiplier(m) => {
                            ((rate_amount as f32) * m).round().max(1.0) as u32
                        }
                    }
                    // Final clamp 1..=99 (EC-002/EC-003, FR-007)
                    .min(MAX_TOTAL_QUANTITY)
                }
            })
            .collect();

        let total_items: u32 = quantities.iter().sum();
        let notes = format!(
            "Calculated for {} weekdays, {} style, {} weather",
            input.weekdays.get(),
            input.travel_style,
            input.weather,
        );

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

    /// An automatic spec with the default modifiers (rate 1.0, no buffer).
    fn auto() -> ItemQuantitySpec {
        ItemQuantitySpec::Auto {
            rate: 1.0,
            buffer: QuantityBuffer::None,
        }
    }

    fn input(
        weekdays: u32,
        style: f32,
        weather: f32,
        specs: Vec<ItemQuantitySpec>,
    ) -> QuantityCalculationInput {
        QuantityCalculationInput {
            weekdays: Weekdays::new(weekdays).unwrap(),
            item_specs: specs,
            travel_style: "test-style".to_string(),
            travel_style_multiplier: style,
            weather: "test-weather".to_string(),
            weather_multiplier: weather,
        }
    }

    #[test]
    fn base_quantity_is_weekdays_times_multipliers() {
        // 7 days, standard (1.0), cold (1.5)
        let specs = vec![auto(); 3];
        let out = QuantityCalculation
            .calculate(&input(7, 1.0, 1.5, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![11, 11, 11]); // 7 * 1.5 = 10.5 -> 11
        assert_eq!(out.total_items, 33);
    }

    #[test]
    fn weather_multiplier_always_applies() {
        // The pre-activity rule (spec 017): nothing can suppress the
        // weather multiplier; the direct product always governs
        let specs = vec![auto(); 3];
        let out = QuantityCalculation
            .calculate(&input(7, 1.0, 0.7, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![5, 5, 5]); // 7 * 0.7 = 4.9 -> 5
        assert!(!out.calculation_notes.contains("activities"));
    }

    #[test]
    fn quantity_is_capped_at_twenty() {
        // 30 days * 1.5 * 2.0 = 90 -> capped base 20, then 20 * 3.0 = 60
        let specs = vec![auto(); 3];
        let out = QuantityCalculation
            .calculate(&input(30, 1.5, 2.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![60, 60, 60]);
    }

    #[test]
    fn quantity_never_drops_below_one() {
        // 1 day * 0.5 * 0.7 = 0.35 -> rounds to 0 -> clamped to 1
        let specs = vec![auto(); 3];
        let out = QuantityCalculation
            .calculate(&input(1, 0.5, 0.7, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![1, 1, 1]);
    }

    #[test]
    fn fixed_items_keep_their_quantity() {
        // A toothbrush stays at 1 regardless of trip parameters
        let specs = vec![
            auto(),
            ItemQuantitySpec::Fixed(1),
            ItemQuantitySpec::Fixed(3),
        ];
        let out = QuantityCalculation
            .calculate(&input(14, 1.5, 1.8, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![38, 1, 3]); // 14 * 1.5 * 1.8 = 37.8 -> 38
    }

    #[test]
    fn output_length_matches_item_count() {
        let specs = vec![auto(); 3];
        let out = QuantityCalculation
            .calculate(&input(7, 1.0, 1.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities.len(), 3);
    }

    #[test]
    fn per_item_rate_scales_the_amount() {
        // 5 days, standard, mixed: rate 1.0 -> 5; rate 0.15 -> 0.75 -> 1
        let specs = vec![
            ItemQuantitySpec::Auto {
                rate: 1.0,
                buffer: QuantityBuffer::None,
            },
            ItemQuantitySpec::Auto {
                rate: 0.15,
                buffer: QuantityBuffer::None,
            },
        ];
        let out = QuantityCalculation
            .calculate(&input(5, 1.0, 1.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![5, 1]);
    }

    #[test]
    fn reserve_adds_flat_on_top() {
        // Underwear: 5 days rate 1.0 + reserve 2 -> 7 (the user's example)
        let specs = vec![ItemQuantitySpec::Auto {
            rate: 1.0,
            buffer: QuantityBuffer::Reserve(2),
        }];
        let out = QuantityCalculation
            .calculate(&input(5, 1.0, 1.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![7]);
    }

    #[test]
    fn multiplier_scales_the_amount() {
        // Socks: 5 days rate 1.0 x 1.25 -> 6.25 -> 6
        let specs = vec![ItemQuantitySpec::Auto {
            rate: 1.0,
            buffer: QuantityBuffer::Multiplier(1.25),
        }];
        let out = QuantityCalculation
            .calculate(&input(5, 1.0, 1.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![6]);
    }

    #[test]
    fn reserve_is_invariant_under_trip_multipliers() {
        // SC-006: changing style/weather never changes the reserve's
        // contribution — only the rate-based part moves
        let mk = || {
            vec![ItemQuantitySpec::Auto {
                rate: 1.0,
                buffer: QuantityBuffer::Reserve(2),
            }]
        };
        let plain = QuantityCalculation
            .calculate(&input(7, 1.0, 1.0, mk()))
            .unwrap()
            .item_quantities[0];
        let styled = QuantityCalculation
            .calculate(&input(7, 1.5, 1.0, mk()))
            .unwrap()
            .item_quantities[0];
        // 7 vs 10.5->11: the difference is exactly the rate part's
        // change (7->11), the reserve (+2) is inside both
        assert_eq!(plain, 9); // 7 + 2
        assert_eq!(styled, 13); // 11 + 2
    }

    #[test]
    fn total_is_capped_at_99() {
        // EC-003: rate 10 on a 30-day trip -> 300 -> capped 99
        let specs = vec![ItemQuantitySpec::Auto {
            rate: 10.0,
            buffer: QuantityBuffer::None,
        }];
        let out = QuantityCalculation
            .calculate(&input(30, 1.0, 1.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![99]);
    }

    #[test]
    fn minimum_is_one_even_with_tiny_rate() {
        // EC-002: rate 0.05 on 1 day -> 0.05 -> clamped 1
        let specs = vec![ItemQuantitySpec::Auto {
            rate: 0.05,
            buffer: QuantityBuffer::None,
        }];
        let out = QuantityCalculation
            .calculate(&input(1, 1.0, 1.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![1]);
    }

    #[test]
    fn defaults_equal_the_old_rule() {
        // SC-002/SC-005: rate 1.0 + None reproduces the pre-010 rule
        // exactly: round(min(days,20) * style * weather), min 1
        for (days, style, weather) in [(7, 1.0, 1.5), (30, 1.5, 2.0), (1, 0.5, 0.7), (14, 1.5, 1.8)]
        {
            let old_rule = ((days.min(20) as f32) * style * weather).round().max(1.0) as u32;
            let out = QuantityCalculation
                .calculate(&input(days, style, weather, vec![auto()]))
                .unwrap();
            assert_eq!(out.item_quantities[0], old_rule);
        }
    }

    #[test]
    fn fixed_items_ignore_their_modifiers() {
        // FR-005 (spec 010): fixed items never recalculate; their stored
        // rate/buffer are inactive while fixed
        let specs = vec![
            ItemQuantitySpec::Fixed(1),
            ItemQuantitySpec::Auto {
                rate: 1.0,
                buffer: QuantityBuffer::None,
            },
        ];
        let out = QuantityCalculation
            .calculate(&input(30, 2.0, 2.0, specs))
            .unwrap();
        assert_eq!(out.item_quantities, vec![1, 80]); // fixed stays 1; auto: 20x2x2
    }
}
