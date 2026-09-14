//! Phase 12 — Tauri IPC commands exposing updater state to the frontend.
//!
//! The actual update check, download, signature verification, and install are
//! performed exclusively by the official `tauri-plugin-updater` plugin against
//! the official KAIRO GitHub Releases manifest. These commands only expose
//! version/architecture facts and a clean restart hook.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::utils::update_config;

/// Runtime flag set during plugin registration. Keeps `get_updater_status`
/// safe to call even if the updater plugin could not be registered.
static UPDATER_PLUGIN_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Records whether the updater plugin was registered successfully.
pub fn mark_updater_plugin_active(active: bool) {
    UPDATER_PLUGIN_ACTIVE.store(active, Ordering::SeqCst);
}

/// Aggregated updater facts for the Settings UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterStatus {
    /// Installed application version (from the compiled Tauri package info).
    pub app_version: String,
    /// Updater platform target, e.g. `windows-x86_64` / `windows-i686`.
    pub updater_target: Option<String>,
    /// Architecture of the running binary.
    pub arch: String,
    /// `true` when a production signing public key is configured, meaning the
    /// update channel is actually usable in this build.
    pub signer_configured: bool,
}

/// Returns current application version, architecture, and updater readiness.
#[tauri::command]
pub fn get_updater_status(app: AppHandle) -> UpdaterStatus {
    let signer_configured = UPDATER_PLUGIN_ACTIVE.load(Ordering::SeqCst) && app.updater().is_ok();

    UpdaterStatus {
        app_version: app.package_info().version.to_string(),
        updater_target: tauri_plugin_updater::target(),
        arch: update_config::detect_arch().to_string(),
        signer_configured,
    }
}

/// Restarts KAIRO cleanly (used only as a post-update fallback — on Windows
/// the NSIS updater automatically restarts the app after installation).
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    let _ = app.restart();
}
