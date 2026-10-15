//! Client for Packagist (`https://repo.packagist.org/p2/{vendor}/{package}.json`).
//!
//! The `p2` metadata endpoint returns the package's versions; the latest stable
//! tag is chosen, skipping `dev-*` and prerelease tags.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::classifier::latest_stable;
use crate::model::Ecosystem;
use crate::registry::{RegistryClient, RegistryError};

/// Registry identifier used in error messages.
const REGISTRY_NAME: &str = "Packagist";

/// Base URL for the Packagist `p2` metadata API.
const PACKAGIST_BASE: &str = "https://repo.packagist.org/p2";

/// Packagist registry client.
#[derive(Debug, Clone)]
pub struct ComposerClient {
    http: reqwest::Client,
}

impl ComposerClient {
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

impl RegistryClient for ComposerClient {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }

    async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError> {
        let url = format!("{PACKAGIST_BASE}/{name}.json");
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

        parse_composer_response(&body, name)
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

/// The Packagist `p2` response body, reduced to the fields RustyDeps reads.
#[derive(Debug, Deserialize)]
struct P2Response {
    #[serde(default)]
    packages: BTreeMap<String, Vec<PackageVersion>>,
}

#[derive(Debug, Deserialize)]
struct PackageVersion {
    version: String,
}

/// Extract the latest stable version from a Packagist `p2` response body.
pub fn parse_composer_response(body: &str, package: &str) -> Result<String, RegistryError> {
    let parsed: P2Response =
        serde_json::from_str(body).map_err(|err| RegistryError::InvalidResponse {
            registry: REGISTRY_NAME,
            package: package.to_string(),
            message: err.to_string(),
        })?;

    let versions = parsed
        .packages
        .values()
        .flat_map(|versions| versions.iter())
        .map(|entry| entry.version.as_str())
        .filter(|version| !version.starts_with("dev-"));

    latest_stable(versions).ok_or_else(|| RegistryError::InvalidResponse {
        registry: REGISTRY_NAME,
        package: package.to_string(),
        message: "no stable tagged versions".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_latest_stable_tag() {
        let body = r#"{
            "packages": {
                "monolog/monolog": [
                    { "version": "4.0.0-beta" },
                    { "version": "3.5.0" },
                    { "version": "3.4.0" },
                    { "version": "dev-main" }
                ]
            }
        }"#;

        assert_eq!(
            parse_composer_response(body, "monolog/monolog").expect("version"),
            "3.5.0"
        );
    }

    #[test]
    fn keeps_the_v_prefix() {
        let body = r#"{ "packages": { "acme/pkg": [{ "version": "v1.2.3" }] } }"#;
        assert_eq!(
            parse_composer_response(body, "acme/pkg").expect("version"),
            "v1.2.3"
        );
    }

    #[test]
    fn errors_when_only_dev_versions_exist() {
        let body = r#"{ "packages": { "acme/pkg": [{ "version": "dev-main" }] } }"#;
        let error = parse_composer_response(body, "acme/pkg").expect_err("dev-only should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }

    #[test]
    fn rejects_unexpected_shapes() {
        let error = parse_composer_response("not json", "acme/pkg").expect_err("invalid json");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }
}
