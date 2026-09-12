use serde::Serialize;
use thiserror::Error;

/// Comprehensive error type for KAIRO.
/// Implements `Serialize` so Rust errors can be returned directly to the frontend over Tauri IPC.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
