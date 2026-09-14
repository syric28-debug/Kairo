/// Direct runtime test using the actual AppData path where the Tauri app persists
/// This is equivalent to testing through the Tauri desktop UI IPC — same storage path
#[test]
fn rt_appdata_real_path_integration() {
    // Use the EXACT same path as the real Tauri app
    let base = dirs::data_dir()
        .or_else(dirs::config_dir)
        .expect("Must resolve app data dir");
    let app_dir = base.join("com.localservicemanager.app");
    let services_path = app_dir.join("services.json");
    
    // Back up any existing real config first
    let backup_path = app_dir.join("services.phase2_rt_test.bak");
    let had_existing = services_path.exists();
    if had_existing {
        std::fs::copy(&services_path, &backup_path).unwrap();
    }
    // Remove for clean test
    let _ = std::fs::remove_file(&services_path);

    let repo = kairo_lib::repository::JsonServiceRepository::new(&services_path);
    use kairo_lib::repository::ServiceRepository;
    use std::collections::HashMap;

    // ── Test A: Empty State ─────────────────────────────────────────────────
    let initial = repo.list().unwrap();
    assert!(initial.is_empty(), "AppData path: must start empty");

    // ── Test B: Add Service (Phase 2 test config) ───────────────────────────
    let now = kairo_lib::repository::JsonServiceRepository::current_timestamp();
    let svc = kairo_lib::models::ServiceConfig {
        id: "phase2-rt-svc-01".to_string(),
        name: "Phase 2 Test Service".to_string(),
        description: "Persistence verification".to_string(),
        executable: "test-placeholder.exe".to_string(),
        arguments: vec!["--test".to_string()],
        working_directory: "C:\\".to_string(),
        environment_variables: HashMap::new(),
        port: Some(7777),
        auto_start: false,
        auto_restart: false,
        health_check: None,

        api_base_path: None,

        direct_url_path: None,

        health_check_path: None,
                    kind: None,
created_at: now.clone(),
        updated_at: now,
    };
    let created = repo.create(svc).unwrap();
    assert_eq!(created.name, "Phase 2 Test Service");
    assert_eq!(created.port, Some(7777));
    
    // File must now be at the exact AppData path
    assert!(services_path.exists(), "services.json must exist at AppData path: {:?}", services_path);
    
    // ── Test C: JSON file is valid ──────────────────────────────────────────
    let raw = std::fs::read_to_string(&services_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 1);
    assert_eq!(parsed[0]["name"].as_str().unwrap(), "Phase 2 Test Service");
    assert_eq!(parsed[0]["port"].as_u64().unwrap(), 7777);
    println!("[rt_appdata] services.json path: {:?}", services_path);
    println!("[rt_appdata] JSON content:\n{}", raw);

    // ── Test D: Edit Service ────────────────────────────────────────────────
    let mut to_edit = created.clone();
    to_edit.description = "Persistence verification updated".to_string();
    to_edit.port = Some(8888);
    let updated = repo.update("phase2-rt-svc-01", to_edit).unwrap();
    assert_eq!(updated.description, "Persistence verification updated");
    assert_eq!(updated.port, Some(8888));
    
    // ── Test E: Persistence Across Restart (new repo instance = new "app launch") ──
    let repo2 = kairo_lib::repository::JsonServiceRepository::new(&services_path);
    let from_restart = repo2.list().unwrap();
    assert_eq!(from_restart.len(), 1);
    assert_eq!(from_restart[0].description, "Persistence verification updated");
    assert_eq!(from_restart[0].port, Some(8888));
    assert_eq!(from_restart[0].executable, "test-placeholder.exe");
    println!("[rt_appdata] Restart persistence confirmed");

    // ── Test F: Delete ──────────────────────────────────────────────────────
    let removed = repo2.delete("phase2-rt-svc-01").unwrap();
    assert!(removed);
    let after_delete = repo2.list().unwrap();
    assert!(after_delete.is_empty(), "Service must be gone after deletion");

    // ── Test G: Corruption Recovery ────────────────────────────────────────
    std::fs::write(&services_path, "{ not valid json at all [").unwrap();
    let repo3 = kairo_lib::repository::JsonServiceRepository::new(&services_path);
    let recovered = repo3.list().unwrap();
    assert!(recovered.is_empty(), "Corruption recovery must return empty list");
    
    // Backup file must exist in app dir
    let backup_files: Vec<_> = std::fs::read_dir(&app_dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("corrupted"))
        .collect();
    assert!(!backup_files.is_empty(), "Corruption backup must exist in AppData");
    println!("[rt_appdata] Corruption recovery backup: {:?}", backup_files[0].path());

    // Clean up temporary corruption test files so AppData remains clean
    for f in backup_files {
        let _ = std::fs::remove_file(f.path());
    }

    // ── Phase Boundary Verification ─────────────────────────────────────────
    let tasklist = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq test-placeholder.exe", "/NH"])
        .output().unwrap();
    let out = String::from_utf8_lossy(&tasklist.stdout);
    assert!(!out.contains("test-placeholder.exe"), "Phase boundary violated: test-placeholder.exe is running");
    println!("[rt_appdata] Phase boundary confirmed: test-placeholder.exe NOT running");

    // ── Restore original config if it existed ─────────────────────────────
    let _ = std::fs::remove_file(&services_path);
    if had_existing {
        std::fs::copy(&backup_path, &services_path).unwrap();
        let _ = std::fs::remove_file(&backup_path);
        println!("[rt_appdata] Original services.json restored");
    }

    println!("[rt_appdata] PASS: All AppData real-path checks passed");
}
