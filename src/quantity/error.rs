use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantityError {
    #[error("Invalid weekdays value: {0}")]
    InvalidWeekdays(u32),
    #[error("Invalid travel style: {0}")]
    InvalidTravelStyle(String),
    #[error("Invalid weather input: {0}")]
    InvalidWeatherInput(String),
    #[error("Invalid activity: {0}")]
    InvalidActivity(String),
    #[error("Calculation overflow: {0}")]
    CalculationOverflow(String),
}
