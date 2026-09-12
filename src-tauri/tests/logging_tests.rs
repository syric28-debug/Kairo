use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use kairo_lib::models::log::{LogEntry, LogStream};
use kairo_lib::models::ServiceConfig;
use kairo_lib::repository::json_repository::JsonServiceRepository;
use kairo_lib::repository::ServiceRepository;
use kairo_lib::services::auto_restart::AutoRestartTracker;
use kairo_lib::services::{LogManager, ProcessManager};

fn temp_repo_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("lsm_test_logging")
        .join(name)
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("services.json")
}

fn make_service_config(id: &str, name: &str, exe: &str, args: Vec<&str>) -> ServiceConfig {
    let now = JsonServiceRepository::current_timestamp();
    ServiceConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: "Logging test service".to_string(),
        executable: exe.to_string(),
        arguments: args.into_iter().map(|s| s.to_string()).collect(),
        working_directory: "C:\\".to_string(),
        environment_variables: HashMap::new(),
        port: None,
        auto_start: false,
        auto_restart: false,
        health_check: None,
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Helper to poll until a condition is met or timeout expires.
fn wait_until<F>(timeout: Duration, interval: Duration, condition: F) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        thread::sleep(interval);
    }
    condition()
}

/// 1. stdout lines become LogEntry with stream: Stdout
#[test]
fn test_01_stdout_capture() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let svc = make_service_config(
        "test-log-stdout-01",
        "Stdout Test",
        "cmd.exe",
        vec!["/C", "echo Hello stdout 1& echo Hello stdout 2"],
    );

    let res = pm.start_service(&svc);
    assert!(res.is_ok(), "Service should start: {:?}", res.err());

    let has_logs = wait_until(Duration::from_secs(4), Duration::from_millis(50), || {
        let logs = log_manager.get_logs("test-log-stdout-01");
        logs.len() >= 2
    });

    assert!(has_logs, "Logs should have arrived in buffer");
    let logs = log_manager.get_logs("test-log-stdout-01");
    assert_eq!(logs[0].stream, LogStream::Stdout);
    assert_eq!(logs[0].message, "Hello stdout 1");
    assert_eq!(logs[1].stream, LogStream::Stdout);
    assert_eq!(logs[1].message, "Hello stdout 2");
    assert_eq!(logs[0].service_id, "test-log-stdout-01");
    assert!(!logs[0].id.is_empty());
    assert!(!logs[0].timestamp.is_empty());

    let _ = pm.stop_service("test-log-stdout-01");
}

/// 2. stderr lines become LogEntry with stream: Stderr
#[test]
fn test_02_stderr_capture() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let svc = make_service_config(
        "test-log-stderr-01",
        "Stderr Test",
        "powershell.exe",
        vec![
            "-NoProfile",
            "-Command",
            "[Console]::Error.WriteLine('Error line 1'); [Console]::Error.WriteLine('Error line 2')",
        ],
    );

    let res = pm.start_service(&svc);
    assert!(res.is_ok(), "Service should start");

    let has_logs = wait_until(Duration::from_secs(6), Duration::from_millis(50), || {
        let logs = log_manager.get_logs("test-log-stderr-01");
        logs.len() >= 2
    });

    assert!(has_logs, "Stderr logs should be captured");
    let logs = log_manager.get_logs("test-log-stderr-01");
    assert_eq!(logs[0].stream, LogStream::Stderr);
    assert_eq!(logs[0].message, "Error line 1");
    assert_eq!(logs[1].stream, LogStream::Stderr);
    assert_eq!(logs[1].message, "Error line 2");

    let _ = pm.stop_service("test-log-stderr-01");
}

