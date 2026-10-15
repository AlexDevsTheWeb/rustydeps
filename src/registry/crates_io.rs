//! Client for the crates.io registry (`https://crates.io/api/v1/crates/{name}`).
//!
//! crates.io requires a descriptive `User-Agent`; requests without one are
//! rejected. The upgrade target defaults to the latest *stable* release:
//! `crate.max_stable_version` when present, otherwise the highest non-prerelease,
//! non-yanked entry in `versions`. A prerelease-only crate falls back to
//! `crate.max_version` so the scan still reports something.

use serde::Deserialize;

use crate::classifier::latest_stable;
use crate::model::Ecosystem;
use crate::registry::{RegistryClient, RegistryError};

/// Registry identifier used in error messages.
const REGISTRY_NAME: &str = "crates.io";

/// Base URL for the crates.io crate endpoint.
const API_BASE: &str = "https://crates.io/api/v1/crates";

/// crates.io registry client.
#[derive(Debug, Clone)]
pub struct CratesIoClient {
    http: reqwest::Client,
}

impl CratesIoClient {
    /// Build a client with the descriptive `User-Agent` crates.io requires.
    pub fn new() -> Result<Self, reqwest::Error> {
        let user_agent = format!(
            "{name}/{version} ({repository})",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            repository = env!("CARGO_PKG_REPOSITORY"),
        );
        let http = reqwest::Client::builder().user_agent(user_agent).build()?;
        Ok(Self { http })
    }
}

impl RegistryClient for CratesIoClient {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Cargo
    }

    async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError> {
        let url = format!("{API_BASE}/{name}");
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|source| http_error(name, source))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::NotFound {
                registry: REGISTRY_NAME,
                package: name.to_string(),
            });
        }

        let response = response
            .error_for_status()
            .map_err(|source| http_error(name, source))?;
        let body = response
            .text()
            .await
            .map_err(|source| http_error(name, source))?;

        parse_crates_io_response(&body, name)
    }
}

/// Wrap a transport error as a [`RegistryError::Http`].
fn http_error(package: &str, source: reqwest::Error) -> RegistryError {
    RegistryError::Http {
        registry: REGISTRY_NAME,
        package: package.to_string(),
        source,
    }
}

/// The crates.io response body, reduced to the fields RustyDeps reads.
#[derive(Debug, Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
    #[serde(default)]
    versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
struct CrateInfo {
    max_version: String,
    #[serde(default)]
    max_stable_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VersionEntry {
    num: String,
    #[serde(default)]
    yanked: bool,
}

/// Extract the latest stable version from a crates.io API response body.
pub fn parse_crates_io_response(body: &str, package: &str) -> Result<String, RegistryError> {
    let parsed: CrateResponse =
        serde_json::from_str(body).map_err(|err| RegistryError::InvalidResponse {
            registry: REGISTRY_NAME,
            package: package.to_string(),
            message: err.to_string(),
        })?;

    let invalid = |message: &str| RegistryError::InvalidResponse {
        registry: REGISTRY_NAME,
        package: package.to_string(),
        message: message.to_string(),
    };

    if let Some(stable) = parsed
        .krate
        .max_stable_version
        .as_deref()
        .filter(|version| !version.is_empty())
    {
        return Ok(stable.to_string());
    }

    let from_versions = latest_stable(
        parsed
            .versions
            .iter()
            .filter(|entry| !entry.yanked)
            .map(|entry| entry.num.as_str()),
    );
    if let Some(stable) = from_versions {
        return Ok(stable);
    }

    // No stable release published: fall back to the newest version, even if it
    // is a prerelease.
    if parsed.krate.max_version.is_empty() {
        return Err(invalid("missing `crate.max_version`"));
    }

    Ok(parsed.krate.max_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_max_stable_version() {
        let body = r#"{
            "crate": { "name": "serde", "max_version": "2.0.0-rc.1", "max_stable_version": "1.0.229" }
        }"#;

        assert_eq!(
            parse_crates_io_response(body, "serde").expect("version"),
            "1.0.229"
        );
    }

    #[test]
    fn falls_back_to_the_highest_stable_version() {
        let body = r#"{
            "crate": { "name": "serde", "max_version": "2.0.0-rc.1" },
            "versions": [
                { "num": "1.7.0" },
                { "num": "2.0.0-rc.1" },
                { "num": "1.9.0" },
                { "num": "1.8.0", "yanked": true }
            ]
        }"#;

        assert_eq!(
            parse_crates_io_response(body, "serde").expect("version"),
            "1.9.0"
        );
    }

    #[test]
    fn falls_back_to_max_version_when_only_prereleases_exist() {
        let body = r#"{
            "crate": { "name": "serde", "max_version": "0.1.0-alpha.1" },
            "versions": [ { "num": "0.1.0-alpha.1" } ]
        }"#;

        assert_eq!(
            parse_crates_io_response(body, "serde").expect("version"),
            "0.1.0-alpha.1"
        );
    }

    #[test]
    fn rejects_unexpected_shapes() {
        let error = parse_crates_io_response(r#"{"errors":[{"detail":"not found"}]}"#, "nope")
            .expect_err("missing `crate` should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));

        let error =
            parse_crates_io_response("not json", "nope").expect_err("invalid json should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }

    #[test]
    fn rejects_a_response_with_no_version_at_all() {
        let body = r#"{ "crate": { "max_version": "" } }"#;
        let error = parse_crates_io_response(body, "empty").expect_err("no version should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }
}
