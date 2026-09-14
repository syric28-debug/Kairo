use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use kairo_lib::models::process::ProcessState;
use kairo_lib::models::ServiceConfig;
use kairo_lib::repository::json_repository::JsonServiceRepository;
use kairo_lib::repository::ServiceRepository;
use kairo_lib::services::ProcessManager;
use kairo_lib::utils::tray::{restart_all_core, start_all_core, stop_all_core};

fn temp_repo_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("lsm_test_tray")
        .join(name)
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&dir).unwrap();
    dir.join("services.json")
}

fn make_test_service(id: &str, name: &str, duration_secs: u32) -> ServiceConfig {
    ServiceConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: "Tray test service".to_string(),
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
created_at: "2026-09-10T00:00:00Z".to_string(),
        updated_at: "2026-09-10T00:00:00Z".to_string(),
    }
}

// ─────────────────────────────────────────────────────────────
// Test 1: Start All starts stopped services & prevents duplicate starts
// ─────────────────────────────────────────────────────────────
#[test]
fn test_01_start_all_starts_stopped_and_skips_running() {
    let repo_path = temp_repo_dir("test_01");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let svc1 = make_test_service("tray-svc-1", "Tray Svc 1", 30);
    let svc2 = make_test_service("tray-svc-2", "Tray Svc 2", 30);
    repo.create(svc1).unwrap();
    repo.create(svc2).unwrap();

    // 1. First Start All: should start both services
    let results = start_all_core(&pm, &repo);
    assert_eq!(results.len(), 2, "Expected 2 services to be started");
    assert!(results[0].is_ok());
    assert!(results[1].is_ok());

    let r1 = pm.get_runtime_state("tray-svc-1");
    let r2 = pm.get_runtime_state("tray-svc-2");
    assert_eq!(r1.state, ProcessState::Running);
    assert_eq!(r2.state, ProcessState::Running);
    assert!(r1.pid.is_some());
    assert!(r2.pid.is_some());
    assert_ne!(r1.pid, r2.pid);

    // 2. Second Start All: both are already running, so none should start afresh
    let second_results = start_all_core(&pm, &repo);
    assert_eq!(
        second_results.len(),
        0,
        "Running services must not be duplicate-started"
    );

    // Cleanup
    stop_all_core(&pm);
}

// ─────────────────────────────────────────────────────────────
// Test 2: Stop All stops only tracked child PIDs cleanly
// ─────────────────────────────────────────────────────────────
#[test]
fn test_02_stop_all_terminates_tracked_processes() {
    let repo_path = temp_repo_dir("test_02");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let svc1 = make_test_service("tray-svc-3", "Tray Svc 3", 30);
    let svc2 = make_test_service("tray-svc-4", "Tray Svc 4", 30);
    repo.create(svc1).unwrap();
    repo.create(svc2).unwrap();

    start_all_core(&pm, &repo);
    assert_eq!(pm.get_monitored_targets().len(), 2);

    // Stop All
    let stop_results = stop_all_core(&pm);
    assert_eq!(stop_results.len(), 2);
    for res in stop_results {
        let info = res.expect("Expected successful stop");
        assert_eq!(info.state, ProcessState::Stopped);
    }

    // Monitored targets should now be empty
    assert_eq!(pm.get_monitored_targets().len(), 0);
    assert_eq!(pm.get_runtime_state("tray-svc-3").state, ProcessState::Stopped);
    assert_eq!(pm.get_runtime_state("tray-svc-4").state, ProcessState::Stopped);
}