/// 3. Service log isolation: Service A logs never appear in Service B's buffer
#[test]
fn test_03_service_log_isolation() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let svc_a = make_service_config(
        "svc-isolation-A",
        "Service A",
        "cmd.exe",
        vec!["/C", "echo Alpha Output"],
    );
    let svc_b = make_service_config(
        "svc-isolation-B",
        "Service B",
        "cmd.exe",
        vec!["/C", "echo Beta Output"],
    );

    pm.start_service(&svc_a).unwrap();
    pm.start_service(&svc_b).unwrap();

    let logs_ready = wait_until(Duration::from_secs(4), Duration::from_millis(50), || {
        !log_manager.get_logs("svc-isolation-A").is_empty()
            && !log_manager.get_logs("svc-isolation-B").is_empty()
    });
    assert!(logs_ready, "Logs from both services should be captured");

    let logs_a = log_manager.get_logs("svc-isolation-A");
    let logs_b = log_manager.get_logs("svc-isolation-B");

    for entry in &logs_a {
        assert_eq!(entry.service_id, "svc-isolation-A");
        assert!(!entry.message.contains("Beta"));
    }

    for entry in &logs_b {
        assert_eq!(entry.service_id, "svc-isolation-B");
        assert!(!entry.message.contains("Alpha"));
    }

    let _ = pm.stop_service("svc-isolation-A");
    let _ = pm.stop_service("svc-isolation-B");
}

/// 4. Bounded log buffer: buffer caps at max capacity and evicts oldest FIFO
#[test]
fn test_04_bounded_log_buffer() {
    let custom_limit = 5;
    let log_manager = Arc::new(LogManager::with_capacity(custom_limit));

    for i in 1..=10 {
        let entry = LogEntry::new(
            "test-bounded-svc",
            LogStream::Stdout,
            format!("Message number {}", i),
        );
        log_manager.append(entry);
    }

    let logs = log_manager.get_logs("test-bounded-svc");
    assert_eq!(logs.len(), custom_limit, "Buffer must not exceed limit");
    // Oldest entries 1..=5 should have been evicted; remaining are 6..=10
    assert_eq!(logs[0].message, "Message number 6");
    assert_eq!(logs[4].message, "Message number 10");
}

/// 5. Clear service logs empties the target buffer without affecting other services
#[test]
fn test_05_clear_service_logs() {
    let log_manager = Arc::new(LogManager::new());

    log_manager.append(LogEntry::new("svc-1", LogStream::Stdout, "svc1 log"));
    log_manager.append(LogEntry::new("svc-2", LogStream::Stdout, "svc2 log"));

    assert_eq!(log_manager.get_logs("svc-1").len(), 1);
    assert_eq!(log_manager.get_logs("svc-2").len(), 1);

    log_manager.clear_logs("svc-1");

    assert_eq!(log_manager.get_logs("svc-1").len(), 0, "svc-1 should be cleared");
    assert_eq!(log_manager.get_logs("svc-2").len(), 1, "svc-2 should remain untouched");

    log_manager.clear_all_logs();
    assert_eq!(log_manager.get_logs("svc-2").len(), 0, "clear_all should clear svc-2");
}

/// 6. Log event payload contains correct schema: id, service_id, timestamp, stream, message
#[test]
fn test_06_log_event_payload() {
    let log_manager = Arc::new(LogManager::new());
    let captured_count = Arc::new(AtomicUsize::new(0));
    let last_captured = Arc::new(std::sync::Mutex::new(None));

    let count_clone = Arc::clone(&captured_count);
    let captured_clone = Arc::clone(&last_captured);

    log_manager.set_event_sink(move |entry: &LogEntry| {
        count_clone.fetch_add(1, Ordering::SeqCst);
        let mut guard = captured_clone.lock().unwrap();
        *guard = Some(entry.clone());
    });

    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));
    let svc = make_service_config(
        "test-event-payload",
        "Payload Test",
        "cmd.exe",
        vec!["/C", "echo Event payload verified"],
    );

    pm.start_service(&svc).unwrap();

    let received = wait_until(Duration::from_secs(4), Duration::from_millis(50), || {
        captured_count.load(Ordering::SeqCst) >= 1
    });

    assert!(received, "Event sink must be invoked on stream read");
    let guard = last_captured.lock().unwrap();
    let entry = guard.as_ref().expect("Should have captured entry");

    assert_eq!(entry.service_id, "test-event-payload");
    assert_eq!(entry.stream, LogStream::Stdout);
    assert_eq!(entry.message, "Event payload verified");
    assert!(!entry.id.is_empty());
    assert!(!entry.timestamp.is_empty());

    let _ = pm.stop_service("test-event-payload");
}

