/// JSON-backed settings repository for application-level configuration.
///
/// Storage: `%APPDATA%\com.kairo.localruntimemanager\settings.json`
/// Always isolated from `services.json`.
///
/// Uses atomic write (`.tmp` + rename) for crash safety.
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::error::AppError;
use crate::models::AppSettings;
use crate::repository::json_repository::JsonServiceRepository;

pub struct JsonSettingsRepository {
    storage_path: PathBuf,
    lock: RwLock<()>,
}

impl JsonSettingsRepository {
    /// Creates a repository at a custom file path (primarily for tests).
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            storage_path: path.as_ref().to_path_buf(),
            lock: RwLock::new(()),
        }
    }

    /// Default path: `%APPDATA%\com.kairo.localruntimemanager\settings.json`
    /// Migrates existing settings from legacy `%APPDATA%\com.localservicemanager.app\settings.json` if present.
    pub fn default_path() -> PathBuf {
        let base_dir = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        let kairo_dir = base_dir.join("com.kairo.localruntimemanager");
        let kairo_path = kairo_dir.join("settings.json");

        if !kairo_path.exists() {
            let legacy_path = base_dir
                .join("com.localservicemanager.app")
                .join("settings.json");
            if legacy_path.exists() {
                let _ = fs::create_dir_all(&kairo_dir);
                let _ = fs::copy(&legacy_path, &kairo_path);
            }
        }

        kairo_path
    }

    /// Creates an instance using the standard application data path.
    pub fn default() -> Self {
        Self::new(Self::default_path())
    }

    /// Reads settings from disk, returning defaults if the file does not exist.
    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let _guard = self.lock.read().map_err(|_| {
            AppError::Repository("Settings lock poisoned (read)".into())
        })?;

        if !self.storage_path.exists() {
            return Ok(AppSettings::default());
        }

        let content = fs::read_to_string(&self.storage_path).map_err(|e| {
            AppError::Repository(format!("Failed to read settings.json: {}", e))
        })?;

        serde_json::from_str::<AppSettings>(&content).map_err(|e| {
            AppError::Repository(format!(
                "settings.json is malformed ({}). Using defaults.",
                e
            ))
        })
    }

    /// Atomically persists `AppSettings` to disk.
    pub fn update_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        let _guard = self.lock.write().map_err(|_| {
            AppError::Repository("Settings lock poisoned (write)".into())
        })?;

        // Ensure parent directory exists.
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Repository(format!("Failed to create settings directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(settings).map_err(|e| {
            AppError::Repository(format!("Failed to serialize settings: {}", e))
        })?;

        // Atomic write: write to `.tmp` then rename.
        let tmp_path = self.storage_path.with_extension("json.tmp");
        fs::write(&tmp_path, &json).map_err(|e| {
            AppError::Repository(format!("Failed to write settings.json.tmp: {}", e))
        })?;
        fs::rename(&tmp_path, &self.storage_path).map_err(|e| {
            AppError::Repository(format!("Failed to finalize settings.json: {}", e))
        })?;

        Ok(())
    }
}

// Convenience method alias to share the timestamp utility.
impl JsonSettingsRepository {
    pub fn current_timestamp() -> String {
        JsonServiceRepository::current_timestamp()
    }
}
