use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
/// The trip length in days; validated to the 1..=30 range.
pub(crate) struct Weekdays(u32);

impl Weekdays {
    pub(crate) const MIN: u32 = 1;
    pub(crate) const MAX: u32 = 30;

    /// Create a weekdays value, rejecting anything outside 1..=30.
    pub(crate) fn new(value: u32) -> Result<Self, WeekdaysError> {
        if value < Self::MIN {
            return Err(WeekdaysError::TooFewDays(Self::MIN, value));
        }
        if value > Self::MAX {
            return Err(WeekdaysError::TooManyDays(Self::MAX, value));
        }
        Ok(Self(value))
    }

    /// The number of days.
    pub(crate) fn get(&self) -> u32 {
        self.0
    }
}

impl Default for Weekdays {
    fn default() -> Self {
        Self(7)
    }
}

#[derive(Error, Debug)]
/// A trip length outside the valid range.
pub(crate) enum WeekdaysError {
    #[error("Weekdays must be at least {0}, got {1}")]
    TooFewDays(u32, u32),
    #[error("Weekdays must be at most {0}, got {1}")]
    TooManyDays(u32, u32),
}
