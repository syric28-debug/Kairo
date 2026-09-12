use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::AppError;
use crate::models::process::{ProcessRuntimeInfo, ProcessState};
use crate::repository::json_repository::JsonServiceRepository;
use crate::repository::ServiceRepository;
use crate::services::process_monitor::SERVICE_RUNTIME_UPDATED_EVENT;
use crate::services::{ProcessManager, ProcessMonitor};

pub const TRAY_ID: &str = "main-tray";

/// Restores, unminimizes, and brings the primary application window to the foreground.
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Updates the system tray tooltip to reflect the current high-level service status.
pub fn update_tray_tooltip(app: &AppHandle, manager: &ProcessManager) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let states = manager.list_runtime_states();
        let running_count = states
            .iter()
            .filter(|s| s.state == ProcessState::Running || s.state == ProcessState::Ready)
            .count();
        let failed_count = states
            .iter()
            .filter(|s| s.state == ProcessState::Failed)
            .count();

        let tooltip = if states.is_empty() {
            "KAIRO".to_string()
        } else {
            format!(
                "KAIRO\n{} Running • {} Failed",
                running_count, failed_count
            )
        };

        let _ = tray.set_tooltip(Some(tooltip));
    }
}

/// Core logic: Starts all currently configured services that are stopped and valid.
/// Skips already running, ready, starting, or invalid services to prevent duplicates.
/// Returns results for each attempted start.
pub fn start_all_core(
    manager: &ProcessManager,
    repo: &JsonServiceRepository,
) -> Vec<Result<ProcessRuntimeInfo, AppError>> {
    let services = match repo.list() {
        Ok(s) => s,
        Err(e) => return vec![Err(AppError::Repository(e.to_string()))],
    };

    let mut results = Vec::new();
    for service in services {
        let current_runtime = manager.get_runtime_state(&service.id);
        if current_runtime.state == ProcessState::Stopped {
            if service.validate().is_ok() {
                results.push(manager.start_service(&service));
            }
        }
    }
    results
}

/// Core logic: Stops all actively tracked child processes via exact PID handles.
/// Does not terminate untracked, external, or system processes.
pub fn stop_all_core(manager: &ProcessManager) -> Vec<Result<ProcessRuntimeInfo, AppError>> {
    let targets = manager.get_monitored_targets();
    let mut results = Vec::new();
    for target in targets {
        results.push(manager.stop_service(&target.service_id));
    }
    results
}

/// Core logic: Restarts all configured services using the existing ProcessManager restart lifecycle.
/// Terminates running PIDs cleanly and launches fresh child processes.
pub fn restart_all_core(
    manager: &ProcessManager,
    repo: &JsonServiceRepository,
) -> Vec<Result<ProcessRuntimeInfo, AppError>> {
    let services = match repo.list() {
        Ok(s) => s,
        Err(e) => return vec![Err(AppError::Repository(e.to_string()))],
    };

    let mut results = Vec::new();
    for service in services {
        if service.validate().is_ok() {
            results.push(manager.restart_service(&service));
        }
    }
    results
}

/// Starts all stopped, valid services and emits runtime update events.
pub fn start_all_services(
    app: &AppHandle,
    manager: &ProcessManager,
    repo: &JsonServiceRepository,
) {
    let results = start_all_core(manager, repo);
    for res in results {
        match res {
            Ok(info) => {
                let _ = app.emit(SERVICE_RUNTIME_UPDATED_EVENT, &info);
            }
            Err(e) => {
                eprintln!("Tray Start All error: {}", e);
            }
        }
    }
    update_tray_tooltip(app, manager);
}

/// Stops all tracked services and emits runtime update events.
pub fn stop_all_services(app: &AppHandle, manager: &ProcessManager) {
    let results = stop_all_core(manager);
    for res in results {
        match res {
            Ok(info) => {
                let _ = app.emit(SERVICE_RUNTIME_UPDATED_EVENT, &info);
            }
            Err(e) => {
                eprintln!("Tray Stop All error: {}", e);
            }
        }
    }
    update_tray_tooltip(app, manager);
}

/// Restarts all configured services and emits runtime update events.
pub fn restart_all_services(
    app: &AppHandle,
    manager: &ProcessManager,
    repo: &JsonServiceRepository,
) {
    let results = restart_all_core(manager, repo);
    for res in results {
        match res {
            Ok(info) => {
                let _ = app.emit(SERVICE_RUNTIME_UPDATED_EVENT, &info);
            }
            Err(e) => {
                eprintln!("Tray Restart All error: {}", e);
            }
        }
    }
    update_tray_tooltip(app, manager);
}

