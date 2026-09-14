//! Phase 12 — Update-source configuration validation helpers.
//!
//! KAIRO's updater must only ever talk to the official GitHub Releases source
//! of this repository. These pure helpers are covered by unit tests so that
//! update-source safety rules can be verified without any network access.

/// The only host permitted to serve update metadata and update artifacts.
pub const OFFICIAL_UPDATE_HOST: &str = "github.com";

/// The official static update manifest asset published with every release.
pub const OFFICIAL_UPDATE_MANIFEST: &str = "latest.json";

/// Validates that an update endpoint is HTTPS and points at the official
/// KAIRO GitHub Releases source.
///
/// This mirrors (and additionally constrains) the plugin's own HTTPS-only
/// transport rule: no third-party update servers, no mirrors, no CDNs.
pub fn validate_update_endpoint(endpoint: &str) -> Result<(), String> {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return Err("Update endpoint is empty".into());
    }

    let rest = match trimmed.strip_prefix("https://") {
        Some(rest) => rest,
        None => {
            return Err(format!(
                "Update endpoint '{trimmed}' must use HTTPS"
            ))
        }
    };

    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();

    if authority.is_empty() {
        return Err("Update endpoint has no host".into());
    }
    if authority.contains('@') {
        return Err("Update endpoint must not embed credentials".into());
    }
    if !authority.eq_ignore_ascii_case(OFFICIAL_UPDATE_HOST) {
        return Err(format!(
            "Update endpoint host '{authority}' is not the official KAIRO update source ({OFFICIAL_UPDATE_HOST})"
        ));
    }

    Ok(())
}

/// Maps a CPU architecture identifier to the official Tauri updater platform
/// key used in the static `latest.json` manifest.
///
/// A 64-bit KAIRO installation always resolves to `windows-x86_64` and a
/// 32-bit installation to `windows-i686`, so an x86 install can never receive
/// an x64 update (and vice versa).
pub fn platform_key_for_arch(arch: &str) -> Option<&'static str> {
    match arch.trim().to_ascii_lowercase().as_str() {
        "x86_64" | "amd64" | "x64" => Some("windows-x86_64"),
        "i686" | "i386" | "x86" => Some("windows-i686"),
        _ => None,
    }
}

/// Detects the architecture of the running KAIRO binary (compile-time target).
pub fn detect_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "i686"
    } else {
        "unknown"
    }
}

/// Conservative check that a Tauri updater public key has actually been
/// configured in `tauri.conf.json`.
///
/// The plugin performs the real cryptographic validation when a check is
/// executed; this only decides whether the UI should report "updates are not
/// configured in this build" (e.g. a development build without the
/// production signing key configured).
///
/// Tauri updater public keys are the base64-encoded minisign public key file,
/// which always begins with the base64 encoding of
/// `untrusted comment: minisign public key` (`dW50cnVzdGVkIGNvbW1lbnQ6...`).
pub fn is_updater_pubkey_configured(pubkey: &str) -> bool {
    let key = pubkey.trim();
    key.starts_with("dW50cnVzdGVkIGNvbW1lbnQ6") && key.len() >= 64
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_github_endpoint_is_accepted() {
        assert!(validate_update_endpoint(
            "https://github.com/syric28-debug/Kairo/releases/latest/download/latest.json"
        )
        .is_ok());
    }

    #[test]
    fn https_is_required() {
        assert!(validate_update_endpoint(
            "http://github.com/syric28-debug/Kairo/releases/latest/download/latest.json"
        )
        .is_err());
        assert!(validate_update_endpoint("ftp://github.com/latest.json").is_err());
        assert!(validate_update_endpoint("").is_err());
        assert!(validate_update_endpoint("   ").is_err());
    }

    #[test]
    fn non_official_hosts_are_rejected() {
        assert!(validate_update_endpoint("https://evil.example.com/latest.json").is_err());
        assert!(validate_update_endpoint("https://github.com.evil.com/latest.json").is_err());
        assert!(
            validate_update_endpoint("https://raw.githubusercontent.com/x/y/latest.json").is_err()
        );
    }

    #[test]
    fn embedded_credentials_are_rejected() {
        assert!(validate_update_endpoint("https://user:pass@github.com/latest.json").is_err());
    }

    #[test]
    fn platform_keys_match_architecture() {
        assert_eq!(platform_key_for_arch("x86_64"), Some("windows-x86_64"));
        assert_eq!(platform_key_for_arch("AMD64"), Some("windows-x86_64"));
        assert_eq!(platform_key_for_arch("x64"), Some("windows-x86_64"));
        assert_eq!(platform_key_for_arch("i686"), Some("windows-i686"));
        assert_eq!(platform_key_for_arch("x86"), Some("windows-i686"));
        assert_eq!(platform_key_for_arch("arm64"), None);
        assert_eq!(platform_key_for_arch(""), None);
    }

    #[test]
    fn detected_arch_maps_to_a_platform_key_on_windows_targets() {
        let arch = detect_arch();
        let mapped = platform_key_for_arch(arch);
        if arch == "x86_64" || arch == "i686" {
            assert!(mapped.is_some());
        } else {
            assert_eq!(mapped, None);
        }
    }

    #[test]
    fn placeholder_pubkey_is_reported_as_unconfigured() {
        assert!(!is_updater_pubkey_configured(""));
        assert!(!is_updater_pubkey_configured("PLACEHOLDER"));
        assert!(!is_updater_pubkey_configured("dW50cnVzdGVkIGNvbW1lbnQ6"));
    }

    #[test]
    fn minisign_pubkey_shape_is_recognized() {
        // Shape of a real key: base64("untrusted comment: minisign public key <hex>")
        // followed by the base64-encoded key line.
        let fake = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDkwYTIwZGE1MDk2YmQwNzY3ZmEwNzc2ZTAwNzBhZTc0NAoK";
        assert!(is_updater_pubkey_configured(fake));
    }
}
