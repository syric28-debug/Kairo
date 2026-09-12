use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::error::AppError;
use crate::models::log::{LogEntry, LogStream};
use crate::models::process::{ProcessRuntimeInfo, ProcessState};
use crate::models::ServiceConfig;
use crate::repository::json_repository::JsonServiceRepository;
use crate::services::LogManager;

/// Internal representation of an actively tracked child process.
pub struct ManagedProcess {
    pub service_id: String,
    pub pid: u32,
    pub child: Child,
    pub state: ProcessState,
    pub started_at: String,
    pub port: Option<u16>,
    pub reader_threads: Vec<JoinHandle<()>>,
}

/// Lightweight snapshot of an active process target for external observation.
#[derive(Debug, Clone)]
pub struct MonitoredTarget {
    pub service_id: String,
    pub pid: u32,
    pub port: Option<u16>,
    pub state: ProcessState,
    pub started_at: String,
}

/// Internal state managed behind a single synchronization boundary.
struct ProcessManagerState {
    processes: HashMap<String, ManagedProcess>,
    in_flight: HashSet<String>,
}

/// Core process management service for KAIRO.
///
/// Responsibilities:
/// - Direct, non-shell execution using `std::process::Command`
/// - Exact PID tracking and isolated process termination
/// - In-memory runtime state tracking independent of persistence
/// - Single synchronization boundary preventing overlapping operations
/// - Strict Phase 3 boundary: no continuous background watchers or auto-restart
#[derive(Clone)]
pub struct ProcessManager {
    state: Arc<Mutex<ProcessManagerState>>,
    log_manager: Arc<LogManager>,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self::with_log_manager(Arc::new(LogManager::default()))
    }

    pub fn with_log_manager(log_manager: Arc<LogManager>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ProcessManagerState {
                processes: HashMap::new(),
                in_flight: HashSet::new(),
            })),
            log_manager,
        }
    }

    pub fn log_manager(&self) -> Arc<LogManager> {
        Arc::clone(&self.log_manager)
    }

    /// Helper to get current ISO 8601 timestamp.
    fn current_timestamp() -> String {
        JsonServiceRepository::current_timestamp()
    }

    /// Launches a service process using its exact configuration.
    ///
    /// Security & Execution Rules:
    /// - Direct execution via `std::process::Command` (no shell, no cmd.exe, no powershell)
    /// - Executable, arguments, working directory, and environment variables remain separated
    /// - Configured environment variables extend the environment and override matching keys
    /// - Standard streams are bound to `Stdio::null()` to prevent blocking in Phase 3
    pub fn start_service(&self, config: &ServiceConfig) -> Result<ProcessRuntimeInfo, AppError> {
        // 1. Validate configuration
        config.validate()?;

        // 2. Check working directory if specified
        let working_dir_str = config.working_directory.trim();
        if !working_dir_str.is_empty() {
            let path = Path::new(working_dir_str);
            if !path.exists() {
                return Err(AppError::Validation(format!(
                    "Working directory does not exist: '{}'",
                    working_dir_str
                )));
            }
            if !path.is_dir() {
                return Err(AppError::Validation(format!(
                    "Configured working directory is not a directory: '{}'",
                    working_dir_str
                )));
            }
        }

        // 3. Single synchronization boundary check & reserve in-flight status
        {
            let mut state = self.state.lock().map_err(|_| {
                AppError::Process("Failed to acquire process manager lock".into())
            })?;

            if state.in_flight.contains(&config.id) {
                return Err(AppError::Process(format!(
                    "An operation is already in progress for service '{}'",
                    config.name
                )));
            }

            if let Some(managed) = state.processes.get_mut(&config.id) {
                match managed.child.try_wait() {
                    Ok(None) => {
                        return Err(AppError::Process(format!(
                            "Service '{}' is already running (PID {})",
                            config.name, managed.pid
                        )));
                    }
                    Ok(Some(_)) | Err(_) => {
                        // Process had exited previously; clean up stale handle
                        state.processes.remove(&config.id);
                    }
                }
            }

            state.in_flight.insert(config.id.clone());
        }

        // RAII guard to guarantee in_flight cleanup even on failure
        let guard = FlightGuard {
            service_id: config.id.clone(),
            state: Arc::clone(&self.state),
        };

        // 4. Build secure Command without shell wrapper
        let mut cmd = Command::new(&config.executable);
        cmd.args(&config.arguments);

        if !working_dir_str.is_empty() {
            cmd.current_dir(working_dir_str);
        }

        // Environment variables extend and override inherited environment
        for (key, val) in &config.environment_variables {
            cmd.env(key, val);
        }

        // Captured asynchronous streams for Phase 9 logging
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Apply platform-specific flags (isolated to platform helper)
        platform::apply_child_process_flags(&mut cmd);

        // 5. Spawn child process
        let mut child = cmd.spawn().map_err(|e| {
            let msg = match e.kind() {
                std::io::ErrorKind::NotFound => format!(
                    "Executable '{}' not found. Please verify the file path or command name.",
                    config.executable
                ),
                std::io::ErrorKind::PermissionDenied => format!(
                    "Permission denied when attempting to execute '{}'.",
                    config.executable
                ),
                _ => format!(
                    "Failed to launch service executable '{}': {}",
                    config.executable, e
                ),
            };
            AppError::Process(msg)
        })?;

        // 6. Capture exact PID and store tracking handle
        let pid = child.id();
        let started_at = Self::current_timestamp();

        let mut reader_threads = Vec::new();
        if let Some(stdout) = child.stdout.take() {
            let handle = spawn_stream_reader(
                stdout,
                config.id.clone(),
                LogStream::Stdout,
                Arc::clone(&self.log_manager),
            );
            reader_threads.push(handle);
        }

        if let Some(stderr) = child.stderr.take() {
            let handle = spawn_stream_reader(
                stderr,
                config.id.clone(),
                LogStream::Stderr,
                Arc::clone(&self.log_manager),
            );
            reader_threads.push(handle);
        }

        let managed = ManagedProcess {
            service_id: config.id.clone(),
            pid,
            child,
            state: ProcessState::Running,
            started_at: started_at.clone(),
            port: config.port,
            reader_threads,
        };

        {
            let mut state = self.state.lock().map_err(|_| {
                AppError::Process("Failed to acquire process manager lock".into())
            })?;
            state.processes.insert(config.id.clone(), managed);
        }

        // Dismiss the in-flight guard
        drop(guard);

        Ok(ProcessRuntimeInfo::running(config.id.clone(), pid, started_at)
            .with_port(config.port, config.port.map(|_| false)))
    }

    /// Stops an actively tracked service process.
    ///
    /// Termination Rules:
    /// - Targets strictly the exact child process handle tracked in memory
    /// - Never searches by executable name
    /// - Waits for the process to terminate before completing
    pub fn stop_service(&self, service_id: &str) -> Result<ProcessRuntimeInfo, AppError> {
        // Reserve in-flight lock and extract child handle
        let mut child_to_stop = {
            let mut state = self.state.lock().map_err(|_| {
                AppError::Process("Failed to acquire process manager lock".into())
            })?;

            if state.in_flight.contains(service_id) {
                return Err(AppError::Process(format!(
                    "An operation is already in progress for service '{}'",
                    service_id
                )));
            }

            let managed = match state.processes.get_mut(service_id) {
                Some(m) => m,
                None => {
                    return Err(AppError::Process(format!(
                        "Service '{}' is not currently running",
                        service_id
                    )));
                }
            };

            // Check if process has already terminated on its own
            match managed.child.try_wait() {
                Ok(Some(_)) => {
                    state.processes.remove(service_id);
                    return Ok(ProcessRuntimeInfo::stopped(service_id));
                }
                Ok(None) => {}
                Err(e) => {
                    return Err(AppError::Process(format!(
                        "Error checking process state before stopping: {}",
                        e
                    )));
                }
            }

            state.in_flight.insert(service_id.to_string());
            // Remove from active map to terminate outside lock
            state.processes.remove(service_id).unwrap()
        };

        let guard = FlightGuard {
            service_id: service_id.to_string(),
            state: Arc::clone(&self.state),
        };

        // Terminate exact process handle
        if let Err(e) = child_to_stop.child.kill() {
            if e.kind() != std::io::ErrorKind::NotFound && e.kind() != std::io::ErrorKind::InvalidInput {
                return Err(AppError::Process(format!(
                    "Failed to terminate process (PID {}): {}",
                    child_to_stop.pid, e
                )));
            }
        }

        // Wait for child exit
        let _ = child_to_stop.child.wait();

        // Drain / wait for log reader threads to terminate
        for handle in child_to_stop.reader_threads {
            let _ = handle.join();
        }

        drop(guard);

        Ok(ProcessRuntimeInfo::stopped(service_id))
    }

    /// Restarts a service:
    /// - If running: stops exact process, waits for exit, then starts afresh
    /// - If stopped: starts directly
    pub fn restart_service(&self, config: &ServiceConfig) -> Result<ProcessRuntimeInfo, AppError> {
        // Check if currently running and stop if so
        let is_running = {
            let mut state = self.state.lock().map_err(|_| {
                AppError::Process("Failed to acquire process manager lock".into())
            })?;

            if state.in_flight.contains(&config.id) {
                return Err(AppError::Process(format!(
                    "An operation is already in progress for service '{}'",
                    config.name
                )));
            }

            if let Some(managed) = state.processes.get_mut(&config.id) {
                match managed.child.try_wait() {
                    Ok(None) => true,
                    _ => {
                        state.processes.remove(&config.id);
                        false
                    }
                }
            } else {
                false
            }
        };

        if is_running {
            self.stop_service(&config.id)?;
        }

        self.start_service(config)
    }

    /// Passive query for the current runtime state of a specific service.
    /// Does NOT run any background watcher or loop.
    pub fn get_runtime_state(&self, service_id: &str) -> ProcessRuntimeInfo {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return ProcessRuntimeInfo::stopped(service_id),
        };

        if let Some(managed) = state.processes.get_mut(service_id) {
            match managed.child.try_wait() {
                Ok(None) => {
                    let port = managed.port;
                    let port_listening = if managed.state == ProcessState::Ready {
                        Some(true)
                    } else if port.is_some() {
                        Some(false)
                    } else {
                        None
                    };
                    ProcessRuntimeInfo {
                        service_id: service_id.to_string(),
                        pid: Some(managed.pid),
                        state: managed.state,
                        started_at: Some(managed.started_at.clone()),
                        error_message: None,
                        port,
                        port_listening,
                    }
                }
                Ok(Some(status)) => {
                    let exit_code = status.code();
                    let pid = managed.pid;
                    let port = managed.port;
                    state.processes.remove(service_id);

                    // Any exit detected passively (not via intentional Stop) is Failed,
                    // even if exit code is 0 — the user did not stop the service.
                    let error_message = match exit_code {
                        Some(code) => format!("Process exited unexpectedly with code {}", code),
                        None => "Process terminated unexpectedly (external termination)".to_string(),
                    };
                    ProcessRuntimeInfo {
                        service_id: service_id.to_string(),
                        pid: Some(pid),
                        state: ProcessState::Failed,
                        started_at: None,
                        error_message: Some(error_message),
                        port,
                        port_listening: None,
                    }
                }
                Err(e) => {
                    let port = managed.port;
                    state.processes.remove(service_id);
                    ProcessRuntimeInfo::failed(service_id, format!("Process query error: {}", e))
                        .with_port(port, None)
                }
            }
        } else {
            ProcessRuntimeInfo::stopped(service_id)
        }
    }

    /// Passive query for all currently tracked process runtime states.
    pub fn list_runtime_states(&self) -> Vec<ProcessRuntimeInfo> {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        let mut exited = Vec::new();

        for (id, managed) in state.processes.iter_mut() {
            match managed.child.try_wait() {
                Ok(None) => {
                    let port = managed.port;
                    let port_listening = if managed.state == ProcessState::Ready {
                        Some(true)
                    } else if port.is_some() {
                        Some(false)
                    } else {
                        None
                    };
                    results.push(ProcessRuntimeInfo {
                        service_id: id.clone(),
                        pid: Some(managed.pid),
                        state: managed.state,
                        started_at: Some(managed.started_at.clone()),
                        error_message: None,
                        port,
                        port_listening,
                    });
                }
                Ok(Some(status)) => {
                    exited.push(id.clone());
                    // Any exit detected passively is classified as Failed,
                    // even if exit code is 0 — the user did not stop the service.
                    let exit_code = status.code();
                    let error_message = match exit_code {
                        Some(code) => format!("Process exited unexpectedly with code {}", code),
                        None => "Process terminated unexpectedly (external termination)".to_string(),
                    };
                    results.push(ProcessRuntimeInfo {
                        service_id: id.clone(),
                        pid: Some(managed.pid),
                        state: ProcessState::Failed,
                        started_at: None,
                        error_message: Some(error_message),
                        port: managed.port,
                        port_listening: None,
                    });
                }
                Err(e) => {
                    exited.push(id.clone());
                    results.push(
                        ProcessRuntimeInfo::failed(id, format!("Process query error: {}", e))
                            .with_port(managed.port, None),
                    );
                }
            }
        }

        for id in exited {
            state.processes.remove(&id);
        }

        results
    }

    /// Reconciles actively tracked processes and detects unexpected exits / crashes.
    ///
    /// Rules:
    /// - Ignores services currently in `in_flight` (actively being stopped/restarted by user)
    /// - Checks active child handles via non-blocking `try_wait()`
    /// - If process exited while tracked as Running and not in intentional stop, marks as Failed
    /// - Captures exit code / termination status
    /// - Returns a list of state updates so the monitor can notify the frontend
    pub fn reconcile_unexpected_exits(&self) -> Vec<ProcessRuntimeInfo> {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut exited_ids = Vec::new();
        let mut updates = Vec::new();

        // Snapshot in-flight IDs to avoid simultaneous borrow conflict
        let in_flight_snapshot: HashSet<String> = state.in_flight.clone();

        for (id, managed) in state.processes.iter_mut() {
            // Skip any service that is actively in an intentional start/stop/restart transition
            if in_flight_snapshot.contains(id) {
                continue;
            }

            match managed.child.try_wait() {
                Ok(Some(status)) => {
                    exited_ids.push(id.clone());
                    let exit_code = status.code();
                    let pid = managed.pid;
                    let error_message = match exit_code {
                        Some(code) => format!("Process exited unexpectedly with code {}", code),
                        None => "Process terminated unexpectedly (external termination)".to_string(),
                    };

                    updates.push(ProcessRuntimeInfo {
                        service_id: id.clone(),
                        pid: Some(pid),
                        state: ProcessState::Failed,
                        started_at: Some(managed.started_at.clone()),
                        error_message: Some(error_message),
                        port: managed.port,
                        port_listening: None,
                    });
                }
                Ok(None) => {
                    // Still running normally
                }
                Err(e) => {
                    exited_ids.push(id.clone());
                    updates.push(
                        ProcessRuntimeInfo::failed(
                            id.clone(),
                            format!("Process query error: {}", e),
                        )
                        .with_port(managed.port, None),
                    );
                }
            }
        }

        let mut exited_processes = Vec::new();
        for id in exited_ids {
            if let Some(p) = state.processes.remove(&id) {
                exited_processes.push(p);
            }
        }

        // Drain any lingering reader threads outside lock
        for p in exited_processes {
            for handle in p.reader_threads {
                let _ = handle.join();
            }
        }

        updates
    }

    /// Returns a lightweight snapshot of all currently managed active processes.
    /// Used by ProcessMonitor to probe port readiness outside of locks.
    pub fn get_monitored_targets(&self) -> Vec<MonitoredTarget> {
        let state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        state
            .processes
            .values()
            .map(|m| MonitoredTarget {
                service_id: m.service_id.clone(),
                pid: m.pid,
                port: m.port,
                state: m.state,
                started_at: m.started_at.clone(),
            })
            .collect()
    }

    /// Updates the port readiness state of a tracked process.
    ///
    /// Concurrency & Stale Result Guards:
    /// - Checks that service is not actively in `in_flight` (stop/restart in progress)
    /// - Matches `expected_pid` to ensure a stale probe result from an old process instance
    ///   is never applied to a restarted process instance
    /// - Verifies that the child process is still alive
    /// - Transitions `Running` -> `Ready` (when listening) or `Ready` -> `Running` (when not listening)
    /// - Emits update ONLY when an actual state transition occurs
    pub fn update_port_readiness(
        &self,
        service_id: &str,
        expected_pid: u32,
        is_listening: bool,
    ) -> Option<ProcessRuntimeInfo> {
        let mut state = self.state.lock().ok()?;

        // Guard 1: In-flight operations
        if state.in_flight.contains(service_id) {
            return None;
        }

        // Guard 2: Process must still exist
        let managed = state.processes.get_mut(service_id)?;

        // Guard 3: Strict PID match prevents stale results from old process instance
        if managed.pid != expected_pid {
            return None;
        }

        // Guard 4: Process must still be alive
        match managed.child.try_wait() {
            Ok(None) => {}
            _ => return None,
        }

        let new_state = if is_listening {
            ProcessState::Ready
        } else {
            ProcessState::Running
        };

        if managed.state != new_state {
            managed.state = new_state;

            let info = match new_state {
                ProcessState::Ready => ProcessRuntimeInfo::ready(
                    service_id,
                    managed.pid,
                    &managed.started_at,
                    managed.port.unwrap_or_default(),
                ),
                _ => ProcessRuntimeInfo::running(
                    service_id,
                    managed.pid,
                    &managed.started_at,
                )
                .with_port(managed.port, Some(false)),
            };

            Some(info)
        } else {
            None
        }
    }

    /// Stops all currently managed child processes.
    pub fn stop_all(&self) {
        if let Ok(mut state) = self.state.lock() {
            for (_, managed) in state.processes.iter_mut() {
                let _ = managed.child.kill();
                let _ = managed.child.wait();
            }
            state.processes.clear();
            state.in_flight.clear();
        }
    }
}

