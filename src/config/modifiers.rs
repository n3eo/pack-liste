use serde::{Deserialize, Serialize};

/// A named multiplier used by the quantity calculation, e.g. a travel
/// style or a weather condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ModifierConfig {
    pub(crate) name: String,
    pub(crate) multiplier: f32,
}

/// Configuration of the quantity calculation modifiers, loaded from the
/// config file. All fields default to the built-in values so older config
/// files without these sections keep working.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ModifiersConfig {
    #[serde(default = "default_travel_styles")]
    pub(crate) travel_styles: Vec<ModifierConfig>,
    #[serde(default = "default_weather")]
    pub(crate) weather: Vec<ModifierConfig>,
}

impl Default for ModifiersConfig {
    fn default() -> Self {
        Self {
            travel_styles: default_travel_styles(),
            weather: default_weather(),
        }
    }
}

impl ModifiersConfig {
    pub(crate) fn travel_style_multiplier(&self, name: &str) -> Option<f32> {
        self.travel_styles
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.multiplier)
    }

    pub(crate) fn weather_multiplier(&self, name: &str) -> Option<f32> {
        self.weather
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.multiplier)
    }
}

fn modifier(name: &str, multiplier: f32) -> ModifierConfig {
    ModifierConfig {
        name: name.to_string(),
        multiplier,
    }
}

fn default_travel_styles() -> Vec<ModifierConfig> {
    vec![
        modifier("minimalist", 0.5),
        modifier("standard", 1.0),
        modifier("luxury", 1.5),
        modifier("backpacker", 1.2),
    ]
}

fn default_weather() -> Vec<ModifierConfig> {
    vec![
        modifier("hot", 0.7),
        modifier("mixed", 1.0),
        modifier("rainy", 1.2),
        modifier("cold", 1.5),
        modifier("snowy", 1.8),
    ]
}
