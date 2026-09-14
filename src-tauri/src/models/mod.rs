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
    /// Optional user-defined API base path (e.g. "/v1") used to build a
    /// convenient local API Base URL. KAIRO never guesses API routes.
    #[serde(default)]
    pub api_base_path: Option<String>,
    /// Optional user-defined direct link path (e.g. "/") used to build a
    /// Direct Link URL for this service.
    #[serde(default)]
    pub direct_url_path: Option<String>,
    /// Optional user-defined health check path (e.g. "/health") used to
    /// build a Health URL for this service.
    #[serde(default)]
    pub health_check_path: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub kind: Option<String>,

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

        // Optional endpoint path fields must be plain URL path components,
        // never full URLs or protocol-bearing strings (prevents protocol
        // injection such as "http://evil" or "javascript:alert(1)").
        for (field, value) in [
            ("API base path", &self.api_base_path),
            ("Direct URL path", &self.direct_url_path),
            ("Health check path", &self.health_check_path),
        ] {
            if let Some(path) = value {
                Self::validate_endpoint_path(field, path)?;
            }
        }

        Ok(())
    }

    /// Validates a single optional endpoint path configuration field.
    ///
    /// Rules:
    /// - must not be empty/whitespace-only
    /// - must not contain a protocol separator ("://")
    /// - must not contain whitespace or control characters
    /// - must not exceed 512 characters
    fn validate_endpoint_path(field: &str, path: &str) -> Result<(), AppError> {
        if path.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "{} cannot be empty (leave it blank to disable)",
                field
            )));
        }
        if path.len() > 512 {
            return Err(AppError::Validation(format!(
                "{} cannot exceed 512 characters",
                field
            )));
        }
        if path.contains("://") {
            return Err(AppError::Validation(format!(
                "{} must be a path component (e.g. /v1), not a full URL",
                field
            )));
        }
        if path
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(AppError::Validation(format!(
                "{} cannot contain whitespace or control characters",
                field
            )));
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
    #[serde(default)]
    pub api_base_path: Option<String>,
    #[serde(default)]
    pub direct_url_path: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,

    #[serde(default)]
    pub health_check_path: Option<String>,
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
    #[serde(default)]
    pub kind: Option<String>,

    pub api_base_path: Option<String>,
    #[serde(default)]
    pub direct_url_path: Option<String>,
    #[serde(default)]
    pub health_check_path: Option<String>,
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

#[cfg(test)]
mod endpoint_field_tests {
    use super::*;

    fn config_from_json(json: &str) -> ServiceConfig {
        serde_json::from_str(json).expect("legacy JSON must deserialize")
    }

    /// Existing services.json files (pre-endpoint-fields) must continue loading.
    #[test]
    fn legacy_service_without_endpoint_fields_loads() {
        let legacy = r#"{
            "id": "svc-1",
            "name": "My Local API",
            "description": "test",
            "executable": "C:\\tools\\api.exe",
            "arguments": ["--port", "8080"],
            "workingDirectory": "C:\\tools",
            "environmentVariables": {},
            "port": 8080,
            "autoStart": false,
            "autoRestart": false,
            "healthCheck": null,
            "createdAt": "2025-01-01T00:00:00Z",
            "updatedAt": "2025-01-01T00:00:00Z"
        }"#;
        let cfg = config_from_json(legacy);
        assert_eq!(cfg.port, Some(8080));
        assert!(cfg.api_base_path.is_none());
        assert!(cfg.direct_url_path.is_none());
        assert!(cfg.health_check_path.is_none());
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn service_with_endpoint_fields_loads() {
        let json = r#"{
            "id": "svc-2",
            "name": "My Local API",
            "description": "",
            "executable": "api.exe",
            "arguments": [],
            "workingDirectory": "C:\\",
            "environmentVariables": {},
            "port": 8080,
            "autoStart": false,
            "autoRestart": false,
            "healthCheck": null,
            "apiBasePath": "/v1",
            "directUrlPath": "/",
            "healthCheckPath": "/health",
            "createdAt": "2025-01-01T00:00:00Z",
            "updatedAt": "2025-01-01T00:00:00Z"
        }"#;
        let cfg = config_from_json(json);
        assert_eq!(cfg.api_base_path.as_deref(), Some("/v1"));
        assert_eq!(cfg.direct_url_path.as_deref(), Some("/"));
        assert_eq!(cfg.health_check_path.as_deref(), Some("/health"));
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn protocol_injection_in_path_rejected() {
        let mut cfg = valid_base();
        cfg.api_base_path = Some("http://evil.example/steal".into());
        assert!(cfg.validate().is_err(), ":// in path must be rejected");
    }

    #[test]
    fn whitespace_in_path_rejected() {
        let mut cfg = valid_base();
        cfg.direct_url_path = Some("/v1 chat/completions".into());
        assert!(cfg.validate().is_err(), "whitespace in path must be rejected");
    }

    #[test]
    fn empty_path_rejected() {
        let mut cfg = valid_base();
        cfg.health_check_path = Some("   ".into());
        assert!(cfg.validate().is_err(), "empty path must be rejected");
    }

    #[test]
    fn oversized_path_rejected() {
        let mut cfg = valid_base();
        cfg.api_base_path = Some(format!("/{}", "a".repeat(513)));
        assert!(cfg.validate().is_err(), "over-length path must be rejected");
    }

    #[test]
    fn valid_paths_accepted() {
        let mut cfg = valid_base();
        cfg.api_base_path = Some("/v1".into());
        cfg.direct_url_path = Some("/".into());
        cfg.health_check_path = Some("/health".into());
        assert!(cfg.validate().is_ok());
    }

    fn valid_base() -> ServiceConfig {
        config_from_json(
            r#"{
            "id": "svc-3",
            "name": "Base",
            "description": "",
            "executable": "a.exe",
            "arguments": [],
            "workingDirectory": "C:\\\\",
            "environmentVariables": {},
            "port": 8080,
            "autoStart": false,
            "autoRestart": false,
            "healthCheck": null,
            "createdAt": "2025-01-01T00:00:00Z",
            "updatedAt": "2025-01-01T00:00:00Z"
        }"#,
        )
    }
}
