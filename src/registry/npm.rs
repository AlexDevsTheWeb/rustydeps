//! Client for the npm registry (`https://registry.npmjs.org/{name}`).
//!
//! The latest version is read from the packument's `dist-tags.latest`. Scoped
//! packages (`@scope/name`) have their slash percent-encoded for the request
//! path.

use serde::Deserialize;

use crate::model::Ecosystem;
use crate::registry::{RegistryClient, RegistryError};

/// Registry identifier used in error messages.
const REGISTRY_NAME: &str = "npm";

/// Base URL for the npm registry.
const REGISTRY_BASE: &str = "https://registry.npmjs.org";

/// npm registry client.
#[derive(Debug, Clone)]
pub struct NpmRegistryClient {
    http: reqwest::Client,
}

impl NpmRegistryClient {
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

impl RegistryClient for NpmRegistryClient {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Npm
    }

    async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError> {
        let url = package_url(name);
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

        parse_npm_response(&body, name)
    }
}

/// Percent-encode a package name for use in a registry URL path.
fn package_url(name: &str) -> String {
    format!("{REGISTRY_BASE}/{}", name.replace('/', "%2F"))
}

/// Wrap a transport error as a [`RegistryError::Http`].
fn http_error(package: &str, source: reqwest::Error) -> RegistryError {
    RegistryError::Http {
        registry: REGISTRY_NAME,
        package: package.to_string(),
        source,
    }
}

/// The npm response body, reduced to the fields RustyDeps reads.
#[derive(Debug, Deserialize)]
struct Packument {
    #[serde(default, rename = "dist-tags")]
    dist_tags: DistTags,
}

#[derive(Debug, Default, Deserialize)]
struct DistTags {
    latest: Option<String>,
}

/// Extract `dist-tags.latest` from an npm registry response body.
pub fn parse_npm_response(body: &str, package: &str) -> Result<String, RegistryError> {
    let parsed: Packument =
        serde_json::from_str(body).map_err(|err| RegistryError::InvalidResponse {
            registry: REGISTRY_NAME,
            package: package.to_string(),
            message: err.to_string(),
        })?;

    parsed
        .dist_tags
        .latest
        .ok_or_else(|| RegistryError::InvalidResponse {
            registry: REGISTRY_NAME,
            package: package.to_string(),
            message: "missing `dist-tags.latest`".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_latest_dist_tag() {
        let body = r#"{
            "name": "express",
            "dist-tags": { "latest": "4.19.2", "next": "5.0.0-beta.1" }
        }"#;

        assert_eq!(
            parse_npm_response(body, "express").expect("version"),
            "4.19.2"
        );
    }

    #[test]
    fn rejects_response_without_latest() {
        let error = parse_npm_response(r#"{ "name": "express" }"#, "express")
            .expect_err("missing dist-tags should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));

        let error = parse_npm_response(r#"{ "dist-tags": { "next": "1.0.0" } }"#, "express")
            .expect_err("missing latest should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }

    #[test]
    fn percent_encodes_scoped_package_names() {
        assert_eq!(
            package_url("@types/node"),
            "https://registry.npmjs.org/@types%2Fnode"
        );
        assert_eq!(package_url("express"), "https://registry.npmjs.org/express");
    }
}
