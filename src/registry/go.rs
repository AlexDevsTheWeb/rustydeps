//! Client for the Go module proxy (`https://proxy.golang.org/{module}/@v/list`).
//!
//! The proxy's `@v/list` endpoint returns the module's tagged versions, one per
//! line; the highest stable SemVer is chosen. Uppercase letters in a module path
//! are encoded as `!` + the lowercase letter, per the proxy protocol.

use crate::classifier::latest_stable;
use crate::model::Ecosystem;
use crate::registry::{RegistryClient, RegistryError};

/// Registry identifier used in error messages.
const REGISTRY_NAME: &str = "proxy.golang.org";

/// Base URL for the Go module proxy.
const PROXY_BASE: &str = "https://proxy.golang.org";

/// Go module proxy client.
#[derive(Debug, Clone)]
pub struct GoRegistryClient {
    http: reqwest::Client,
}

impl GoRegistryClient {
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

impl RegistryClient for GoRegistryClient {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Go
    }

    async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError> {
        let url = format!("{PROXY_BASE}/{}/@v/list", encode_module_path(name));
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

        parse_go_list_response(&body, name)
    }
}

/// Encode a module path for the Go module proxy: uppercase letters become
/// `!` followed by the lowercase letter.
fn encode_module_path(module: &str) -> String {
    let mut encoded = String::with_capacity(module.len());
    for character in module.chars() {
        if character.is_ascii_uppercase() {
            encoded.push('!');
            encoded.push(character.to_ascii_lowercase());
        } else {
            encoded.push(character);
        }
    }
    encoded
}

/// Wrap a transport error as a [`RegistryError::Http`].
fn http_error(package: &str, source: reqwest::Error) -> RegistryError {
    RegistryError::Http {
        registry: REGISTRY_NAME,
        package: package.to_string(),
        source,
    }
}

/// Extract the highest stable version from a `@v/list` response body.
pub fn parse_go_list_response(body: &str, package: &str) -> Result<String, RegistryError> {
    let versions = body.lines().map(str::trim).filter(|line| !line.is_empty());

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
    fn picks_the_highest_stable_version() {
        let body = "v1.9.0\nv1.9.1\nv1.10.0\nv2.0.0-rc.1\n";
        assert_eq!(
            parse_go_list_response(body, "github.com/gin-gonic/gin").expect("version"),
            "v1.10.0"
        );
    }

    #[test]
    fn keeps_the_v_prefix() {
        let body = "v1.2.3\n";
        assert_eq!(
            parse_go_list_response(body, "example.com/x").expect("version"),
            "v1.2.3"
        );
    }

    #[test]
    fn errors_when_only_prereleases_exist() {
        let error = parse_go_list_response("v2.0.0-rc.1\n", "example.com/x")
            .expect_err("no stable version should fail");
        assert!(matches!(error, RegistryError::InvalidResponse { .. }));
    }

    #[test]
    fn encodes_uppercase_module_paths() {
        assert_eq!(
            encode_module_path("github.com/Masterminds/semver"),
            "github.com/!masterminds/semver"
        );
        assert_eq!(
            encode_module_path("github.com/gin-gonic/gin"),
            "github.com/gin-gonic/gin"
        );
    }
}
