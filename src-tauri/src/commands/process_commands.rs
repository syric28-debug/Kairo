use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::models::process::ProcessRuntimeInfo;
use crate::repository::json_repository::JsonServiceRepository;
use crate::repository::ServiceRepository;
use crate::services::ProcessManager;

#[tauri::command]
pub fn start_service(
    id: String,
    manager: State<'_, ProcessManager>,
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<ProcessRuntimeInfo, AppError> {
    let config = repo
        .get(&id)?
        .ok_or_else(|| AppError::NotFound(format!("Service with ID '{}' not found", id)))?;

    manager.start_service(&config)
}

#[tauri::command]
pub fn stop_service(
    id: String,
    manager: State<'_, ProcessManager>,
) -> Result<ProcessRuntimeInfo, AppError> {
    manager.stop_service(&id)
}

#[tauri::command]
pub fn restart_service(
    id: String,
    manager: State<'_, ProcessManager>,
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<ProcessRuntimeInfo, AppError> {
    let config = repo
        .get(&id)?
        .ok_or_else(|| AppError::NotFound(format!("Service with ID '{}' not found", id)))?;

    manager.restart_service(&config)
}

#[tauri::command]
pub fn get_service_runtime_state(
    id: String,
    manager: State<'_, ProcessManager>,
) -> Result<ProcessRuntimeInfo, AppError> {
    Ok(manager.get_runtime_state(&id))
}

#[tauri::command]
pub fn list_service_runtime_states(
    manager: State<'_, ProcessManager>,
) -> Result<Vec<ProcessRuntimeInfo>, AppError> {
    Ok(manager.list_runtime_states())
}
