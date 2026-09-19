use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TravelStyle {
    Minimalist,
    Standard,
    Luxury,
    Backpacker,
}

impl std::str::FromStr for TravelStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimalist" => Ok(Self::Minimalist),
            "standard" => Ok(Self::Standard),
            "luxury" => Ok(Self::Luxury),
            "backpacker" => Ok(Self::Backpacker),
            _ => Err(format!("Invalid travel style: {}", s)),
        }
    }
}

impl Default for TravelStyle {
    fn default() -> Self {
        Self::Standard
    }
}

impl TravelStyle {
    pub fn multiplier(&self) -> f32 {
        match self {
            TravelStyle::Minimalist => 0.5,
            TravelStyle::Standard => 1.0,
            TravelStyle::Luxury => 1.5,
            TravelStyle::Backpacker => 1.2,
        }
    }
}
