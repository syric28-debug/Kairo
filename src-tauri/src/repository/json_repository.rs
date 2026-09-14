use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::models::ServiceConfig;
use crate::repository::ServiceRepository;

/// JSON-backed implementation of `ServiceRepository`.
/// Uses atomic file writes and graceful corruption recovery.
pub struct JsonServiceRepository {
    storage_path: PathBuf,
    lock: RwLock<()>,
}

impl JsonServiceRepository {
    /// Creates a repository at a custom file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let storage_path = path.as_ref().to_path_buf();
        Self {
            storage_path,
            lock: RwLock::new(()),
        }
    }

    /// Resolves the default application data storage path:
    /// `%APPDATA%\com.kairo.localruntimemanager\services.json`
    /// Migrates existing services from legacy `%APPDATA%\com.localservicemanager.app\services.json` if present.
    pub fn default_path() -> PathBuf {
        let base_dir = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        let kairo_dir = base_dir.join("com.kairo.localruntimemanager");
        let kairo_path = kairo_dir.join("services.json");

        if !kairo_path.exists() {
            let legacy_path = base_dir
                .join("com.localservicemanager.app")
                .join("services.json");
            if legacy_path.exists() {
                let _ = fs::create_dir_all(&kairo_dir);
                let _ = fs::copy(&legacy_path, &kairo_path);
            }
        }

        kairo_path
    }

    /// Creates an instance using the standard application data path.
    pub fn default() -> Self {
        Self::new(Self::default_path())
    }

    /// Returns current timestamp string in ISO 8601 / RFC 3339 format.
    pub fn current_timestamp() -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        format!("{}.{:03}Z", secs, millis)
    }

    /// Reads services from disk.
    /// If the file does not exist, returns an empty vector.
    /// If the file is corrupted/malformed, creates a backup file and returns an empty vector safely.
    fn load_internal(&self) -> Result<Vec<ServiceConfig>, AppError> {
        if !self.storage_path.exists() {
            return Ok(Vec::new());
        }

        let content = match fs::read_to_string(&self.storage_path) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("[LSM Warning] Failed to read services file: {}", err);
                return Err(AppError::Io(err));
            }
        };

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        match serde_json::from_str::<Vec<ServiceConfig>>(&content) {
            Ok(services) => Ok(services),
            Err(err) => {
                eprintln!(
                    "[LSM Warning] services.json is corrupted: {}. Preserving backup and recovering safely.",
                    err
                );
                self.preserve_corrupted_backup(&content);
                Ok(Vec::new())
            }
        }
    }

    /// Backs up corrupted JSON file safely to `services.corrupted.<timestamp>.json`.
    fn preserve_corrupted_backup(&self, content: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let parent = self.storage_path.parent().unwrap_or_else(|| Path::new("."));
        let backup_name = format!("services.corrupted.{}.json", timestamp);
        let backup_path = parent.join(backup_name);

        if let Err(write_err) = fs::write(&backup_path, content) {
            eprintln!(
                "[LSM Error] Failed to write corrupted backup to {:?}: {}",
                backup_path, write_err
            );
        } else {
            eprintln!(
                "[LSM Info] Successfully preserved corrupted configuration backup at {:?}",
                backup_path
            );
        }
    }

    /// Performs an atomic write by saving to a `.tmp` file and renaming.
    fn save_internal(&self, services: &[ServiceConfig]) -> Result<(), AppError> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json_data = serde_json::to_string_pretty(services)?;

        // Write to temporary file in the same directory to guarantee atomic rename across the same filesystem
        let tmp_path = self.storage_path.with_extension("tmp");
        fs::write(&tmp_path, json_data)?;

        // Atomic replace
        fs::rename(&tmp_path, &self.storage_path)?;

        Ok(())
    }
}

impl ServiceRepository for JsonServiceRepository {
    fn list(&self) -> Result<Vec<ServiceConfig>, AppError> {
        let _guard = self.lock.read().unwrap();
        self.load_internal()
    }

    fn get(&self, id: &str) -> Result<Option<ServiceConfig>, AppError> {
        let _guard = self.lock.read().unwrap();
        let services = self.load_internal()?;
        Ok(services.into_iter().find(|s| s.id == id))
    }

    fn create(&self, service: ServiceConfig) -> Result<ServiceConfig, AppError> {
        service.validate()?;
        let _guard = self.lock.write().unwrap();
        let mut services = self.load_internal()?;

        if services.iter().any(|s| s.id == service.id) {
            return Err(AppError::Validation(format!(
                "Service with id '{}' already exists",
                service.id
            )));
        }

        services.push(service.clone());
        self.save_internal(&services)?;

        Ok(service)
    }

    fn update(&self, id: &str, mut service: ServiceConfig) -> Result<ServiceConfig, AppError> {
        service.validate()?;
        let _guard = self.lock.write().unwrap();
        let mut services = self.load_internal()?;

        let index = services.iter().position(|s| s.id == id).ok_or_else(|| {
            AppError::NotFound(format!("Service with id '{}' does not exist", id))
        })?;

        // Preserve created_at and ID
        let existing_created_at = services[index].created_at.clone();
        service.id = id.to_string();
        service.created_at = existing_created_at;
        service.updated_at = Self::current_timestamp();

        services[index] = service.clone();
        self.save_internal(&services)?;

        Ok(service)
    }