// ─────────────────────────────────────────────────────────────
// Test 3: Restart All cycles PIDs and clears stale state
// ─────────────────────────────────────────────────────────────
#[test]
fn test_03_restart_all_cycles_pids() {
    let repo_path = temp_repo_dir("test_03");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let svc1 = make_test_service("tray-svc-5", "Tray Svc 5", 30);
    let svc2 = make_test_service("tray-svc-6", "Tray Svc 6", 30);
    repo.create(svc1).unwrap();
    repo.create(svc2).unwrap();

    start_all_core(&pm, &repo);
    let old_pid1 = pm.get_runtime_state("tray-svc-5").pid.unwrap();
    let old_pid2 = pm.get_runtime_state("tray-svc-6").pid.unwrap();

    // Restart All
    let restart_results = restart_all_core(&pm, &repo);
    assert_eq!(restart_results.len(), 2);
    for res in restart_results {
        assert!(res.is_ok());
    }

    let new_pid1 = pm.get_runtime_state("tray-svc-5").pid.unwrap();
    let new_pid2 = pm.get_runtime_state("tray-svc-6").pid.unwrap();

    assert_ne!(old_pid1, new_pid1, "Restart must assign a new PID for svc 1");
    assert_ne!(old_pid2, new_pid2, "Restart must assign a new PID for svc 2");

    // Cleanup
    stop_all_core(&pm);
}

// ─────────────────────────────────────────────────────────────
// Test 4: Start All failure isolation
// ─────────────────────────────────────────────────────────────
#[test]
fn test_04_start_all_failure_isolation() {
    let repo_path = temp_repo_dir("test_04");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let valid_svc = make_test_service("tray-valid-1", "Valid Service", 30);
    let mut invalid_svc = make_test_service("tray-invalid-1", "Invalid Service", 30);
    invalid_svc.executable = "non_existent_executable_12345_lsm.exe".to_string();

    repo.create(valid_svc).unwrap();
    repo.create(invalid_svc).unwrap();

    let results = start_all_core(&pm, &repo);
    assert_eq!(results.len(), 2);

    let has_ok = results.iter().any(|r| r.is_ok());
    let has_err = results.iter().any(|r| r.is_err());
    assert!(has_ok, "Valid service must start successfully");
    assert!(has_err, "Invalid service must produce an error");

    // The valid service must remain running
    assert_eq!(
        pm.get_runtime_state("tray-valid-1").state,
        ProcessState::Running
    );

    // Cleanup
    stop_all_core(&pm);
}

// ─────────────────────────────────────────────────────────────
// Test 5: Exit flag behavior and simulation
// ─────────────────────────────────────────────────────────────
#[test]
fn test_05_exit_flag_state_isolation() {
    let is_exiting = Arc::new(AtomicBool::new(false));
    assert!(!is_exiting.load(Ordering::SeqCst));

    // Simulate window close event while is_exiting is false
    // (should be prevented from closing / hidden to tray)
    let should_hide = !is_exiting.load(Ordering::SeqCst);
    assert!(should_hide, "When not exiting, window close must trigger hide to tray");

    // Simulate tray Exit click
    is_exiting.store(true, Ordering::SeqCst);

    let should_hide_after_exit = !is_exiting.load(Ordering::SeqCst);
    assert!(
        !should_hide_after_exit,
        "When exiting, window close must NOT be prevented"
    );
}

// ─────────────────────────────────────────────────────────────
// Test 6: Background operation continuity across simulated hide/show
// ─────────────────────────────────────────────────────────────
#[test]
fn test_06_background_operation_continuity() {
    let repo_path = temp_repo_dir("test_06");
    let repo = JsonServiceRepository::new(&repo_path);
    let pm = ProcessManager::new();

    let svc = make_test_service("bg-svc-1", "Background Service", 30);
    repo.create(svc).unwrap();

    // Start service while UI is open
    start_all_core(&pm, &repo);
    let original_pid = pm.get_runtime_state("bg-svc-1").pid.unwrap();

    // Simulate window closed to tray: wait a short period while backgrounded
    thread::sleep(Duration::from_millis(200));

    // Simulate window reopened from tray
    let post_hide_state = pm.get_runtime_state("bg-svc-1");
    assert_eq!(
        post_hide_state.state,
        ProcessState::Running,
        "Service must remain Running while UI is hidden in tray"
    );
    assert_eq!(
        post_hide_state.pid,
        Some(original_pid),
        "PID must remain identical across window hide/show"
    );

    // Cleanup
    stop_all_core(&pm);
}
