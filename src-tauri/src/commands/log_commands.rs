use std::sync::Arc;
use tauri::State;

use crate::error::AppError;
use crate::models::LogEntry;
use crate::services::LogManager;

/// Retrieves the bounded in-memory log buffer for a specific service.
#[tauri::command]
pub fn get_service_logs(
    id: String,
    log_manager: State<'_, Arc<LogManager>>,
) -> Result<Vec<LogEntry>, AppError> {
    Ok(log_manager.get_logs(&id))
}

/// Clears the in-memory log buffer for a specific service.
#[tauri::command]
pub fn clear_service_logs(
    id: String,
    log_manager: State<'_, Arc<LogManager>>,
) -> Result<(), AppError> {
    log_manager.clear_logs(&id);
    Ok(())
}

/// Clears the in-memory log buffer across all tracked services.
#[tauri::command]
pub fn clear_all_service_logs(
    log_manager: State<'_, Arc<LogManager>>,
) -> Result<(), AppError> {
    log_manager.clear_all_logs();
    Ok(())
}
