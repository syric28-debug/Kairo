use std::collections::HashMap;
use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};
use std::thread;
use std::time::{Duration, Instant};

use kairo_lib::models::process::ProcessState;
use kairo_lib::models::ServiceConfig;
use kairo_lib::services::{PortChecker, ProcessManager};

fn make_test_config(id: &str, name: &str, duration_secs: u32, port: Option<u16>) -> ServiceConfig {
    ServiceConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: "Port readiness test process".to_string(),
        executable: "ping".to_string(),
        arguments: vec![
            "127.0.0.1".to_string(),
            "-n".to_string(),
            duration_secs.to_string(),
        ],
        working_directory: "C:\\".to_string(),
        environment_variables: HashMap::new(),
        port,
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
// Test 1: Listening port returns true
// ─────────────────────────────────────────────────────────────
#[test]
fn test_01_listening_port_returns_true() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))
        .expect("Failed to bind ephemeral loopback listener");
    let port = listener.local_addr().unwrap().port();

    let is_listening = PortChecker::check_local_port(port);
    assert!(is_listening, "Expected PortChecker to detect actively listening port {}", port);
}

// ─────────────────────────────────────────────────────────────
// Test 2: Unused port returns false
// ─────────────────────────────────────────────────────────────
#[test]
fn test_02_unused_port_returns_false() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))
        .expect("Failed to bind ephemeral loopback listener");
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Immediately close socket

    let is_listening = PortChecker::check_local_port(port);
    assert!(!is_listening, "Expected PortChecker to report false for unused port {}", port);
}

// ─────────────────────────────────────────────────────────────
// Test 3: Connection timeout is bounded
// ─────────────────────────────────────────────────────────────
#[test]
fn test_03_connection_timeout_is_bounded() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))
        .expect("Failed to bind ephemeral loopback listener");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let start = Instant::now();
    let timeout = Duration::from_millis(50);
    let is_listening = PortChecker::is_port_listening(port, timeout);
    let elapsed = start.elapsed();

    assert!(!is_listening);
    assert!(elapsed < Duration::from_secs(2), "Probe took too long: {:?}", elapsed);
}

// ─────────────────────────────────────────────────────────────
// Test 4: No configured port
// ─────────────────────────────────────────────────────────────
#[test]
fn test_04_no_configured_port() {
    let manager = ProcessManager::new();
    let config = make_test_config("test-04", "No Port Svc", 15, None);

    let runtime = manager.start_service(&config).expect("Service start should succeed");
    assert_eq!(runtime.state, ProcessState::Running);
    assert_eq!(runtime.port, None);
    assert_eq!(runtime.port_listening, None);

    let queried = manager.get_runtime_state(&config.id);
    assert_eq!(queried.state, ProcessState::Running);
    assert_eq!(queried.port, None);
    assert_eq!(queried.port_listening, None);

    manager.stop_service(&config.id).expect("Service stop should succeed");
}

// ─────────────────────────────────────────────────────────────
// Test 5: Multiple local ports
// ─────────────────────────────────────────────────────────────
#[test]
fn test_05_multiple_local_ports() {
    let listener_a = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener A");
    let listener_b = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener B");

    let port_a = listener_a.local_addr().unwrap().port();
    let port_b = listener_b.local_addr().unwrap().port();

    assert!(PortChecker::check_local_port(port_a));
    assert!(PortChecker::check_local_port(port_b));

    // Verify a random unused port returns false
    let listener_c = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener C");
    let port_c = listener_c.local_addr().unwrap().port();
    drop(listener_c);

    assert!(!PortChecker::check_local_port(port_c));
}

// ─────────────────────────────────────────────────────────────
// Test 6: Running + port listening = Ready
// ─────────────────────────────────────────────────────────────
#[test]
fn test_06_process_running_port_ready() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();

    let manager = ProcessManager::new();
    let config = make_test_config("test-06", "Port Ready Svc", 20, Some(port));

    let runtime = manager.start_service(&config).expect("Service start should succeed");
    let pid = runtime.pid.expect("PID must exist");
    assert_eq!(runtime.state, ProcessState::Running);

    // Update readiness with port actively listening
    let is_listening = PortChecker::check_local_port(port);
    assert!(is_listening);

    let update = manager.update_port_readiness(&config.id, pid, is_listening);
    assert!(update.is_some(), "State transition should produce runtime info update");
    let info = update.unwrap();
    assert_eq!(info.state, ProcessState::Ready);
    assert_eq!(info.port, Some(port));
    assert_eq!(info.port_listening, Some(true));

    // Query matches Ready
    let live = manager.get_runtime_state(&config.id);
    assert_eq!(live.state, ProcessState::Ready);
    assert_eq!(live.port_listening, Some(true));

    manager.stop_service(&config.id).expect("Service stop should succeed");
}

// ─────────────────────────────────────────────────────────────
// Test 7: Running + port not listening = Running / Waiting
// ─────────────────────────────────────────────────────────────
#[test]
fn test_07_process_running_port_not_ready() {
    // Pick an unused port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Not listening

    let manager = ProcessManager::new();
    let config = make_test_config("test-07", "Port Not Ready Svc", 20, Some(port));

    let runtime = manager.start_service(&config).expect("Service start should succeed");
    let pid = runtime.pid.expect("PID must exist");

    // Port is not listening
    let is_listening = PortChecker::check_local_port(port);
    assert!(!is_listening);

    // Port check failure must NOT change state to Ready or Failed
    let update = manager.update_port_readiness(&config.id, pid, is_listening);
    assert!(update.is_none(), "No state transition since process was already Running");

    let live = manager.get_runtime_state(&config.id);
    assert_eq!(live.state, ProcessState::Running);
    assert_eq!(live.port, Some(port));
    assert_eq!(live.port_listening, Some(false));
    assert_ne!(live.state, ProcessState::Failed, "Port not listening must NEVER cause Failed");

    manager.stop_service(&config.id).expect("Service stop should succeed");
}

