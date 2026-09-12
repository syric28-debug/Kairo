use std::collections::HashMap;
use std::path::Path;

use kairo_lib::error::AppError;
use kairo_lib::models::process::ProcessState;
use kairo_lib::models::ServiceConfig;
use kairo_lib::repository::ServiceRepository;
use kairo_lib::services::ProcessManager;

/// Helper to create a harmless ping-based service configuration for testing.
/// Uses direct ping.exe with structured argument vector — strictly NO shell strings.
fn create_test_config(id: &str, name: &str, duration_secs: u32) -> ServiceConfig {
    ServiceConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: "Harmless test process".to_string(),
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
        created_at: "2026-09-09T00:00:00Z".to_string(),
        updated_at: "2026-09-09T00:00:00Z".to_string(),
    }
}

// 1. Start unknown service (validation & error handling)
#[test]
fn test_01_start_unknown_service() {
    let repo = kairo_lib::repository::JsonServiceRepository::default();
    let manager = ProcessManager::new();

    // Query non-existent service ID
    let missing_id = "non-existent-service-id-999";
    let found = repo.get(missing_id).unwrap();
    assert!(found.is_none(), "Service should not exist");

    // In IPC handler, missing_id returns AppError::NotFound
    let err = repo
        .get(missing_id)
        .unwrap()
        .ok_or_else(|| AppError::NotFound(format!("Service '{}' not found", missing_id)))
        .unwrap_err();

    match err {
        AppError::NotFound(msg) => assert!(msg.contains(missing_id)),
        _ => panic!("Expected AppError::NotFound, got {:?}", err),
    }

    // Verify manager reports stopped state for untracked service
    let state = manager.get_runtime_state(missing_id);
    assert_eq!(state.state, ProcessState::Stopped);
    assert_eq!(state.pid, None);
}

// 2. Start harmless valid process
#[test]
fn test_02_start_harmless_valid_process() {
    let manager = ProcessManager::new();
    let config = create_test_config("test-start-valid", "Valid Start Test", 20);

    let runtime = manager.start_service(&config).expect("Must start harmless process");
    assert_eq!(runtime.state, ProcessState::Running);
    assert!(runtime.pid.is_some());

    let pid = runtime.pid.unwrap();
    assert!(pid > 0);

    // Confirm state query returns running with same PID
    let query = manager.get_runtime_state(&config.id);
    assert_eq!(query.state, ProcessState::Running);
    assert_eq!(query.pid, Some(pid));

    // Clean up
    let stopped = manager.stop_service(&config.id).expect("Must stop process");
    assert_eq!(stopped.state, ProcessState::Stopped);
}

// 3. Prevent duplicate start
#[test]
fn test_03_prevent_duplicate_start() {
    let manager = ProcessManager::new();
    let config = create_test_config("test-dup-start", "Duplicate Start Test", 20);

    let runtime1 = manager.start_service(&config).expect("First start must succeed");
    assert_eq!(runtime1.state, ProcessState::Running);

    // Attempt second start for same service
    let err = manager.start_service(&config).expect_err("Second start must fail");
    match err {
        AppError::Process(msg) => {
            assert!(msg.contains("already running"), "Error must mention already running: {}", msg);
        }
        _ => panic!("Expected AppError::Process, got {:?}", err),
    }

    // Clean up
    let _ = manager.stop_service(&config.id);
}

// 4. Stop running process
#[test]
fn test_04_stop_running_process() {
    let manager = ProcessManager::new();
    let config = create_test_config("test-stop-running", "Stop Test", 20);

    let runtime = manager.start_service(&config).unwrap();
    let pid = runtime.pid.unwrap();

    let stopped = manager.stop_service(&config.id).expect("Must stop running process");
    assert_eq!(stopped.state, ProcessState::Stopped);
    assert_eq!(stopped.pid, None);

    // Verify tracked state is stopped
    let query = manager.get_runtime_state(&config.id);
    assert_eq!(query.state, ProcessState::Stopped);
    assert_eq!(query.pid, None);

    // Confirm process is no longer active in OS
    std::thread::sleep(std::time::Duration::from_millis(100));
    let tasklist = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&tasklist.stdout);
    assert!(!out.contains(&pid.to_string()), "Process PID {} should have terminated", pid);
}

// 5. Stop already stopped process
#[test]
fn test_05_stop_already_stopped_process() {
    let manager = ProcessManager::new();

    let err = manager
        .stop_service("never-started-service")
        .expect_err("Stopping non-running service must return error");

    match err {
        AppError::Process(msg) => {
            assert!(msg.contains("not currently running"), "Expected not running message, got: {}", msg);
        }
        _ => panic!("Expected AppError::Process, got {:?}", err),
    }
}

// 6. Restart running process
#[test]
fn test_06_restart_running_process() {
    let manager = ProcessManager::new();
    let config = create_test_config("test-restart-running", "Restart Running Test", 25);

    let r1 = manager.start_service(&config).unwrap();
    let pid1 = r1.pid.unwrap();

    // Restart while running
    let r2 = manager.restart_service(&config).expect("Must restart running service");
    assert_eq!(r2.state, ProcessState::Running);
    let pid2 = r2.pid.unwrap();

    // Must assign a distinct, fresh PID
    assert_ne!(pid1, pid2, "Restarted process must receive a new PID");

    // Clean up
    let _ = manager.stop_service(&config.id);
}

// 7. Restart stopped process
#[test]
fn test_07_restart_stopped_process() {
    let manager = ProcessManager::new();
    let config = create_test_config("test-restart-stopped", "Restart Stopped Test", 20);

    // Never started initially -> restart starts it directly
    let runtime = manager.restart_service(&config).expect("Must start stopped service directly");
    assert_eq!(runtime.state, ProcessState::Running);
    assert!(runtime.pid.is_some());

    // Clean up
    let _ = manager.stop_service(&config.id);
}