/// 7. Process restart cleans up old reader and spawns fresh readers for new PID
#[test]
fn test_07_restart_creates_new_log_reader() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let svc = make_service_config(
        "test-restart-readers",
        "Restart Readers",
        "ping",
        vec!["127.0.0.1", "-n", "10"],
    );

    let run1 = pm.start_service(&svc).unwrap();
    let pid1 = run1.pid.unwrap();

    thread::sleep(Duration::from_millis(300));

    let run2 = pm.restart_service(&svc).unwrap();
    let pid2 = run2.pid.unwrap();

    assert_ne!(pid1, pid2, "Restart must generate a new PID");

    // Let new process emit output
    let has_logs = wait_until(Duration::from_secs(4), Duration::from_millis(100), || {
        !log_manager.get_logs("test-restart-readers").is_empty()
    });
    assert!(has_logs, "Logs should stream from the restarted process");

    pm.stop_service("test-restart-readers").unwrap();
}

/// 8. Auto restart log continuity: output from new PID continues under same service_id
#[test]
fn test_08_auto_restart_log_continuity() {
    let repo_file = temp_repo_path("auto_restart_continuity");
    let repo = Arc::new(JsonServiceRepository::new(&repo_file));

    let mut cfg = make_service_config(
        "svc-continuity",
        "Continuity Svc",
        "ping",
        vec!["127.0.0.1", "-n", "10"],
    );
    cfg.auto_restart = true;
    repo.create(cfg.clone()).unwrap();

    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));
    let run1 = pm.start_service(&cfg).unwrap();
    let pid1 = run1.pid.unwrap();

    // Wait for first ping line
    wait_until(Duration::from_secs(3), Duration::from_millis(50), || {
        !log_manager.get_logs("svc-continuity").is_empty()
    });
    let initial_count = log_manager.get_logs("svc-continuity").len();
    assert!(initial_count > 0, "Initial process must have emitted logs");

    // Externally kill child process to trigger unexpected exit
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid1.to_string()])
        .output();

    // Reconcile exit
    let mut tracker = AutoRestartTracker::new(3, Duration::from_secs(10), Duration::from_secs(30));
    let exited = pm.reconcile_unexpected_exits();
    assert_eq!(exited.len(), 1, "Must detect exit");

    // Auto-restart logic
    let should_restart = tracker.can_restart("svc-continuity");
    assert!(should_restart);
    tracker.record_attempt("svc-continuity");
    let run2 = pm.start_service(&cfg).unwrap();
    let pid2 = run2.pid.unwrap();
    assert_ne!(pid1, pid2);

    // Wait for subsequent logs under same service_id from new process
    let continued = wait_until(Duration::from_secs(4), Duration::from_millis(50), || {
        log_manager.get_logs("svc-continuity").len() > initial_count
    });

    assert!(continued, "Logs should continue accumulating from PID #2 under same service_id");
    let all_logs = log_manager.get_logs("svc-continuity");
    assert!(all_logs.len() > initial_count);

    let _ = pm.stop_service("svc-continuity");
    let _ = std::fs::remove_file(&repo_file);
}

/// 9. Stopping a service shuts down reader threads without hang or deadlock
#[test]
fn test_09_intentional_stop_log_reader_shutdown() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let svc = make_service_config(
        "test-stop-readers",
        "Stop Readers",
        "ping",
        vec!["127.0.0.1", "-n", "30"],
    );

    pm.start_service(&svc).unwrap();
    thread::sleep(Duration::from_millis(200));

    let stop_start = std::time::Instant::now();
    let res = pm.stop_service("test-stop-readers");
    let stop_duration = stop_start.elapsed();

    assert!(res.is_ok(), "Stop must succeed");
    assert!(
        stop_duration < Duration::from_secs(3),
        "Stopping service and joining reader threads must not hang (took {:?})",
        stop_duration
    );
}

