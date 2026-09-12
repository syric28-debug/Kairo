use serde::{Deserialize, Serialize};

/// Runtime lifecycle state of a managed child process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessState {
    Stopped,
    Starting,
    Running,
    Ready,
    Stopping,
    Failed,
    Unknown,
}

impl Default for ProcessState {
    fn default() -> Self {
        ProcessState::Stopped
    }
}

/// Runtime snapshot of a managed service process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRuntimeInfo {
    pub service_id: String,
    pub pid: Option<u32>,
    pub state: ProcessState,
    pub started_at: Option<String>,
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port_listening: Option<bool>,
}

impl ProcessRuntimeInfo {
    pub fn stopped(service_id: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            pid: None,
            state: ProcessState::Stopped,
            started_at: None,
            error_message: None,
            port: None,
            port_listening: None,
        }
    }

    pub fn starting(service_id: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            pid: None,
            state: ProcessState::Starting,
            started_at: None,
            error_message: None,
            port: None,
            port_listening: None,
        }
    }

    pub fn running(service_id: impl Into<String>, pid: u32, started_at: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            pid: Some(pid),
            state: ProcessState::Running,
            started_at: Some(started_at.into()),
            error_message: None,
            port: None,
            port_listening: None,
        }
    }

    pub fn ready(
        service_id: impl Into<String>,
        pid: u32,
        started_at: impl Into<String>,
        port: u16,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            pid: Some(pid),
            state: ProcessState::Ready,
            started_at: Some(started_at.into()),
            error_message: None,
            port: Some(port),
            port_listening: Some(true),
        }
    }

    pub fn failed(service_id: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            pid: None,
            state: ProcessState::Failed,
            started_at: None,
            error_message: Some(error_message.into()),
            port: None,
            port_listening: None,
        }
    }

    pub fn with_port(mut self, port: Option<u16>, port_listening: Option<bool>) -> Self {
        self.port = port;
        self.port_listening = port_listening;
        self
    }
}