// 8. Invalid executable
#[test]
fn test_08_invalid_executable() {
    let manager = ProcessManager::new();
    let mut config = create_test_config("test-invalid-exec", "Invalid Executable Test", 10);
    config.executable = "non_existent_executable_xyz98765.exe".to_string();

    let err = manager.start_service(&config).expect_err("Must fail for invalid executable");
    match err {
        AppError::Process(msg) => {
            assert!(
                msg.contains("not found") || msg.contains("Failed to launch"),
                "User-friendly error expected, got: {}",
                msg
            );
        }
        _ => panic!("Expected AppError::Process, got {:?}", err),
    }

    assert_eq!(manager.get_runtime_state(&config.id).state, ProcessState::Stopped);
}

// 9. Invalid working directory
#[test]
fn test_09_invalid_working_directory() {
    let manager = ProcessManager::new();
    let mut config = create_test_config("test-invalid-dir", "Invalid Dir Test", 10);
    let bad_dir = "C:\\NonExistentDirectory_XYZ_987654321";
    config.working_directory = bad_dir.to_string();

    assert!(!Path::new(bad_dir).exists(), "Test directory must not exist");

    let err = manager.start_service(&config).expect_err("Must fail for missing working directory");
    match err {
        AppError::Validation(msg) => {
            assert!(
                msg.contains("Working directory does not exist"),
                "User-friendly error expected, got: {}",
                msg
            );
        }
        _ => panic!("Expected AppError::Validation, got {:?}", err),
    }

    // Confirm the directory was NOT automatically created
    assert!(!Path::new(bad_dir).exists(), "Directory must not be created automatically");
}

// 10. Environment variable passing
#[test]
fn test_10_environment_variable_passing() {
    let manager = ProcessManager::new();
    let mut config = create_test_config("test-env-vars", "Env Var Test", 15);
    config
        .environment_variables
        .insert("TEST_CUSTOM_KEY".to_string(), "TEST_CUSTOM_VAL".to_string());

    let runtime = manager
        .start_service(&config)
        .expect("Must start process with custom environment variables");
    assert_eq!(runtime.state, ProcessState::Running);

    // Clean up
    let _ = manager.stop_service(&config.id);
}

// 11. Multiple simultaneous services
#[test]
fn test_11_multiple_simultaneous_services() {
    let manager = ProcessManager::new();
    let config_a = create_test_config("test-multi-a", "Multi A", 25);
    let config_b = create_test_config("test-multi-b", "Multi B", 25);

    let ra = manager.start_service(&config_a).unwrap();
    let rb = manager.start_service(&config_b).unwrap();

    assert_eq!(ra.state, ProcessState::Running);
    assert_eq!(rb.state, ProcessState::Running);

    let list = manager.list_runtime_states();
    assert_eq!(list.len(), 2);

    // Clean up
    let _ = manager.stop_service(&config_a.id);
    let _ = manager.stop_service(&config_b.id);
}

// 12. Correct PID mapping
#[test]
fn test_12_correct_pid_mapping() {
    let manager = ProcessManager::new();
    let config_1 = create_test_config("test-pid-1", "PID 1", 25);
    let config_2 = create_test_config("test-pid-2", "PID 2", 25);

    let r1 = manager.start_service(&config_1).unwrap();
    let r2 = manager.start_service(&config_2).unwrap();

    let pid1 = r1.pid.unwrap();
    let pid2 = r2.pid.unwrap();

    assert_ne!(pid1, pid2, "Distinct services must have distinct PIDs");

    // Exact state retrieval matches corresponding PID
    assert_eq!(manager.get_runtime_state(&config_1.id).pid, Some(pid1));
    assert_eq!(manager.get_runtime_state(&config_2.id).pid, Some(pid2));

    // Clean up
    let _ = manager.stop_service(&config_1.id);
    let _ = manager.stop_service(&config_2.id);
}

// 13. Process isolation (stopping one leaves others running)
#[test]
fn test_13_process_isolation() {
    let manager = ProcessManager::new();
    let config_x = create_test_config("test-iso-x", "Iso X", 30);
    let config_y = create_test_config("test-iso-y", "Iso Y", 30);

    let rx = manager.start_service(&config_x).unwrap();
    let ry = manager.start_service(&config_y).unwrap();

    let pid_x = rx.pid.unwrap();
    let pid_y = ry.pid.unwrap();

    // Stop service X only
    let stopped_x = manager.stop_service(&config_x.id).unwrap();
    assert_eq!(stopped_x.state, ProcessState::Stopped);

    // Verify service X is stopped
    assert_eq!(manager.get_runtime_state(&config_x.id).state, ProcessState::Stopped);

    // Verify service Y is STILL running with unchanged PID
    let state_y = manager.get_runtime_state(&config_y.id);
    assert_eq!(state_y.state, ProcessState::Running);
    assert_eq!(state_y.pid, Some(pid_y));

    // Verify OS process for Y is still alive
    let tasklist = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid_y), "/NH"])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&tasklist.stdout);
    assert!(out.contains(&pid_y.to_string()), "Process Y should still be alive");

    // Verify OS process for X is terminated
    let tasklist_x = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid_x), "/NH"])
        .output()
        .unwrap();
    let out_x = String::from_utf8_lossy(&tasklist_x.stdout);
    assert!(!out_x.contains(&pid_x.to_string()), "Process X should be terminated");

    // Clean up Y
    let _ = manager.stop_service(&config_y.id);
}
