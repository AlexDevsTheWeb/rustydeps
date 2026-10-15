//! Client for PyPI (`https://pypi.org/pypi/{name}/json`).
//!
//! The latest stable release is taken from the `releases` map (skipping fully
//! yanked files and prereleases); if no stable release exists, `info.version`
//! (which may be a prerelease) is used as a fallback.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::classifier::latest_stable;
use crate::model::Ecosystem;
use crate::registry::{RegistryClient, RegistryError};

/// Registry identifier used in error messages.
const REGISTRY_NAME: &str = "PyPI";

/// Base URL for the PyPI JSON API.
const API_BASE: &str = "https://pypi.org/pypi";

/// PyPI registry client.
#[derive(Debug, Clone)]
pub struct PypiClient {
    http: reqwest::Client,
}

impl PypiClient {
    /// Build a client with a descriptive `User-Agent`.
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

impl RegistryClient for PypiClient {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Pypi
    }

    async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError> {
        let url = format!("{API_BASE}/{name}/json");
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

        parse_pypi_response(&body, name)
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

/// The PyPI response body, reduced to the fields RustyDeps reads.
#[derive(Debug, Deserialize)]
struct PypiResponse {
    info: Info,
    #[serde(default)]
    releases: BTreeMap<String, Vec<ReleaseFile>>,
}

#[derive(Debug, Deserialize)]
struct Info {
    version: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseFile {
    #[serde(default)]
    yanked: bool,
}

/// Extract the latest stable version from a PyPI JSON API response body.
pub fn parse_pypi_response(body: &str, package: &str) -> Result<String, RegistryError> {
    let parsed: PypiResponse =
        serde_json::from_str(body).map_err(|err| RegistryError::InvalidResponse {
            registry: REGISTRY_NAME,
            package: package.to_string(),
            message: err.to_string(),
        })?;

    let available = parsed
        .releases
        .iter()
        .filter(|(_, files)| files.iter().any(|file| !file.yanked))
        .map(|(version, _)| version.as_str());

    if let Some(stable) = latest_stable(available) {
        return Ok(stable);
    }

    if !parsed.info.version.is_empty() {
        return Ok(parsed.info.version);
    }

    Err(RegistryError::InvalidResponse {
        registry: REGISTRY_NAME,
        package: package.to_string(),
        message: "no releases found".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_latest_stable_release() {
        let body = r#"{
            "info": { "version": "2.31.0" },
            "releases": {
                "2.30.0": [{ "yanked": false }],
                "2.31.0": [{ "yanked": false }],
                "2.32.0b1": [{ "yanked": false }]
            }
        }"#;

        assert_eq!(
            parse_pypi_response(body, "requests").expect("version"),
            "2.31.0"
        );
    }

    #[test]
    fn skips_yanked_releases() {
        let body = r#"{
            "info": { "version": "1.0.0" },
            "releases": {
                "1.0.0": [{ "yanked": true }],
                "0.9.0": [{ "yanked": false }]
            }
        }"#;

        assert_eq!(parse_pypi_response(body, "pkg").expect("version"), "0.9.0");
    }

    #[test]
    fn falls_back_to_info_version_when_only_prereleases_exist() {
        let body = r#"{
            "info": { "version": "2.0.0rc1" },
            "releases": { "2.0.0rc1": [{ "yanked": false }] }
        }"#;

        assert_eq!(
            parse_pypi_response(body, "pkg").expect("version"),
            "2.0.0rc1"
        );
    }

    #[test]
    fn rejects_unexpected_shapes() {
        let error = parse_pypi_response("not json", "pkg").expect_err("invalid json should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }
}
