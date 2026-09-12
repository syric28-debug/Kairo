use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::models::process::ProcessRuntimeInfo;
use crate::repository::json_repository::JsonServiceRepository;
use crate::repository::ServiceRepository;
use crate::services::process_manager::ProcessManager;
use crate::services::process_monitor::SERVICE_RUNTIME_UPDATED_EVENT;

pub const DEFAULT_MAX_CONSECUTIVE_ATTEMPTS: u32 = 3;
pub const DEFAULT_COOLDOWN_WINDOW_SECS: u64 = 30;
pub const DEFAULT_STABLE_DURATION_SECS: u64 = 10;

/// Records restart statistics for a single service to enforce loop protection.
#[derive(Debug, Clone)]
pub struct AutoRestartRecord {
    pub consecutive_attempts: u32,
    pub last_attempt: Instant,
}

/// Tracks and rate-limits automatic restart attempts per service.
/// Prevents uncontrolled rapid restart loops when a service repeatedly crashes.
pub struct AutoRestartTracker {
    records: HashMap<String, AutoRestartRecord>,
    max_consecutive_attempts: u32,
    cooldown_window: Duration,
    stable_duration: Duration,
}

impl Default for AutoRestartTracker {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_CONSECUTIVE_ATTEMPTS,
            Duration::from_secs(DEFAULT_COOLDOWN_WINDOW_SECS),
            Duration::from_secs(DEFAULT_STABLE_DURATION_SECS),
        )
    }
}

impl AutoRestartTracker {
    pub fn new(
        max_consecutive_attempts: u32,
        cooldown_window: Duration,
        stable_duration: Duration,
    ) -> Self {
        Self {
            records: HashMap::new(),
            max_consecutive_attempts,
            cooldown_window,
            stable_duration,
        }
    }

    /// Checks if a service is allowed to attempt an automatic restart.
    /// If the cooldown window has elapsed since the last attempt, consecutive attempts reset.
    pub fn can_restart(&mut self, service_id: &str) -> bool {
        if let Some(record) = self.records.get_mut(service_id) {
            if record.last_attempt.elapsed() > self.cooldown_window {
                record.consecutive_attempts = 0;
            }
            record.consecutive_attempts < self.max_consecutive_attempts
        } else {
            true
        }
    }

    /// Records that an automatic restart attempt has been initiated.
    pub fn record_attempt(&mut self, service_id: &str) {
        let entry = self
            .records
            .entry(service_id.to_string())
            .or_insert_with(|| AutoRestartRecord {
                consecutive_attempts: 0,
                last_attempt: Instant::now(),
            });
        entry.consecutive_attempts += 1;
        entry.last_attempt = Instant::now();
    }

    /// Resets the consecutive restart count for a service.
    pub fn reset(&mut self, service_id: &str) {
        self.records.remove(service_id);
    }

    /// If a service has been continuously running for at least `stable_duration`,
    /// clears consecutive failure tracking so future unexpected crashes get fresh restart attempts.
    pub fn reset_if_stable(&mut self, service_id: &str, uptime: Duration) {
        if uptime >= self.stable_duration {
            if let Some(record) = self.records.get_mut(service_id) {
                if record.consecutive_attempts > 0 {
                    record.consecutive_attempts = 0;
                }
            }
        }
    }

    /// Returns the current consecutive attempt count for a service.
    pub fn get_consecutive_attempts(&self, service_id: &str) -> u32 {
        self.records
            .get(service_id)
            .map(|r| r.consecutive_attempts)
            .unwrap_or(0)
    }

    /// Evaluates an unexpected exit for auto-restart eligibility, enforces loop protection,
    /// and invokes the existing `ProcessManager.start_service()` pathway.
    ///
    /// Lifecycle transition:
    /// - Emits `Starting` event.
    /// - Invokes `ProcessManager.start_service()`.
    /// - Emits `Running` event on success, or `Failed` on failure.
    pub fn handle_unexpected_exit<F>(
        &mut self,
        service_id: &str,
        service_repo: &JsonServiceRepository,
        process_manager: &ProcessManager,
        mut emit_fn: F,
    ) -> Option<Result<ProcessRuntimeInfo, AppError>>
    where
        F: FnMut(&str, &ProcessRuntimeInfo),
    {
        // 1. Retrieve the service's current configuration
        let config = match service_repo.get(service_id) {
            Ok(Some(c)) => c,
            Ok(None) => return None,
            Err(e) => {
                eprintln!(
                    "AutoRestart: Failed to load configuration for service '{}': {}",
                    service_id, e
                );
                return None;
            }
        };

        // 2. Check if auto_restart is enabled on the configuration
        if !config.auto_restart {
            return None;
        }

        // 3. Enforce restart loop protection
        if !self.can_restart(service_id) {
            let msg = format!(
                "Process exited unexpectedly (auto-restart suspended after {} consecutive failures)",
                self.max_consecutive_attempts
            );
            eprintln!("AutoRestart: {} for service '{}'", msg, config.name);

            let failed_info = ProcessRuntimeInfo::failed(service_id, msg)
                .with_port(config.port, None);
            emit_fn(SERVICE_RUNTIME_UPDATED_EVENT, &failed_info);
            return None;
        }

        // 4. Emit intermediate Starting transition
        let starting_info = ProcessRuntimeInfo::starting(service_id)
            .with_port(config.port, None);
        emit_fn(SERVICE_RUNTIME_UPDATED_EVENT, &starting_info);

        // 5. Record attempt and dispatch restart via existing ProcessManager path
        self.record_attempt(service_id);

        match process_manager.start_service(&config) {
            Ok(info) => {
                emit_fn(SERVICE_RUNTIME_UPDATED_EVENT, &info);
                Some(Ok(info))
            }
            Err(e) => {
                let err_msg = format!("Auto-restart failed: {}", e);
                eprintln!("AutoRestart: {} for service '{}'", err_msg, config.name);
                let failed_info = ProcessRuntimeInfo::failed(service_id, err_msg)
                    .with_port(config.port, None);
                emit_fn(SERVICE_RUNTIME_UPDATED_EVENT, &failed_info);
                Some(Err(e))
            }
        }
    }
}
