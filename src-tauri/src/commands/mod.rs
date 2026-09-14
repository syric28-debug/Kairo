pub mod log_commands;
pub mod process_commands;
pub mod settings_commands;
pub mod update_commands;
pub use log_commands::*;
pub use process_commands::*;
pub use settings_commands::*;
pub use update_commands::*;

use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{CreateServiceDto, ServiceConfig, UpdateServiceDto};
use crate::repository::json_repository::JsonServiceRepository;
use crate::repository::ServiceRepository;

/// Helper to get current timestamp.
fn now() -> String {
    JsonServiceRepository::current_timestamp()
}

#[tauri::command]
pub fn list_services(
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<Vec<ServiceConfig>, AppError> {
    repo.list()
}

#[tauri::command]
pub fn get_service(
    id: String,
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<ServiceConfig, AppError> {
    repo.get(&id)?
        .ok_or_else(|| AppError::NotFound(format!("Service '{}' not found", id)))
}

#[tauri::command]
pub fn create_service(
    dto: CreateServiceDto,
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<ServiceConfig, AppError> {
    let ts = now();
    let service = ServiceConfig {
        id: Uuid::new_v4().to_string(),
        name: dto.name,
        description: dto.description.unwrap_or_default(),
        executable: dto.executable,
        arguments: dto.arguments.unwrap_or_default(),
        working_directory: dto.working_directory.unwrap_or_else(|| "C:\\".into()),
        environment_variables: dto.environment_variables.unwrap_or_default(),
        port: dto.port,
        auto_start: dto.auto_start.unwrap_or(false),
        auto_restart: dto.auto_restart.unwrap_or(false),
        health_check: dto.health_check,
        kind: dto.kind,
        api_base_path: dto.api_base_path,
        direct_url_path: dto.direct_url_path,
        health_check_path: dto.health_check_path,
        created_at: ts.clone(),
        updated_at: ts,
    };
    repo.create(service)
}

#[tauri::command]
pub fn update_service(
    id: String,
    dto: UpdateServiceDto,
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<ServiceConfig, AppError> {
    let ts = now();
    let service = ServiceConfig {
        id: id.clone(),
        name: dto.name,
        description: dto.description.unwrap_or_default(),
        executable: dto.executable,
        arguments: dto.arguments.unwrap_or_default(),
        working_directory: dto.working_directory.unwrap_or_else(|| "C:\\".into()),
        environment_variables: dto.environment_variables.unwrap_or_default(),
        port: dto.port,
        auto_start: dto.auto_start.unwrap_or(false),
        auto_restart: dto.auto_restart.unwrap_or(false),
        health_check: dto.health_check,
        kind: dto.kind,
        api_base_path: dto.api_base_path,
        direct_url_path: dto.direct_url_path,
        health_check_path: dto.health_check_path,
        created_at: String::new(), // preserved by repository
        updated_at: ts,
    };
    repo.update(&id, service)
}

#[tauri::command]
pub fn delete_service(
    id: String,
    repo: State<'_, Arc<JsonServiceRepository>>,
) -> Result<bool, AppError> {
    repo.delete(&id)
}
