/// StartupOrchestrator — service auto-start on application boot.
///
/// Executes exactly once per application session (guarded by `AtomicBool`).
/// Reads the global `serviceAutoStart` setting and each service's `autoStart` flag.
/// Launches eligible services through the existing shared `ProcessManager`.
/// Errors from individual services are isolated — one failure never halts the rest.
/// Emits `service-runtime-updated` and `startup-progress` Tauri events so the UI
/// can reflect live progress without polling.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use tauri::{AppHandle, Emitter};

use crate::models::AppSettings;
use crate::repository::{JsonServiceRepository, JsonSettingsRepository, ServiceRepository};
use crate::services::ProcessManager;

/// Progress payload emitted to the frontend during auto-start sequencing.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupProgress {
    pub total: usize,
    pub started: usize,
    pub failed: usize,
    pub is_complete: bool,
}

/// Single-execution guard owned by Tauri state.
pub struct StartupOrchestrator {
    has_run: Arc<AtomicBool>,
}

impl StartupOrchestrator {
    pub fn new() -> Self {
        Self {
            has_run: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Runs the auto-start sequence once per session.
    ///
    /// Must be called only after `ProcessManager` and `ProcessMonitor` are both
    /// fully initialized and managed in Tauri state (i.e. inside `.setup()`).
    ///
    /// If the global `serviceAutoStart` setting is disabled, emits one
    /// completion event with `total = 0` and returns immediately.
    pub fn run_once(
        &self,
        app_handle: AppHandle,
        process_manager: ProcessManager,
        settings_repo: Arc<JsonSettingsRepository>,
        service_repo: Arc<JsonServiceRepository>,
    ) {
        // Single-execution guard — compare-and-swap false → true.
        if self
            .has_run
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return; // Already ran this session.
        }

        thread::spawn(move || {
            orchestrate(app_handle, process_manager, settings_repo, service_repo);
        });
    }
}

impl Default for StartupOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Core orchestration logic (runs in background thread) ─────────────────

fn orchestrate(
    app_handle: AppHandle,
    process_manager: ProcessManager,
    settings_repo: Arc<JsonSettingsRepository>,
    service_repo: Arc<JsonServiceRepository>,
) {
    // Load settings. If unreadable, skip safely.
    let settings: AppSettings = match settings_repo.get_settings() {
        Ok(s) => s,
        Err(e) => {
            log_startup_error(&app_handle, &format!("Failed to load settings: {}", e));
            emit_complete(&app_handle, 0, 0, 0);
            return;
        }
    };

    if !settings.service_auto_start {
        // Global master switch is off — nothing to start.
        emit_complete(&app_handle, 0, 0, 0);
        return;
    }

    // Load all services; filter to those with autoStart enabled.
    let all_services = match service_repo.list() {
        Ok(s) => s,
        Err(e) => {
            log_startup_error(&app_handle, &format!("Failed to load services: {}", e));
            emit_complete(&app_handle, 0, 0, 0);
            return;
        }
    };

    let eligible: Vec<_> = all_services
        .into_iter()
        .filter(|s| s.auto_start)
        .collect();

    let total = eligible.len();
    if total == 0 {
        emit_complete(&app_handle, 0, 0, 0);
        return;
    }

    let mut started = 0usize;
    let mut failed = 0usize;

    for config in &eligible {
        match process_manager.start_service(config) {
            Ok(info) => {
                started += 1;
                // Notify the frontend that this service is now running.
                let _ = app_handle.emit("service-runtime-updated", &info);
            }
            Err(e) => {
                failed += 1;
                // Emit a Failed state for this service so the UI shows the error.
                let failed_info = crate::models::process::ProcessRuntimeInfo::failed(
                    &config.id,
                    format!("Auto-start failed: {}", e),
                );
                let _ = app_handle.emit("service-runtime-updated", &failed_info);
            }
        }

        // Emit incremental progress after each service attempt.
        let progress = StartupProgress {
            total,
            started,
            failed,
            is_complete: false,
        };
        let _ = app_handle.emit("startup-progress", &progress);
    }

    emit_complete(&app_handle, total, started, failed);
}

fn emit_complete(app_handle: &AppHandle, total: usize, started: usize, failed: usize) {
    let progress = StartupProgress {
        total,
        started,
        failed,
        is_complete: true,
    };
    let _ = app_handle.emit("startup-progress", &progress);
}

fn log_startup_error(app_handle: &AppHandle, msg: &str) {
    eprintln!("[StartupOrchestrator] {}", msg);
    let _ = app_handle.emit(
        "startup-progress",
        StartupProgress {
            total: 0,
            started: 0,
            failed: 0,
            is_complete: true,
        },
    );
}
