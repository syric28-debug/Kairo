use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use kairo_lib::models::process::{ProcessRuntimeInfo, ProcessState};
use kairo_lib::models::ServiceConfig;
use kairo_lib::repository::json_repository::JsonServiceRepository;
use kairo_lib::repository::ServiceRepository;
use kairo_lib::services::auto_restart::AutoRestartTracker;
use kairo_lib::services::{PortChecker, ProcessManager};

fn temp_repo_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("lsm_test_auto_restart")
        .join(name)
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&dir).unwrap();
    dir.join("services.json")
}

fn make_test_service(id: &str, name: &str, duration_secs: u32, auto_restart: bool) -> ServiceConfig {
    ServiceConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: "Auto-restart test process".to_string(),
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
        auto_restart,
        health_check: None,

        api_base_path: None,

        direct_url_path: None,

        health_check_path: None,
                    kind: None,
created_at: "2026-09-10T00:00:00Z".to_string(),
        updated_at: "2026-09-10T00:00:00Z".to_string(),
    }
}

// ─────────────────────────────────────────────────────────────
// Test 1: auto_restart_disabled_does_not_restart
// ─────────────────────────────────────────────────────────────
#[test]
fn test_01_auto_restart_disabled_does_not_restart() {
    let repo_path = temp_repo_path("test_01");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();
    let mut tracker = AutoRestartTracker::default();

    let config = make_test_service("ar-disabled-1", "AR Disabled", 30, false);
    repo.create(config.clone()).unwrap();

    let runtime = pm.start_service(&config).unwrap();
    let pid = runtime.pid.unwrap();

    // Terminate process externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output();
    thread::sleep(Duration::from_millis(200));

    // Monitor detects unexpected exit
    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].state, ProcessState::Failed);

    let mut emitted = Vec::<ProcessRuntimeInfo>::new();
    let restart_res = tracker.handle_unexpected_exit(
        &config.id,
        &repo,
        &pm,
        |_event, info| emitted.push(info.clone()),
    );

    // When auto_restart is false, no restart attempt is made
    assert!(restart_res.is_none());
    assert!(emitted.is_empty());

    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Stopped);
}

// ─────────────────────────────────────────────────────────────
// Test 2: auto_restart_enabled_restarts_service
// ─────────────────────────────────────────────────────────────
#[test]
fn test_02_auto_restart_enabled_restarts_service() {
    let repo_path = temp_repo_path("test_02");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();
    let mut tracker = AutoRestartTracker::default();

    let config = make_test_service("ar-enabled-2", "AR Enabled", 30, true);
    repo.create(config.clone()).unwrap();

    let runtime = pm.start_service(&config).unwrap();
    let old_pid = runtime.pid.unwrap();

    // Terminate process externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &old_pid.to_string()])
        .output();
    thread::sleep(Duration::from_millis(200));

    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1);

    let mut emitted = Vec::<ProcessRuntimeInfo>::new();
    let restart_res = tracker.handle_unexpected_exit(
        &config.id,
        &repo,
        &pm,
        |_event, info| emitted.push(info.clone()),
    );

    assert!(restart_res.is_some());
    let new_info = restart_res.unwrap().expect("Auto restart should succeed");
    assert_eq!(new_info.state, ProcessState::Running);

    let new_pid = new_info.pid.unwrap();
    assert_ne!(old_pid, new_pid, "Restarted process must receive a fresh PID");

    // Check emitted events: Starting then Running
    assert!(emitted.iter().any(|e| e.state == ProcessState::Starting));
    assert!(emitted.iter().any(|e| e.state == ProcessState::Running));

    let live_state = pm.get_runtime_state(&config.id);
    assert_eq!(live_state.state, ProcessState::Running);
    assert_eq!(live_state.pid, Some(new_pid));

    let _ = pm.stop_service(&config.id);
}

// ─────────────────────────────────────────────────────────────
// Test 3: auto_restart_does_not_trigger_on_intentional_stop
// ─────────────────────────────────────────────────────────────
#[test]
fn test_03_auto_restart_does_not_trigger_on_intentional_stop() {
    let repo_path = temp_repo_path("test_03");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let config = make_test_service("ar-stop-3", "AR Stop", 30, true);
    repo.create(config.clone()).unwrap();

    pm.start_service(&config).unwrap();

    // Intentional stop
    let stop_info = pm.stop_service(&config.id).unwrap();
    assert_eq!(stop_info.state, ProcessState::Stopped);

    // Monitoring reconciliation check: must be completely empty!
    let exits = pm.reconcile_unexpected_exits();
    assert!(
        exits.is_empty(),
        "Intentional stop must not appear in unexpected exits"
    );

    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Stopped);
}

