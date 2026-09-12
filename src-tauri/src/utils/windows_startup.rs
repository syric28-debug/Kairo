/// Windows per-user auto-start registry integration.
///
/// Manages a single value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
/// to register or unregister the application for logon auto-start.
///
/// Rules:
/// - No `cmd.exe /c`, no `powershell -Command`, no shell strings of any kind.
/// - Operates on HKCU only — no elevation required, never modifies other keys.
/// - All operations are idempotent (repeated enable/disable calls are safe).
/// - Non-Windows platforms receive safe no-op stub implementations.

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "KAIRO";
const LEGACY_VALUE_NAME: &str = "LocalServiceManager";

// ─── Windows Implementation ─────────────────────────────────────────────────

#[cfg(windows)]
mod imp {
    use super::{RUN_KEY, VALUE_NAME};
    use crate::error::AppError;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    /// Returns `true` if the `KAIRO` value exists in the Run key.
    pub fn is_enabled() -> Result<bool, AppError> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags(RUN_KEY, KEY_READ)
            .map_err(|e| AppError::Config(format!("Failed to open Run registry key: {}", e)))?;

        let result: Result<String, _> = run_key.get_value(VALUE_NAME);
        Ok(result.is_ok())
    }

    /// Writes the current executable path to the Run key.
    /// Idempotent: safe to call even if the value already exists.
    pub fn enable() -> Result<(), AppError> {
        let exe_path = std::env::current_exe()
            .map_err(|e| AppError::Config(format!("Failed to resolve executable path: {}", e)))?;

        // Quote the path in case it contains spaces, matching Windows conventions.
        let exe_str = format!("\"{}\"", exe_path.display());

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run_key, _disp) = hkcu
            .create_subkey(RUN_KEY)
            .map_err(|e| AppError::Config(format!("Failed to open/create Run registry key: {}", e)))?;

        run_key
            .set_value(VALUE_NAME, &exe_str)
            .map_err(|e| AppError::Config(format!("Failed to write registry value: {}", e)))?;

        // Clean up legacy registry entry if present
        let _ = run_key.delete_value(super::LEGACY_VALUE_NAME);

        Ok(())
    }

    /// Removes the `KAIRO` value from the Run key.
    /// Also cleans up the legacy `LocalServiceManager` entry if present.
    /// Idempotent: safe to call even if the value does not exist.
    pub fn disable() -> Result<(), AppError> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
            Ok(k) => k,
            Err(_) => return Ok(()), // Key does not exist — nothing to remove.
        };

        // Remove current KAIRO entry
        match run_key.delete_value(VALUE_NAME) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
            Err(e) => return Err(AppError::Config(format!(
                "Failed to remove registry value: {}",
                e
            ))),
        }

        // Clean up legacy entry
        let _ = run_key.delete_value(super::LEGACY_VALUE_NAME);

        Ok(())
    }
}

// ─── Non-Windows Stubs ───────────────────────────────────────────────────────

#[cfg(not(windows))]
mod imp {
    use crate::error::AppError;

    pub fn is_enabled() -> Result<bool, AppError> {
        Ok(false)
    }

    pub fn enable() -> Result<(), AppError> {
        Ok(())
    }

    pub fn disable() -> Result<(), AppError> {
        Ok(())
    }
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Returns `true` if the application is registered to start with Windows.
pub fn is_app_auto_start_enabled() -> Result<bool, crate::error::AppError> {
    imp::is_enabled()
}

/// Registers the application to start automatically when the user logs in.
/// Safe to call repeatedly (idempotent).
pub fn enable_app_auto_start() -> Result<(), crate::error::AppError> {
    imp::enable()
}

/// Removes the application from Windows startup. Does nothing if not registered.
/// Safe to call repeatedly (idempotent).
pub fn disable_app_auto_start() -> Result<(), crate::error::AppError> {
    imp::disable()
}
