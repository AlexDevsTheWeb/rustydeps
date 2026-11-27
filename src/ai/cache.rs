//! On-disk cache for generated per-library summaries.
//!
//! Summaries live at
//! `~/.cache/rustydeps/changelogs/{ecosystem}_{package}_{target_version}.json`.
//! A cache hit costs no network request and no tokens, and the same version
//! shared by several projects is analyzed once.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config;
use crate::model::Ecosystem;

/// One cached package analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedSummary {
    /// Ecosystem the package belongs to.
    pub ecosystem: Ecosystem,
    /// Package name.
    pub package: String,
    /// Target version the analysis describes.
    pub target_version: String,
    /// Model that produced the analysis.
    pub model: String,
    /// Markdown analysis.
    pub summary: String,
    /// Unix timestamp (seconds) when the entry was written.
    pub created_at: u64,
}

/// A per-package summary cache rooted at the platform cache directory.
#[derive(Debug, Clone)]
pub struct SummaryCache {
    root: Option<PathBuf>,
    enabled: bool,
}

impl SummaryCache {
    /// Build a cache from the resolved configuration flag.
    pub fn new(enabled: bool) -> Self {
        Self {
            root: config::cache_dir(),
            enabled,
        }
    }

    /// Build a cache at an explicit root (tests).
    #[cfg(test)]
    pub fn with_root(enabled: bool, root: PathBuf) -> Self {
        Self {
            root: Some(root),
            enabled,
        }
    }

    /// Whether the cache can be read and written.
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.root.is_some()
    }

    /// Path for one cache entry, or `None` when caching is disabled.
    pub fn path_for(
        &self,
        ecosystem: Ecosystem,
        package: &str,
        target_version: &str,
    ) -> Option<PathBuf> {
        if !self.is_enabled() {
            return None;
        }

        let file = format!("{}.json", cache_key(ecosystem, package, target_version));
        self.root.as_ref().map(|root| root.join(file))
    }

    /// Load a matching, non-empty cache entry. Corrupt entries are ignored.
    pub fn load(
        &self,
        ecosystem: Ecosystem,
        package: &str,
        target_version: &str,
    ) -> Option<CachedSummary> {
        let path = self.path_for(ecosystem, package, target_version)?;
        let contents = fs::read_to_string(path).ok()?;
        let cached: CachedSummary = serde_json::from_str(&contents).ok()?;

        let matches = cached.ecosystem == ecosystem
            && cached.package == package
            && cached.target_version == target_version
            && !cached.summary.trim().is_empty();

        matches.then_some(cached)
    }

    /// Write one cache entry, creating the cache directory when needed.
    pub fn store(&self, cached: &CachedSummary) -> std::io::Result<()> {
        let Some(path) = self.path_for(cached.ecosystem, &cached.package, &cached.target_version)
        else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(cached)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        fs::write(path, json)
    }

    /// Delete every cached entry and return how many files were removed.
    ///
    /// A missing cache directory is not an error; it simply yields zero.
    pub fn clear(&self) -> std::io::Result<usize> {
        let Some(root) = &self.root else {
            return Ok(0);
        };

        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(error),
        };

        let mut removed = 0;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path)?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}

/// Filesystem-safe cache key: `{ecosystem}_{package}_{target_version}`.
pub fn cache_key(ecosystem: Ecosystem, package: &str, target_version: &str) -> String {
    format!(
        "{}_{}_{}",
        ecosystem.label(),
        sanitize(package),
        sanitize(target_version)
    )
}

