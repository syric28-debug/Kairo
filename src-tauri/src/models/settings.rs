use serde::{Deserialize, Serialize};

/// Application-level settings persisted independently of service configurations.
///
/// Stored at `%APPDATA%\com.kairo.localruntimemanager\settings.json`.
/// Always separate from `services.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Whether the application is registered to start with Windows (HKCU\Run).
    /// This field always reflects actual registry state when queried via IPC.
    pub app_auto_start: bool,

    /// Whether eligible services (those with `autoStart: true`) should be launched
    /// automatically after the application starts. Acts as a global master switch.
    pub service_auto_start: bool,

    /// Whether KAIRO should automatically check the official GitHub Releases
    /// channel for updates on startup. Defaults to `true`; `#[serde(default)]`
    /// keeps existing settings.json files loading unchanged.
    #[serde(default = "default_update_auto_check")]
    pub update_auto_check: bool,

    /// ISO 8601 timestamp of the last settings write.
    pub updated_at: String,
}

fn default_update_auto_check() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_auto_start: false,
            service_auto_start: false,
            update_auto_check: default_update_auto_check(),
            updated_at: String::new(),
        }
    }
}
