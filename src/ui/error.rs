use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("Failed to initialize terminal: {0}")]
    TerminalInitializationFailed(String),
    #[error("Failed to restore terminal: {0}")]
    TerminalRestoreFailed(String),
    #[error("Failed to draw UI: {0}")]
    DrawFailed(String),
    #[error("Failed to handle input: {0}")]
    InputHandlingFailed(String),
}
