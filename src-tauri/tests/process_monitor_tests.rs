use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

use kairo_lib::models::process::ProcessState;
use kairo_lib::models::ServiceConfig;
use kairo_lib::services::ProcessManager;

fn harmless_ping_config(id: &str, name: &str, duration_secs: u32) -> ServiceConfig {
    ServiceConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: "Process monitor test process".to_string(),
        executable: "ping".to_string(),
        arguments: vec![
            "127.0.0.1".to_string(),
            "-n".to_string(),
            duration_secs.to_string(),
        ],
        working_directory: "C:\\".to_string(),
        environment_variables: HashMap::new(),
        port: None,
        auto_start: false,
        auto_restart: false,
        health_check: None,

        api_base_path: None,

        direct_url_path: None,

        health_check_path: None,
                    kind: None,
created_at: "2026-09-09T00:00:00Z".to_string(),
        updated_at: "2026-09-09T00:00:00Z".to_string(),
    }
}

// 1. Process monitor starts correctly
#[test]
fn test_01_monitor_starts_correctly() {
    let pm = ProcessManager::new();
    let updates = pm.reconcile_unexpected_exits();
    assert!(updates.is_empty(), "Empty process manager should have no exited processes");
}

// 2. Running process remains Running
#[test]
fn test_02_running_process_remains_running() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-test-2", "Running Monitor Test", 25);

    let runtime = pm.start_service(&config).unwrap();
    assert_eq!(runtime.state, ProcessState::Running);

    // Monitoring reconciliation check
    let exits = pm.reconcile_unexpected_exits();
    assert!(exits.is_empty(), "Active process must not be flagged as exited");

    // State query must still show Running
    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Running);
    assert_eq!(state.pid, runtime.pid);

    let _ = pm.stop_service(&config.id);
}

// 3. Natural process exit is detected as Failed (even if exit code 0)
// Per Phase 4 rules: any exit without an intentional Stop = Failed
#[test]
fn test_03_natural_exit_becomes_failed() {
    let pm = ProcessManager::new();
    // 1 ping exits after ~1 second
    let config = harmless_ping_config("mon-test-3", "Short Ping Test", 1);

    let runtime = pm.start_service(&config).unwrap();
    assert_eq!(runtime.state, ProcessState::Running);

    // Wait for the process to complete naturally
    thread::sleep(Duration::from_millis(1500));

    // Monitor reconciles the exit — must be Failed, not Stopped
    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1, "Exited process must be detected by monitor");
    assert_eq!(exits[0].service_id, config.id);
    assert_eq!(exits[0].state, ProcessState::Failed, "Natural exit without intentional Stop must be Failed");

    // Subsequent query shows process is no longer tracked
    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Stopped, "After cleanup, no tracked process returns Stopped");
}

// 4. Unexpected process exit becomes Failed
#[test]
fn test_04_unexpected_exit_becomes_failed() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-test-4", "Crash Test", 30);

    let runtime = pm.start_service(&config).unwrap();
    let pid = runtime.pid.unwrap();

    // Kill externally to simulate a sudden crash/external kill
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output();

    thread::sleep(Duration::from_millis(200));

    // Watchdog reconciliation
    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1, "Killed process must be detected as an unexpected exit");
    assert_eq!(exits[0].service_id, config.id);
    assert_eq!(exits[0].state, ProcessState::Failed);
    assert!(exits[0].error_message.is_some());

    let err_msg = exits[0].error_message.as_ref().unwrap();
    assert!(
        err_msg.contains("unexpectedly") || err_msg.contains("code"),
        "Error message must document unexpected termination: {}",
        err_msg
    );
}

// 5. Intentional Stop becomes Stopped, not Failed
#[test]
fn test_05_intentional_stop_becomes_stopped() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-test-5", "Intentional Stop Test", 25);

    let _ = pm.start_service(&config).unwrap();

    // User initiates intentional stop
    let stopped = pm.stop_service(&config.id).unwrap();
    assert_eq!(stopped.state, ProcessState::Stopped);

    // Watchdog reconciliation must NOT emit a Failed event for an intentional stop
    let exits = pm.reconcile_unexpected_exits();
    assert!(exits.is_empty(), "Intentional stop must NOT be flagged as unexpected crash");

    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Stopped);
}

// 6. Multiple services are monitored independently
#[test]
fn test_06_multiple_services_monitored() {
    let pm = ProcessManager::new();
    let config_a = harmless_ping_config("mon-multi-a", "Multi A", 30);
    let config_b = harmless_ping_config("mon-multi-b", "Multi B", 30);

    let ra = pm.start_service(&config_a).unwrap();
    let rb = pm.start_service(&config_b).unwrap();

    let pid_a = ra.pid.unwrap();
    let pid_b = rb.pid.unwrap();

    // Kill ONLY service A externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid_a.to_string()])
        .output();

    thread::sleep(Duration::from_millis(200));

    // Reconcile
    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1, "Only killed service should be in exited list");
    assert_eq!(exits[0].service_id, config_a.id);
    assert_eq!(exits[0].state, ProcessState::Failed);

    // Service B must STILL be running
    let state_b = pm.get_runtime_state(&config_b.id);
    assert_eq!(state_b.state, ProcessState::Running);
    assert_eq!(state_b.pid, Some(pid_b));

    // Clean up service B
    let _ = pm.stop_service(&config_b.id);
}

// 7. Restart interaction does not create duplicate monitoring
#[test]
fn test_07_restart_interaction() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-restart-7", "Restart Interaction Test", 30);

    let _ = pm.start_service(&config).unwrap();
    let restarted = pm.restart_service(&config).unwrap();

    assert_eq!(restarted.state, ProcessState::Running);

    // Immediately check reconciliation
    let exits = pm.reconcile_unexpected_exits();
    assert!(exits.is_empty(), "Restarted process should be active without spurious exit reports");

    let _ = pm.stop_service(&config.id);
}

// 8. Monitor does not create duplicate watchers
#[test]
fn test_08_no_duplicate_watchers() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-dup-8", "No Dup Watchers", 25);

    let _ = pm.start_service(&config).unwrap();

    // Calling reconciliation repeatedly does not duplicate or corrupt records
    for _ in 0..5 {
        let exits = pm.reconcile_unexpected_exits();
        assert!(exits.is_empty());
    }

    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Running);

    let _ = pm.stop_service(&config.id);
}

// 9. PID changes are handled correctly after restart
#[test]
fn test_09_pid_changes_after_restart() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-pid-change-9", "PID Change Test", 30);

    let r1 = pm.start_service(&config).unwrap();
    let pid1 = r1.pid.unwrap();

    let r2 = pm.restart_service(&config).unwrap();
    let pid2 = r2.pid.unwrap();

    assert_ne!(pid1, pid2);

    // Kill the new PID externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid2.to_string()])
        .output();

    thread::sleep(Duration::from_millis(200));

    // Monitor should pick up exit for service with new PID
    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].service_id, config.id);
    assert_eq!(exits[0].pid, Some(pid2));
    assert_eq!(exits[0].state, ProcessState::Failed);
}

// 10. Monitor shutdown is clean
#[test]
fn test_10_clean_monitor_shutdown() {
    let pm = ProcessManager::new();
    let config = harmless_ping_config("mon-shutdown-10", "Shutdown Test", 25);

    let _ = pm.start_service(&config).unwrap();

    // Stop all processes cleanly
    pm.stop_all();

    let remaining = pm.list_runtime_states();
    assert!(remaining.is_empty(), "All processes must be stopped and cleared");

    let exits = pm.reconcile_unexpected_exits();
    assert!(exits.is_empty(), "No pending exits after stop_all");
}