    fn delete(&self, id: &str) -> Result<bool, AppError> {
        let _guard = self.lock.write().unwrap();
        let mut services = self.load_internal()?;

        let initial_len = services.len();
        services.retain(|s| s.id != id);

        if services.len() != initial_len {
            self.save_internal(&services)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn temp_test_repo(name: &str) -> (JsonServiceRepository, PathBuf) {
        let temp_dir = std::env::temp_dir().join("lsm_tests").join(name);
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("services.json");
        (JsonServiceRepository::new(&file_path), file_path)
    }

    fn sample_service(id: &str, name: &str) -> ServiceConfig {
        ServiceConfig {
            id: id.to_string(),
            name: name.to_string(),
            description: "Sample test service".into(),
            executable: "C:\\test\\service.exe".into(),
            arguments: vec!["--port".into(), "8080".into()],
            working_directory: "C:\\test".into(),
            environment_variables: HashMap::new(),
            port: Some(8080),
            auto_start: true,
            auto_restart: false,
            health_check: None,
            kind: None,
            api_base_path: None,
            direct_url_path: None,
            health_check_path: None,
            created_at: JsonServiceRepository::current_timestamp(),
            updated_at: JsonServiceRepository::current_timestamp(),
        }
    }

    #[test]
    fn test_crud_operations() {
        let (repo, _path) = temp_test_repo("test_crud");

        // 1. Initially empty
        let initial = repo.list().unwrap();
        assert_eq!(initial.len(), 0);

        // 2. Create service
        let svc = sample_service("svc-1", "Gateway Service");
        let created = repo.create(svc).unwrap();
        assert_eq!(created.name, "Gateway Service");

        // 3. Get service
        let fetched = repo.get("svc-1").unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, "svc-1");

        // 4. Update service
        let mut updated_svc = created.clone();
        updated_svc.name = "Gateway Service Updated".into();
        updated_svc.port = Some(9090);
        let updated = repo.update("svc-1", updated_svc).unwrap();
        assert_eq!(updated.name, "Gateway Service Updated");
        assert_eq!(updated.port, Some(9090));

        // 5. List
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Gateway Service Updated");

        // 6. Delete
        let deleted = repo.delete("svc-1").unwrap();
        assert!(deleted);
        assert_eq!(repo.list().unwrap().len(), 0);
        assert!(repo.get("svc-1").unwrap().is_none());
    }

    #[test]
    fn test_atomic_persistence_and_restart() {
        let temp_dir = std::env::temp_dir().join("lsm_tests").join("test_persistence");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("services.json");

        // Session 1: Create
        {
            let repo = JsonServiceRepository::new(&file_path);
            let svc = sample_service("svc-persist", "Persistent Worker");
            repo.create(svc).unwrap();
        }

        // Verify file exists on disk
        assert!(file_path.exists());

        // Session 2: "Restart" application with new repository instance
        {
            let repo_restarted = JsonServiceRepository::new(&file_path);
            let list = repo_restarted.list().unwrap();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].name, "Persistent Worker");
            assert_eq!(list[0].id, "svc-persist");
        }
    }

    #[test]
    fn test_corrupted_json_recovery_does_not_crash() {
        let temp_dir = std::env::temp_dir().join("lsm_tests").join("test_corruption");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("services.json");

        // Write malformed corrupted JSON
        fs::write(&file_path, "{ malformed json: true, broken...").unwrap();

        // Repository should not crash, should backup and return empty list
        let repo = JsonServiceRepository::new(&file_path);
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 0);

        // Check that a backup file was created
        let entries = fs::read_dir(&temp_dir).unwrap();
        let backup_exists = entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains("services.corrupted"));
        assert!(backup_exists, "Corrupted backup file must be preserved");

        // Verify that we can now write new services safely
        let new_svc = sample_service("svc-recovered", "Recovered Service");
        let created = repo.create(new_svc).unwrap();
        assert_eq!(created.id, "svc-recovered");
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn test_validation_rules() {
        let (repo, _path) = temp_test_repo("test_validation");

        // Missing / empty name
        let mut invalid_name = sample_service("v-1", "");
        assert!(repo.create(invalid_name.clone()).is_err());

        invalid_name.name = "   ".into();
        assert!(repo.create(invalid_name).is_err());

        // Missing / empty executable
        let mut invalid_exec = sample_service("v-2", "Valid Name");
        invalid_exec.executable = "".into();
        assert!(repo.create(invalid_exec).is_err());

        // Invalid port (0 is invalid in our check)
        let mut invalid_port = sample_service("v-3", "Valid Name");
        invalid_port.port = Some(0);
        assert!(repo.create(invalid_port).is_err());

        // Name too long (> 100)
        let long_name = sample_service("v-4", &"A".repeat(101));
        assert!(repo.create(long_name).is_err());
    }
}