// ─────────────────────────────────────────────────────────────
// Test 4: auto_restart_does_not_duplicate_restart
// ─────────────────────────────────────────────────────────────
#[test]
fn test_04_auto_restart_does_not_duplicate_restart() {
    let repo_path = temp_repo_path("test_04");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();
    let mut tracker = AutoRestartTracker::default();

    let config = make_test_service("ar-no-dup-4", "AR No Dup", 30, true);
    repo.create(config.clone()).unwrap();

    let runtime = pm.start_service(&config).unwrap();
    let old_pid = runtime.pid.unwrap();

    // Terminate process externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &old_pid.to_string()])
        .output();
    thread::sleep(Duration::from_millis(200));

    // Iteration 1: detect and restart
    let exits1 = pm.reconcile_unexpected_exits();
    assert_eq!(exits1.len(), 1);

    let res = tracker.handle_unexpected_exit(&config.id, &repo, &pm, |_event, _info| {});
    let restarted_pid = res.unwrap().unwrap().pid.unwrap();

    // Iteration 2 (next monitor poll): the newly spawned process is still running
    let exits2 = pm.reconcile_unexpected_exits();
    assert!(
        exits2.is_empty(),
        "Subsequent monitor iteration must not flag the replacement process as exited"
    );

    // PID must remain identical to restarted_pid
    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.pid, Some(restarted_pid));
    assert_eq!(state.state, ProcessState::Running);

    let _ = pm.stop_service(&config.id);
}

// ─────────────────────────────────────────────────────────────
// Test 5: auto_restart_failure_isolated
// ─────────────────────────────────────────────────────────────
#[test]
fn test_05_auto_restart_failure_isolated() {
    let repo_path = temp_repo_path("test_05");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();
    let mut tracker = AutoRestartTracker::default();

    let valid_cfg = make_test_service("ar-iso-valid", "Valid Service", 30, true);
    let invalid_cfg = make_test_service("ar-iso-invalid", "Invalid Service", 30, true);
    repo.create(valid_cfg.clone()).unwrap();
    repo.create(invalid_cfg.clone()).unwrap();

    let r_valid = pm.start_service(&valid_cfg).unwrap();
    let r_invalid = pm.start_service(&invalid_cfg).unwrap();

    // Now modify the invalid service config to point to an un-launchable executable
    let mut broken_cfg = invalid_cfg.clone();
    broken_cfg.executable = "non_existent_fake_binary_12345.exe".to_string();
    let broken_id = broken_cfg.id.clone();
    repo.update(&broken_id, broken_cfg).unwrap();

    // Terminate both processes externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &r_valid.pid.unwrap().to_string()])
        .output();
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &r_invalid.pid.unwrap().to_string()])
        .output();
    thread::sleep(Duration::from_millis(200));

    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 2);

    // Attempt restart on both
    let valid_res = tracker.handle_unexpected_exit(&valid_cfg.id, &repo, &pm, |_event, _info| {});
    let invalid_res =
        tracker.handle_unexpected_exit(&invalid_cfg.id, &repo, &pm, |_event, _info| {});

    assert!(valid_res.is_some());
    assert!(valid_res.unwrap().is_ok());

    assert!(invalid_res.is_some());
    assert!(invalid_res.unwrap().is_err());

    // Valid service must be Running and healthy
    assert_eq!(
        pm.get_runtime_state(&valid_cfg.id).state,
        ProcessState::Running
    );

    let _ = pm.stop_service(&valid_cfg.id);
}

