use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WeatherInput {
    Hot,
    Cold,
    Rainy,
    Snowy,
    Mixed,
}

impl std::str::FromStr for WeatherInput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hot" => Ok(Self::Hot),
            "cold" => Ok(Self::Cold),
            "rainy" => Ok(Self::Rainy),
            "snowy" => Ok(Self::Snowy),
            "mixed" => Ok(Self::Mixed),
            _ => Err(format!("Invalid weather input: {}", s)),
        }
    }
}

impl Default for WeatherInput {
    fn default() -> Self {
        Self::Mixed
    }
}

impl WeatherInput {
    pub fn clothing_multiplier(&self) -> f32 {
        match self {
            WeatherInput::Hot => 0.7,
            WeatherInput::Cold => 1.5,
            WeatherInput::Rainy => 1.2,
            WeatherInput::Snowy => 1.8,
            WeatherInput::Mixed => 1.0,
        }
    }
}