/// Performs a clean application exit initiated by the user via the tray menu.
/// - Sets the `is_exiting` flag to allow the window close interceptor to pass through
/// - Gracefully stops the ProcessMonitor background thread
/// - Terminates the application without killing managed services unless explicitly required
pub fn exit_application(app: &AppHandle, is_exiting: &Arc<AtomicBool>) {
    is_exiting.store(true, Ordering::SeqCst);

    // Stop background process monitor thread cleanly
    if let Some(monitor) = app.try_state::<ProcessMonitor>() {
        monitor.stop();
    }

    // Terminate application process
    app.exit(0);
}

/// Initializes the system tray icon, menu, and event handlers.
/// Executed exactly once during application setup.
pub fn setup_tray(
    app: &AppHandle,
    service_repo: Arc<JsonServiceRepository>,
    process_manager: ProcessManager,
    is_exiting: Arc<AtomicBool>,
) -> Result<TrayIcon, AppError> {
    let open_item = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)
        .map_err(|e| AppError::Service(format!("Failed to create tray menu item 'open': {}", e)))?;
    let start_all_item = MenuItem::with_id(app, "start_all", "Start All", true, None::<&str>)
        .map_err(|e| AppError::Service(format!("Failed to create tray menu item 'start_all': {}", e)))?;
    let stop_all_item = MenuItem::with_id(app, "stop_all", "Stop All", true, None::<&str>)
        .map_err(|e| AppError::Service(format!("Failed to create tray menu item 'stop_all': {}", e)))?;
    let restart_all_item = MenuItem::with_id(app, "restart_all", "Restart All", true, None::<&str>)
        .map_err(|e| AppError::Service(format!("Failed to create tray menu item 'restart_all': {}", e)))?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| AppError::Service(format!("Failed to create tray separator: {}", e)))?;
    let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)
        .map_err(|e| AppError::Service(format!("Failed to create tray menu item 'exit': {}", e)))?;

    let menu = Menu::with_items(
        app,
        &[
            &open_item,
            &start_all_item,
            &stop_all_item,
            &restart_all_item,
            &separator,
            &exit_item,
        ],
    )
    .map_err(|e| AppError::Service(format!("Failed to build tray menu: {}", e)))?;

    let icon = {
        const TRAY_ICON_BYTES: &[u8] = include_bytes!("../../icons/tray-icon.png");
        tauri::image::Image::from_bytes(TRAY_ICON_BYTES)
            .map(|img| img.to_owned())
            .ok()
            .or_else(|| app.default_window_icon().cloned())
            .ok_or_else(|| AppError::Service("Tray icon could not be loaded".into()))?
    };

    let app_handle_menu = app.clone();
    let app_handle_icon = app.clone();
    let pm_menu = process_manager.clone();
    let repo_menu = Arc::clone(&service_repo);
    let is_exiting_menu = Arc::clone(&is_exiting);

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("KAIRO — Local Runtime Manager")
        .on_menu_event(move |_app, event| {
            let id_str = event.id.as_ref();
            match id_str {
                "open" => {
                    show_main_window(&app_handle_menu);
                }
                "start_all" => {
                    let app = app_handle_menu.clone();
                    let pm = pm_menu.clone();
                    let repo = Arc::clone(&repo_menu);
                    thread::spawn(move || {
                        start_all_services(&app, &pm, &repo);
                    });
                }
                "stop_all" => {
                    let app = app_handle_menu.clone();
                    let pm = pm_menu.clone();
                    thread::spawn(move || {
                        stop_all_services(&app, &pm);
                    });
                }
                "restart_all" => {
                    let app = app_handle_menu.clone();
                    let pm = pm_menu.clone();
                    let repo = Arc::clone(&repo_menu);
                    thread::spawn(move || {
                        restart_all_services(&app, &pm, &repo);
                    });
                }
                "exit" => {
                    exit_application(&app_handle_menu, &is_exiting_menu);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(&app_handle_icon);
            }
        })
        .build(app)
        .map_err(|e| AppError::Service(format!("Failed to build tray icon: {}", e)))?;

    Ok(tray)
}