// ─────────────────────────────────────────────────────────────
// Test 6: auto_restart_with_port_readiness
// ─────────────────────────────────────────────────────────────
#[test]
fn test_06_auto_restart_with_port_readiness() {
    let repo_path = temp_repo_path("test_06");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();
    let mut tracker = AutoRestartTracker::default();

    // Bind loopback TCP listener on an ephemeral port
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))
        .expect("Failed to bind ephemeral listener");
    let port = listener.local_addr().unwrap().port();

    let mut config = make_test_service("ar-port-6", "AR Port Test", 30, true);
    config.port = Some(port);
    repo.create(config.clone()).unwrap();

    let runtime = pm.start_service(&config).unwrap();
    let pid1 = runtime.pid.unwrap();

    // Check readiness initially
    let is_listening = PortChecker::check_local_port(port);
    assert!(is_listening);
    let ready_info = pm.update_port_readiness(&config.id, pid1, true).unwrap();
    assert_eq!(ready_info.state, ProcessState::Ready);

    // Kill process externally
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid1.to_string()])
        .output();
    thread::sleep(Duration::from_millis(200));

    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1);

    // Auto restart
    let restart_res = tracker.handle_unexpected_exit(&config.id, &repo, &pm, |_event, _info| {});
    let new_info = restart_res.unwrap().unwrap();
    let pid2 = new_info.pid.unwrap();
    assert_ne!(pid1, pid2);

    // Port is still listening (listener kept alive), so readiness updates to Ready for pid2
    let is_still_listening = PortChecker::check_local_port(port);
    assert!(is_still_listening);

    let re_ready_info = pm
        .update_port_readiness(&config.id, pid2, true)
        .expect("Readiness update should succeed for new PID");
    assert_eq!(re_ready_info.state, ProcessState::Ready);
    assert_eq!(re_ready_info.pid, Some(pid2));

    let _ = pm.stop_service(&config.id);
    drop(listener);
}

// ─────────────────────────────────────────────────────────────
// Test 7: intentional_restart_is_not_double_restarted
// ─────────────────────────────────────────────────────────────
#[test]
fn test_07_intentional_restart_is_not_double_restarted() {
    let repo_path = temp_repo_path("test_07");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let config = make_test_service("ar-intent-7", "AR Intentional Restart", 30, true);
    repo.create(config.clone()).unwrap();

    let r1 = pm.start_service(&config).unwrap();
    let pid1 = r1.pid.unwrap();

    // Intentional restart
    let r2 = pm.restart_service(&config).unwrap();
    let pid2 = r2.pid.unwrap();
    assert_ne!(pid1, pid2);

    // Reconcile unexpected exits immediately: must be empty
    let exits = pm.reconcile_unexpected_exits();
    assert!(
        exits.is_empty(),
        "Intentional restart must never trigger unexpected exit or auto-restart"
    );

    let state = pm.get_runtime_state(&config.id);
    assert_eq!(state.state, ProcessState::Running);
    assert_eq!(state.pid, Some(pid2));

    let _ = pm.stop_service(&config.id);
}

// ─────────────────────────────────────────────────────────────
// Test 8: restart_loop_protection_suspends_after_max_attempts
// ─────────────────────────────────────────────────────────────
#[test]
fn test_08_restart_loop_protection_suspends_after_max_attempts() {
    let repo_path = temp_repo_path("test_08");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();
    // Max 3 attempts, 30s cooldown
    let mut tracker = AutoRestartTracker::new(
        3,
        Duration::from_secs(30),
        Duration::from_secs(10),
    );

    let config = make_test_service("ar-loop-8", "AR Loop Protection", 30, true);
    repo.create(config.clone()).unwrap();

    pm.start_service(&config).unwrap();

    // Simulate 3 rapid crashes
    for attempt in 1..=3 {
        let current_pid = pm.get_runtime_state(&config.id).pid.unwrap();
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &current_pid.to_string()])
            .output();
        thread::sleep(Duration::from_millis(150));

        let exits = pm.reconcile_unexpected_exits();
        assert_eq!(exits.len(), 1);

        let res = tracker.handle_unexpected_exit(&config.id, &repo, &pm, |_event, _info| {});
        assert!(res.is_some(), "Attempt {} should be allowed", attempt);
        assert!(res.unwrap().is_ok());
        assert_eq!(tracker.get_consecutive_attempts(&config.id), attempt);
    }

    // 4th crash: should be BLOCKED by loop protection
    let current_pid = pm.get_runtime_state(&config.id).pid.unwrap();
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &current_pid.to_string()])
        .output();
    thread::sleep(Duration::from_millis(150));

    let exits = pm.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1);

    let mut emitted = Vec::<ProcessRuntimeInfo>::new();
    let res = tracker.handle_unexpected_exit(&config.id, &repo, &pm, |_event, info| {
        emitted.push(info.clone());
    });

    // Blocked: returns None
    assert!(res.is_none(), "4th rapid crash must be blocked by loop protection");

    // Verify explanatory error message was emitted
    let has_suspension_msg = emitted.iter().any(|e| {
        e.state == ProcessState::Failed
            && e.error_message
                .as_ref()
                .map(|m| m.contains("auto-restart suspended"))
                .unwrap_or(false)
    });
    assert!(
        has_suspension_msg,
        "Expected suspension failure message on loop limit"
    );
}