/// Current Unix timestamp in seconds, or `0` if the clock is before the epoch.
pub fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Reduce a package/version string to lowercase `[a-z0-9._-]`, collapsing
/// separators so scoped npm names and Go module paths cannot escape the cache
/// directory.
fn sanitize(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut separator = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
            output.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator {
            output.push('-');
            separator = true;
        }
    }

    let trimmed = output.trim_matches('-');
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached() -> CachedSummary {
        CachedSummary {
            ecosystem: Ecosystem::Cargo,
            package: "tokio".to_string(),
            target_version: "2.0.0".to_string(),
            model: "gpt-4o-mini".to_string(),
            summary: "### 🚨 Breaking Changes\nUse `tokio::runtime`.".to_string(),
            created_at: now_unix_seconds(),
        }
    }

    #[test]
    fn cache_keys_follow_the_documented_format_and_are_safe() {
        assert_eq!(
            cache_key(Ecosystem::Cargo, "tokio", "2.0.0"),
            "cargo_tokio_2.0.0"
        );
        assert_eq!(
            cache_key(Ecosystem::Npm, "@scope/pkg", "1.2.3"),
            "npm_scope-pkg_1.2.3"
        );
        assert_eq!(
            cache_key(Ecosystem::Go, "github.com/gin-gonic/gin", "v1.10.0"),
            "go_github.com-gin-gonic-gin_v1.10.0"
        );
        assert!(!cache_key(Ecosystem::Cargo, "../../etc/passwd", "../1.0").contains('/'));
    }

    #[test]
    fn round_trips_a_summary() {
        let directory = tempfile::tempdir().expect("tempdir");
        let cache = SummaryCache::with_root(true, directory.path().to_path_buf());
        let entry = cached();

        cache.store(&entry).expect("store");
        let loaded = cache
            .load(Ecosystem::Cargo, "tokio", "2.0.0")
            .expect("cache hit");

        assert_eq!(loaded, entry);
        assert!(cache
            .path_for(Ecosystem::Cargo, "tokio", "2.0.0")
            .expect("path")
            .exists());
    }

    #[test]
    fn ignores_mismatched_and_corrupt_entries() {
        let directory = tempfile::tempdir().expect("tempdir");
        let cache = SummaryCache::with_root(true, directory.path().to_path_buf());
        cache.store(&cached()).expect("store");

        assert!(cache.load(Ecosystem::Cargo, "serde", "2.0.0").is_none());
        assert!(cache.load(Ecosystem::Npm, "tokio", "2.0.0").is_none());

        let path = cache
            .path_for(Ecosystem::Cargo, "tokio", "2.0.0")
            .expect("path");
        fs::write(path, "{ not json").expect("corrupt");
        assert!(cache.load(Ecosystem::Cargo, "tokio", "2.0.0").is_none());
    }

    #[test]
    fn a_disabled_cache_never_touches_disk() {
        let directory = tempfile::tempdir().expect("tempdir");
        let cache = SummaryCache::with_root(false, directory.path().to_path_buf());

        assert!(!cache.is_enabled());
        assert!(cache.path_for(Ecosystem::Cargo, "tokio", "2.0.0").is_none());
        cache.store(&cached()).expect("no-op store");
        assert!(cache.load(Ecosystem::Cargo, "tokio", "2.0.0").is_none());
        assert!(fs::read_dir(directory.path())
            .expect("read_dir")
            .next()
            .is_none());
    }

    #[test]
    fn clear_removes_every_cached_file_and_counts_them() {
        let directory = tempfile::tempdir().expect("tempdir");
        let cache = SummaryCache::with_root(true, directory.path().to_path_buf());
        cache.store(&cached()).expect("store first");

        let mut second = cached();
        second.package = "serde".to_string();
        cache.store(&second).expect("store second");

        assert_eq!(cache.clear().expect("clear"), 2);
        assert!(fs::read_dir(directory.path())
            .expect("read_dir")
            .next()
            .is_none());
        assert_eq!(cache.clear().expect("clear empty"), 0);
    }

    #[test]
    fn clear_of_a_missing_directory_is_zero() {
        let directory = tempfile::tempdir().expect("tempdir");
        let missing = directory.path().join("does-not-exist");
        let cache = SummaryCache::with_root(true, missing);

        assert_eq!(cache.clear().expect("clear"), 0);
    }
}