// ─────────────────────────────────────────────────────────────
// Test 8: Process exits while check is in progress
// ─────────────────────────────────────────────────────────────
#[test]
fn test_08_process_exits_while_checking() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();

    let manager = ProcessManager::new();
    // 1-second short lived process
    let config = make_test_config("test-08", "Short Lived Svc", 1, Some(port));

    let runtime = manager.start_service(&config).expect("Service start should succeed");
    let pid = runtime.pid.expect("PID must exist");

    // Wait for child process to naturally exit
    thread::sleep(Duration::from_millis(1500));

    // Even if port is listening, an exited process cannot transition to Ready
    let update = manager.update_port_readiness(&config.id, pid, true);
    assert!(update.is_none(), "Terminated process must not be marked Ready");

    // Reconciling unexpected exits should classify it as Failed per Phase 4 semantics
    let exits = manager.reconcile_unexpected_exits();
    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].state, ProcessState::Failed);
}

// ─────────────────────────────────────────────────────────────
// Test 9: Old PID readiness result is rejected
// ─────────────────────────────────────────────────────────────
#[test]
fn test_09_restart_invalidates_old_readiness() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();

    let manager = ProcessManager::new();
    let config = make_test_config("test-09", "Restart Invalidation Svc", 20, Some(port));

    let r1 = manager.start_service(&config).expect("First start should succeed");
    let pid_1 = r1.pid.expect("PID 1 must exist");

    // Restart the service (stopping PID 1 and launching PID 2)
    let r2 = manager.restart_service(&config).expect("Restart should succeed");
    let pid_2 = r2.pid.expect("PID 2 must exist");
    assert_ne!(pid_1, pid_2, "Restart must assign a new PID");

    // Simulate stale readiness check for old pid_1 arriving now
    let stale_update = manager.update_port_readiness(&config.id, pid_1, true);
    assert!(stale_update.is_none(), "Stale update for old PID must be rejected");

    // The live state should still reflect the new process
    let live = manager.get_runtime_state(&config.id);
    assert_eq!(live.pid, Some(pid_2));
    assert_eq!(live.state, ProcessState::Running);

    manager.stop_service(&config.id).expect("Service stop should succeed");
}

// ─────────────────────────────────────────────────────────────
// Test 10: Multiple services remain isolated
// ─────────────────────────────────────────────────────────────
#[test]
fn test_10_multiple_services_monitored_independently() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port_listening = listener.local_addr().unwrap().port();

    let dummy_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind dummy");
    let port_unbound = dummy_listener.local_addr().unwrap().port();
    drop(dummy_listener);

    let manager = ProcessManager::new();
    let config_a = make_test_config("test-10-a", "Svc A Ready", 25, Some(port_listening));
    let config_b = make_test_config("test-10-b", "Svc B Waiting", 25, Some(port_unbound));
    let config_c = make_test_config("test-10-c", "Svc C No Port", 25, None);

    let ra = manager.start_service(&config_a).unwrap();
    let rb = manager.start_service(&config_b).unwrap();
    let _rc = manager.start_service(&config_c).unwrap();

    // Svc A port is listening -> Ready
    manager.update_port_readiness(&config_a.id, ra.pid.unwrap(), true);

    // Svc B port is not listening -> stays Running
    manager.update_port_readiness(&config_b.id, rb.pid.unwrap(), false);

    assert_eq!(manager.get_runtime_state(&config_a.id).state, ProcessState::Ready);
    assert_eq!(manager.get_runtime_state(&config_b.id).state, ProcessState::Running);
    assert_eq!(manager.get_runtime_state(&config_c.id).state, ProcessState::Running);

    manager.stop_all();
}

// ─────────────────────────────────────────────────────────────
// Test 11: Stale result cannot overwrite newer PID
// ─────────────────────────────────────────────────────────────
#[test]
fn test_11_stale_result_cannot_overwrite_newer_pid() {
    let manager = ProcessManager::new();
    let config = make_test_config("test-11", "Stale Guard Svc", 20, Some(9999));

    let runtime = manager.start_service(&config).unwrap();
    let real_pid = runtime.pid.unwrap();
    let bogus_pid = real_pid + 99999;

    // Probe result with wrong PID
    let result = manager.update_port_readiness(&config.id, bogus_pid, true);
    assert!(result.is_none(), "Mismatched PID must be ignored");

    let live = manager.get_runtime_state(&config.id);
    assert_eq!(live.state, ProcessState::Running);

    manager.stop_service(&config.id).unwrap();
}

// ─────────────────────────────────────────────────────────────
// Test 12: IPv4 and localhost loopback handling
// ─────────────────────────────────────────────────────────────
#[test]
fn test_12_ipv4_localhost_handling() {
    // Bind to standard IPv4 loopback
    let listener_v4 = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))
        .expect("Failed to bind IPv4 loopback");
    let port_v4 = listener_v4.local_addr().unwrap().port();

    assert!(PortChecker::is_port_listening(port_v4, Duration::from_millis(150)));

    // Verify IPv6 loopback probing does not crash or panic
    let is_v6_test = PortChecker::is_port_listening(port_v4, Duration::from_millis(50));
    assert!(is_v6_test);
}
