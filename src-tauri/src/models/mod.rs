use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::AppError;

pub mod log;
pub mod process;
pub mod settings;
pub use log::{LogEntry, LogStream};
pub use process::{ProcessRuntimeInfo, ProcessState};
pub use settings::AppSettings;

/// Runtime lifecycle status of a managed service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Ready,
    Stopping,
    Failed,
    Unknown,
}

/// Generic configuration for arbitrary executable-based local services.
/// Conforms precisely to Phase 2 Service model specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: String,
    pub environment_variables: HashMap<String, String>,
    pub port: Option<u16>,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub health_check: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ServiceConfig {
    /// Validates all fields according to Phase 2 requirements.
    pub fn validate(&self) -> Result<(), AppError> {
        let name_trimmed = self.name.trim();
        if name_trimmed.is_empty() {
            return Err(AppError::Validation("Service name is required".into()));
        }
        if name_trimmed.len() > 100 {
            return Err(AppError::Validation("Service name cannot exceed 100 characters".into()));
        }

        let exec_trimmed = self.executable.trim();
        if exec_trimmed.is_empty() {
            return Err(AppError::Validation("Executable path is required".into()));
        }
        if exec_trimmed.len() > 1024 {
            return Err(AppError::Validation("Executable path cannot exceed 1024 characters".into()));
        }

        if self.description.len() > 500 {
            return Err(AppError::Validation("Description cannot exceed 500 characters".into()));
        }

        if self.working_directory.len() > 1024 {
            return Err(AppError::Validation("Working directory cannot exceed 1024 characters".into()));
        }

        if let Some(port) = self.port {
            if port == 0 {
                return Err(AppError::Validation("Port must be between 1 and 65535".into()));
            }
        }

        if self.arguments.len() > 100 {
            return Err(AppError::Validation("Cannot exceed 100 arguments".into()));
        }

        for arg in &self.arguments {
            if arg.len() > 2048 {
                return Err(AppError::Validation("Argument cannot exceed 2048 characters".into()));
            }
        }

        if self.environment_variables.len() > 100 {
            return Err(AppError::Validation("Cannot exceed 100 environment variables".into()));
        }

        for (k, v) in &self.environment_variables {
            if k.trim().is_empty() {
                return Err(AppError::Validation("Environment variable key cannot be empty".into()));
            }
            if k.len() > 256 || v.len() > 4096 {
                return Err(AppError::Validation("Environment variable length limit exceeded".into()));
            }
        }

        Ok(())
    }
}

/// Data transfer object for creating a new service.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServiceDto {
    pub name: String,
    pub description: Option<String>,
    pub executable: String,
    pub arguments: Option<Vec<String>>,
    pub working_directory: Option<String>,
    pub environment_variables: Option<HashMap<String, String>>,
    pub port: Option<u16>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub health_check: Option<String>,
}

/// Data transfer object for updating an existing service.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServiceDto {
    pub name: String,
    pub description: Option<String>,
    pub executable: String,
    pub arguments: Option<Vec<String>>,
    pub working_directory: Option<String>,
    pub environment_variables: Option<HashMap<String, String>>,
    pub port: Option<u16>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub health_check: Option<String>,
}

/// Dynamic metrics captured from a running service process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub pid: Option<u32>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub uptime_seconds: Option<u64>,
}

/// Aggregated state combining configuration and runtime telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceState {
    pub config: ServiceConfig,
    pub status: ServiceStatus,
    pub metrics: Option<ProcessMetrics>,
    pub last_started_at: Option<String>,
    pub last_exited_at: Option<String>,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
}
