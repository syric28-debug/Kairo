pub mod json_repository;
pub mod settings_repository;

use crate::error::AppError;
use crate::models::ServiceConfig;

pub use json_repository::JsonServiceRepository;
pub use settings_repository::JsonSettingsRepository;

/// Abstract repository interface for persisting and managing generic service configurations.
pub trait ServiceRepository: Send + Sync {
    /// Retrieve all configured services.
    fn list(&self) -> Result<Vec<ServiceConfig>, AppError>;

    /// Retrieve a single service configuration by its unique ID.
    fn get(&self, id: &str) -> Result<Option<ServiceConfig>, AppError>;

    /// Persist a newly created service configuration.
    fn create(&self, service: ServiceConfig) -> Result<ServiceConfig, AppError>;

    /// Update an existing service configuration.
    fn update(&self, id: &str, service: ServiceConfig) -> Result<ServiceConfig, AppError>;

    /// Delete a service configuration by ID. Returns true if removed, false if not found.
    fn delete(&self, id: &str) -> Result<bool, AppError>;
}
