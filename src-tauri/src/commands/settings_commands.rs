/// Tauri IPC commands for reading and updating application startup settings.
use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::models::AppSettings;
use crate::repository::JsonSettingsRepository;
use crate::utils::windows_startup;

/// Returns the current `AppSettings`, with `appAutoStart` reflecting
/// the live registry state (not only what is stored in settings.json).
#[tauri::command]
pub fn get_startup_settings(
    settings_repo: State<'_, Arc<JsonSettingsRepository>>,
) -> Result<AppSettings, AppError> {
    let mut settings = settings_repo.get_settings()?;

    // Sync `appAutoStart` with actual registry state so the UI always shows truth.
    settings.app_auto_start = windows_startup::is_app_auto_start_enabled().unwrap_or(false);

    Ok(settings)
}

/// Enables or disables application Windows logon registration.
/// Updates both the registry (HKCU\Run) and settings.json.
#[tauri::command]
pub fn set_app_auto_start(
    enabled: bool,
    settings_repo: State<'_, Arc<JsonSettingsRepository>>,
) -> Result<AppSettings, AppError> {
    if enabled {
        windows_startup::enable_app_auto_start()?;
    } else {
        windows_startup::disable_app_auto_start()?;
    }

    // Persist the new value to settings.json as well.
    let mut settings = settings_repo.get_settings().unwrap_or_default();
    settings.app_auto_start = enabled;
    settings.updated_at = JsonSettingsRepository::current_timestamp();
    settings_repo.update_settings(&settings)?;

    // Return with live registry state.
    settings.app_auto_start = windows_startup::is_app_auto_start_enabled().unwrap_or(false);
    Ok(settings)
}

/// Enables or disables the global service auto-start master switch.
/// Does not touch the Windows registry.
#[tauri::command]
pub fn set_service_auto_start(
    enabled: bool,
    settings_repo: State<'_, Arc<JsonSettingsRepository>>,
) -> Result<AppSettings, AppError> {
    let mut settings = settings_repo.get_settings().unwrap_or_default();
    settings.service_auto_start = enabled;
    settings.updated_at = JsonSettingsRepository::current_timestamp();
    settings_repo.update_settings(&settings)?;

    // Sync appAutoStart from registry before returning.
    settings.app_auto_start = windows_startup::is_app_auto_start_enabled().unwrap_or(false);
    Ok(settings)
}
