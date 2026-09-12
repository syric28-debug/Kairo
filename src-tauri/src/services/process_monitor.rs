use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::services::{PortChecker, ProcessManager};

pub const SERVICE_RUNTIME_UPDATED_EVENT: &str = "service-runtime-updated";
const MONITOR_POLL_INTERVAL_MS: u64 = 1000;
const SLEEP_STEP_MS: u64 = 100;

/// Background watchdog that continuously monitors managed child processes
/// and detects unexpected terminations / crashes and TCP port readiness.
///
/// Principles:
/// - Single global instance managed by application lifecycle
/// - Non-blocking status checking via `try_wait()`
/// - 1000ms wake-up interval for low CPU overhead
/// - Local TCP endpoint checking without holding ProcessManager synchronization lock
/// - Emits `service-runtime-updated` events strictly when meaningful transitions occur
/// - Graceful interruptible shutdown without leaking threads
pub struct ProcessMonitor {
    is_running: Arc<AtomicBool>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
}

impl ProcessMonitor {
    /// Starts the background monitoring loop.
    pub fn start(
        app_handle: AppHandle,
        process_manager: ProcessManager,
        service_repo: Arc<crate::repository::json_repository::JsonServiceRepository>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);

        let thread_handle = thread::Builder::new()
            .name("lsm-process-monitor".to_string())
            .spawn(move || {
                let steps = MONITOR_POLL_INTERVAL_MS / SLEEP_STEP_MS;
                let mut auto_restart_tracker = crate::services::auto_restart::AutoRestartTracker::default();

                while is_running_clone.load(Ordering::Relaxed) {
                    // Interruptible sleep loop
                    for _ in 0..steps {
                        if !is_running_clone.load(Ordering::Relaxed) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(SLEEP_STEP_MS));
                    }

                    if !is_running_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    // 1. Check for unexpected process terminations
                    let unexpected_exits = process_manager.reconcile_unexpected_exits();

                    // Emit events to frontend for each detected crash / exit
                    let has_exits = !unexpected_exits.is_empty();
                    for info in unexpected_exits {
                        // Emit Failed state to frontend immediately
                        if let Err(e) = app_handle.emit(SERVICE_RUNTIME_UPDATED_EVENT, &info) {
                            eprintln!(
                                "ProcessMonitor: Failed to emit {} for service '{}': {}",
                                SERVICE_RUNTIME_UPDATED_EVENT, info.service_id, e
                            );
                        }

                        // Phase 8: Evaluate automatic restart
                        let app_emit_clone = app_handle.clone();
                        auto_restart_tracker.handle_unexpected_exit(
                            &info.service_id,
                            &service_repo,
                            &process_manager,
                            |event_name, runtime_info| {
                                if let Err(e) = app_emit_clone.emit(event_name, runtime_info) {
                                    eprintln!(
                                        "ProcessMonitor: Failed to emit {} during auto-restart for '{}': {}",
                                        event_name, runtime_info.service_id, e
                                    );
                                }
                            },
                        );
                    }

                    if has_exits {
                        crate::utils::tray::update_tray_tooltip(&app_handle, &process_manager);
                    }

                    // 2. Phase 5: Probe port readiness for active processes with configured ports
                    let targets = process_manager.get_monitored_targets();
                    let mut readiness_changed = false;
                    let now_epoch_secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    for target in targets {
                        // Loop protection: reset consecutive restart counts for stable running processes
                        if let Some(start_secs) = target
                            .started_at
                            .split('.')
                            .next()
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            let uptime_secs = now_epoch_secs.saturating_sub(start_secs);
                            auto_restart_tracker.reset_if_stable(
                                &target.service_id,
                                Duration::from_secs(uptime_secs),
                            );
                        }

                        if let Some(port) = target.port {
                            // Check port on loopback without holding any ProcessManager lock
                            let is_listening = PortChecker::check_local_port(port);

                            // Apply state transition if status changed and target PID still matches
                            if let Some(info) = process_manager.update_port_readiness(
                                &target.service_id,
                                target.pid,
                                is_listening,
                            ) {
                                readiness_changed = true;
                                if let Err(e) = app_handle.emit(SERVICE_RUNTIME_UPDATED_EVENT, &info) {
                                    eprintln!(
                                        "ProcessMonitor: Failed to emit {} for readiness change on '{}': {}",
                                        SERVICE_RUNTIME_UPDATED_EVENT, info.service_id, e
                                    );
                                }
                            }
                        }
                    }

                    if readiness_changed {
                        crate::utils::tray::update_tray_tooltip(&app_handle, &process_manager);
                    }
                }
            })
            .expect("Failed to spawn process monitor background thread");

        Self {
            is_running,
            thread_handle: Mutex::new(Some(thread_handle)),
        }
    }

    /// Stops the monitoring loop and joins the worker thread.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Ok(mut handle_guard) = self.thread_handle.lock() {
            if let Some(handle) = handle_guard.take() {
                let _ = handle.join();
            }
        }
    }

    /// Checks whether the background monitor is currently running.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

impl Drop for ProcessMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}
