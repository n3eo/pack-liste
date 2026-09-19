use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    #[error("Configuration directory does not exist: {0}")]
    DirectoryNotFound(String),
    #[error("Failed to create configuration directory: {0}")]
    DirectoryCreationFailed(String),
}