/// RAII guard ensuring in_flight tracking is always cleared upon exit or unwind.
struct FlightGuard {
    service_id: String,
    state: Arc<Mutex<ProcessManagerState>>,
}

impl Drop for FlightGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.in_flight.remove(&self.service_id);
        }
    }
}

/// Platform-specific process configuration module.
/// Isolates Windows-specific process creation flags from the core manager.
mod platform {
    use std::process::Command;

    #[cfg(windows)]
    pub fn apply_child_process_flags(cmd: &mut Command) {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(windows))]
    pub fn apply_child_process_flags(_cmd: &mut Command) {
        // No-op on non-Windows platforms
    }
}

/// Spawns a dedicated asynchronous reader thread consuming process stdout or stderr.
/// Drains line-by-line into the service's bounded in-memory log buffer.
fn spawn_stream_reader<R: Read + Send + 'static>(
    reader: R,
    service_id: String,
    stream: LogStream,
    log_manager: Arc<LogManager>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("lsm-log-{:?}-{}", stream, service_id))
        .spawn(move || {
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                line.clear();
                match buf_reader.read_line(&mut line) {
                    Ok(0) => break, // EOF reached on process termination
                    Ok(_) => {
                        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
                        let msg = if trimmed.len() > 16384 {
                            format!("{}... [truncated]", &trimmed[..16384])
                        } else {
                            trimmed.to_string()
                        };
                        let entry = LogEntry::new(service_id.clone(), stream, msg);
                        log_manager.append(entry);
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::BrokenPipe {
                            eprintln!(
                                "ProcessManager: Error reading log stream {:?} for service '{}': {}",
                                stream, service_id, e
                            );
                        }
                        break;
                    }
                }
            }
        })
        .expect("Failed to spawn process log reader thread")
}
