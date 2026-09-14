pub mod commands;
pub mod error;
pub mod models;
pub mod repository;
pub mod services;
pub mod utils;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use repository::{JsonServiceRepository, JsonSettingsRepository};
use services::{LogManager, ProcessManager, ProcessMonitor, StartupOrchestrator};
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_manager = Arc::new(LogManager::default());
    let service_repo = JsonServiceRepository::default();
    let settings_repo = JsonSettingsRepository::default();
    let process_manager = ProcessManager::with_log_manager(Arc::clone(&log_manager));
    let pm_clone = process_manager.clone();

    // Wrap repos in Arc so they can be shared into background threads.
    let settings_repo_arc = Arc::new(settings_repo);
    let service_repo_arc = Arc::new(service_repo);
    let log_manager_setup = Arc::clone(&log_manager);

    // Global application exiting flag: false during normal runtime (closing window hides to tray),
    // set to true when the user intentionally clicks "Exit" from the tray menu.
    let is_exiting = Arc::new(AtomicBool::new(false));
    let is_exiting_window = Arc::clone(&is_exiting);
    let is_exiting_setup = Arc::clone(&is_exiting);

    // Clone arcs for the setup closure.
    let settings_arc_setup = Arc::clone(&settings_repo_arc);
    let service_arc_setup = Arc::clone(&service_repo_arc);

    tauri::Builder::default()
        // Manage state
        .manage(Arc::clone(&settings_repo_arc))
        .manage(Arc::clone(&service_repo_arc))
        .manage(Arc::clone(&log_manager))
        .manage(process_manager)
        .manage(Arc::clone(&is_exiting))
        // Phase 7: Close-to-tray window interception
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if !is_exiting_window.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let app_handle_logs = app_handle.clone();
            log_manager_setup.set_event_sink(move |entry| {
                if let Err(e) = app_handle_logs.emit(services::log_manager::SERVICE_LOG_APPENDED_EVENT, entry) {
                    eprintln!("Failed to emit service-log-appended: {}", e);
                }
            });

            // 1. Start ProcessMonitor — it references shared ProcessManager and JsonServiceRepository.
            let monitor = ProcessMonitor::start(
                app_handle.clone(),
                pm_clone.clone(),
                Arc::clone(&service_arc_setup),
            );
            app.manage(monitor);

            // 2. Setup System Tray and context menu
            let _tray = utils::tray::setup_tray(
                &app_handle,
                Arc::clone(&service_arc_setup),
                pm_clone.clone(),
                Arc::clone(&is_exiting_setup),
            )?;

            // 3. Phase 12: Updater plugin — official GitHub Releases channel.
            //    Registration failure must never prevent KAIRO from starting;
            //    the UI degrades to "updates not configured" in that case.
            match app
                .handle()
                .plugin(tauri_plugin_updater::Builder::new().build())
            {
                Ok(_) => commands::update_commands::mark_updater_plugin_active(true),
                Err(e) => {
                    commands::update_commands::mark_updater_plugin_active(false);
                    eprintln!("[KAIRO] Updater plugin unavailable: {}", e);
                }
            }

            // 3b. Phase 12: Opener plugin — used only to open the official
            //     GitHub release page from the update UI.
            if let Err(e) = app.handle().plugin(tauri_plugin_opener::init()) {
                eprintln!("[KAIRO] Opener plugin unavailable: {}", e);
            }

            // 4. Register StartupOrchestrator as Tauri state.
            let orchestrator = StartupOrchestrator::new();
            app.manage(orchestrator);

            // 4. Retrieve the orchestrator from state and run it once.
            //    ProcessManager and ProcessMonitor are fully initialized at this point.
            let orch = app.state::<StartupOrchestrator>();
            orch.run_once(
                app_handle,
                pm_clone,
                Arc::clone(&settings_arc_setup),
                Arc::clone(&service_arc_setup),
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Phase 2-5 service lifecycle commands
            commands::list_services,
            commands::get_service,
            commands::create_service,
            commands::update_service,
            commands::delete_service,
            commands::start_service,
            commands::stop_service,
            commands::restart_service,
            commands::get_service_runtime_state,
            commands::list_service_runtime_states,
            // Phase 6 startup settings commands
            commands::get_startup_settings,
            commands::set_app_auto_start,
            commands::set_service_auto_start,
            // Phase 9 logging commands
            commands::get_service_logs,
            commands::clear_service_logs,
            commands::clear_all_service_logs,
            // Phase 12 updater commands
            commands::get_updater_status,
            commands::restart_app,
            commands::set_update_auto_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
