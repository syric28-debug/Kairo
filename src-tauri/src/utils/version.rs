//! Phase 12 — Semantic version parsing and comparison for the KAIRO updater.
//!
//! Versions are never compared as plain strings. This module provides an
//! independent, dependency-free implementation used to validate release
//! metadata and covered by automated tests, so update-availability logic can
//! be verified without any network access.

use std::cmp::Ordering;

/// A parsed semantic version (`major.minor.patch`).
///
/// Pre-release and build suffixes (e.g. `1.0.0-rc.1`) are accepted on input
/// but ignored for ordering — KAIRO's stable channel only ever ships plain
/// `X.Y.Z` releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemanticVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parses `1.0.0`, `v1.1.0`, `1.10.0`, `2.0.0` (an optional leading `v`
    /// and an optional pre-release/build suffix are tolerated).
    ///
    /// Returns `None` for malformed input: wrong part count, empty parts,
    /// non-numeric parts, or values exceeding `u64`.
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let without_tag = trimmed
            .strip_prefix('v')
            .or_else(|| trimmed.strip_prefix('V'))
            .unwrap_or(trimmed);

        // Drop any pre-release/build metadata suffix ("-" or "+" onwards).
        let core = without_tag.split(['-', '+']).next()?;

        let mut parts = core.split('.');
        let major = parse_numeric_part(parts.next()?)?;
        let minor = parse_numeric_part(parts.next()?)?;
        let patch = parse_numeric_part(parts.next()?)?;

        // Exactly three components — reject "1.0", "1.0.0.0".
        if parts.next().is_some() {
            return None;
        }

        Some(Self {
            major,
            minor,
            patch,
        })
    }

    /// Three-way semantic ordering (major → minor → patch).
    pub fn compare(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
    }
}

fn parse_numeric_part(part: &str) -> Option<u64> {
    if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    part.parse::<u64>().ok()
}

/// Compares two version strings with proper semantic ordering.
///
/// Returns `None` when either input is malformed.
pub fn compare_versions(a: &str, b: &str) -> Option<Ordering> {
    Some(SemanticVersion::parse(a)?.compare(&SemanticVersion::parse(b)?))
}

/// Returns `true` when `candidate` is strictly newer than `current`.
///
/// Malformed input on either side conservatively reports "no update".
pub fn is_newer_version(candidate: &str, current: &str) -> bool {
    matches!(
        compare_versions(candidate, current),
        Some(Ordering::Greater)
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_versions_compare_equal() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), Some(Ordering::Equal));
        assert_eq!(compare_versions("1.1.0", "1.1.0"), Some(Ordering::Equal));
        assert_eq!(compare_versions("2.0.0", "2.0.0"), Some(Ordering::Equal));
    }

    #[test]
    fn newer_minor_is_greater() {
        assert_eq!(compare_versions("1.1.0", "1.0.0"), Some(Ordering::Greater));
        assert_eq!(compare_versions("1.0.0", "1.1.0"), Some(Ordering::Less));
    }

    #[test]
    fn newer_major_is_greater() {
        assert_eq!(compare_versions("2.0.0", "1.9.9"), Some(Ordering::Greater));
        assert_eq!(compare_versions("1.9.9", "2.0.0"), Some(Ordering::Less));
    }

    #[test]
    fn numeric_components_are_not_compared_lexicographically() {
        // 10 > 9 numerically even though "10" < "9" as plain strings.
        assert_eq!(compare_versions("1.10.0", "1.9.0"), Some(Ordering::Greater));
        assert_eq!(compare_versions("1.2.0", "1.10.0"), Some(Ordering::Less));
    }

    #[test]
    fn patch_level_ordering() {
        assert_eq!(compare_versions("1.0.1", "1.0.0"), Some(Ordering::Greater));
        assert_eq!(compare_versions("1.0.0", "1.0.1"), Some(Ordering::Less));
    }

    #[test]
    fn leading_v_tag_is_tolerated() {
        assert_eq!(compare_versions("v1.1.0", "1.0.0"), Some(Ordering::Greater));
        assert_eq!(compare_versions("V2.0.0", "1.0.0"), Some(Ordering::Greater));
    }

    #[test]
    fn pre_release_suffix_is_tolerated_and_ignored() {
        assert_eq!(
            compare_versions("1.1.0-beta.1", "1.0.0"),
            Some(Ordering::Greater)
        );
        assert_eq!(
            compare_versions("1.0.0", "1.0.0+build.5"),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn is_newer_version_logic() {
        assert!(is_newer_version("1.1.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("2.0.0", "1.99.99"));
        assert!(is_newer_version("v1.10.0", "v1.9.9"));
    }

    #[test]
    fn malformed_versions_never_report_an_update() {
        assert_eq!(SemanticVersion::parse(""), None);
        assert_eq!(SemanticVersion::parse("abc"), None);
        assert_eq!(SemanticVersion::parse("1"), None);
        assert_eq!(SemanticVersion::parse("1.0"), None);
        assert_eq!(SemanticVersion::parse("1.0.0.0"), None);
        assert_eq!(SemanticVersion::parse("1..0"), None);
        assert_eq!(SemanticVersion::parse("1.x.0"), None);
        assert_eq!(SemanticVersion::parse("-1.0.0"), None);
        assert_eq!(SemanticVersion::parse("99999999999999999999.0.0"), None);
        assert!(!is_newer_version("garbage", "1.0.0"));
        assert!(!is_newer_version("1.1.0", "garbage"));
    }

    #[test]
    fn current_app_version_is_valid_semver() {
        // "Current version detection": the compiled-in app version must always
        // parse as a valid semantic version (never compared as a plain string).
        let parsed = SemanticVersion::parse(env!("CARGO_PKG_VERSION"));
        assert!(
            parsed.is_some(),
            "app version {} must be parseable semver",
            env!("CARGO_PKG_VERSION")
        );
        // Sanity: the currently installed line (1.0.x) is "updatable" by a 1.1.0.
        assert!(is_newer_version("1.1.0", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn whitespace_is_tolerated() {
        assert_eq!(
            SemanticVersion::parse("  1.2.3  "),
            Some(SemanticVersion::new(1, 2, 3))
        );
    }
}
