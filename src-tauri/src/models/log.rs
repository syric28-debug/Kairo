use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository::json_repository::JsonServiceRepository;

/// Designates whether a log entry originated from stdout or stderr.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// Represents an individual structured log message emitted by a managed service process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: String,
    #[serde(alias = "service_id")]
    pub service_id: String,
    pub timestamp: String,
    pub stream: LogStream,
    pub message: String,
}

impl LogEntry {
    /// Creates a new `LogEntry` with a generated UUID and current ISO-8601 UTC timestamp.
    pub fn new(service_id: impl Into<String>, stream: LogStream, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.into(),
            timestamp: JsonServiceRepository::current_timestamp(),
            stream,
            message: message.into(),
        }
    }

    /// Creates a new `LogEntry` with explicit parameters (useful in tests and deterministic scenarios).
    pub fn with_id_and_timestamp(
        id: impl Into<String>,
        service_id: impl Into<String>,
        timestamp: impl Into<String>,
        stream: LogStream,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            service_id: service_id.into(),
            timestamp: timestamp.into(),
            stream,
            message: message.into(),
        }
    }
}