/// 10. Intentional restart leaves exactly one reader per stream on the new process
#[test]
fn test_10_intentional_restart_no_duplicate_readers() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let svc = make_service_config(
        "test-no-dup-readers",
        "No Dup Readers",
        "cmd.exe",
        vec!["/C", "echo Single output message"],
    );

    pm.start_service(&svc).unwrap();
    wait_until(Duration::from_secs(3), Duration::from_millis(50), || {
        !log_manager.get_logs("test-no-dup-readers").is_empty()
    });

    log_manager.clear_logs("test-no-dup-readers");

    // Restart the service
    pm.restart_service(&svc).unwrap();
    wait_until(Duration::from_secs(3), Duration::from_millis(50), || {
        !log_manager.get_logs("test-no-dup-readers").is_empty()
    });

    thread::sleep(Duration::from_millis(200));

    let logs = log_manager.get_logs("test-no-dup-readers");
    // Should contain exactly 1 line, NOT duplicated
    assert_eq!(logs.len(), 1, "Should have exactly 1 log line after restart, got: {:?}", logs);
    assert_eq!(logs[0].message, "Single output message");

    let _ = pm.stop_service("test-no-dup-readers");
}

/// 11. Multiple concurrent services logging produce uncorrupted, isolated logs
#[test]
fn test_11_concurrent_services_logging() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    let count = 4;
    let mut svcs = Vec::new();
    for i in 0..count {
        let id = format!("concurrent-svc-{}", i);
        let svc = make_service_config(
            &id,
            &format!("Concurrent {}", i),
            "cmd.exe",
            vec!["/C", "echo Line 1& echo Line 2& echo Line 3"],
        );
        svcs.push(svc);
    }

    for svc in &svcs {
        pm.start_service(svc).unwrap();
    }

    let all_done = wait_until(Duration::from_secs(6), Duration::from_millis(50), || {
        svcs.iter().all(|s| log_manager.get_logs(&s.id).len() >= 3)
    });

    assert!(all_done, "All concurrent services should have emitted 3 lines each");

    for svc in &svcs {
        let logs = log_manager.get_logs(&svc.id);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].service_id, svc.id);
        let _ = pm.stop_service(&svc.id);
    }
}

/// 12. Process exiting with output lacking trailing newline still has final line captured
#[test]
fn test_12_final_partial_line_is_not_lost() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    // powershell command that writes without trailing newline
    let svc = make_service_config(
        "test-partial-line",
        "Partial Line",
        "powershell.exe",
        vec![
            "-NoProfile",
            "-Command",
            "[Console]::Out.Write('partial_without_newline')",
        ],
    );

    pm.start_service(&svc).unwrap();

    let captured = wait_until(Duration::from_secs(6), Duration::from_millis(100), || {
        !log_manager.get_logs("test-partial-line").is_empty()
    });

    assert!(captured, "Partial line prior to process EOF must be captured");
    let logs = log_manager.get_logs("test-partial-line");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "partial_without_newline");

    let _ = pm.stop_service("test-partial-line");
}

/// 13. High-volume log stream does not block or crash process management
#[test]
fn test_13_noisy_service_does_not_break_manager() {
    let log_manager = Arc::new(LogManager::new());
    let pm = ProcessManager::with_log_manager(Arc::clone(&log_manager));

    // Emit 1,200 lines quickly
    let svc = make_service_config(
        "test-noisy-svc",
        "Noisy Svc",
        "cmd.exe",
        vec!["/C", "for /L %i in (1,1,1200) do @echo item %i"],
    );

    let res = pm.start_service(&svc);
    assert!(res.is_ok());

    // Wait for the flood of lines
    let flood_received = wait_until(Duration::from_secs(8), Duration::from_millis(100), || {
        log_manager.get_logs("test-noisy-svc").len() >= 1000
    });

    assert!(flood_received, "Log manager should handle high volume stream");
    let logs = log_manager.get_logs("test-noisy-svc");
    assert_eq!(logs.len(), 1000, "Should be capped at default max capacity 1,000");

    let _ = pm.stop_service("test-noisy-svc");
}
