use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Activity {
    Hiking,
    Beach,
    CityTour,
    Business,
    Skiing,
    Snowboarding,
    Camping,
}

impl std::str::FromStr for Activity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hiking" => Ok(Self::Hiking),
            "beach" => Ok(Self::Beach),
            "citytour" | "city tour" => Ok(Self::CityTour),
            "business" => Ok(Self::Business),
            "skiing" => Ok(Self::Skiing),
            "snowboarding" => Ok(Self::Snowboarding),
            "camping" => Ok(Self::Camping),
            _ => Err(format!("Invalid activity: {}", s)),
        }
    }
}

impl Default for Activity {
    fn default() -> Self {
        Self::CityTour
    }
}

impl Activity {
    pub fn gear_multiplier(&self) -> f32 {
        match self {
            Activity::Hiking => 1.5,
            Activity::Beach => 0.8,
            Activity::CityTour => 1.0,
            Activity::Business => 1.2,
            Activity::Skiing => 2.0,
            Activity::Snowboarding => 2.0,
            Activity::Camping => 1.8,
        }
    }
}
