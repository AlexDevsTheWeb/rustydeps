//! Asynchronous clients for the public package registries.
//!
//! One client per registry implements [`RegistryClient`]. Lookups are driven by
//! [`fetch_latest_versions`], which bounds the number of concurrent in-flight
//! requests so a large scan stays polite to the registries.

pub mod crates_io;
pub mod npm;

use futures::stream::{self, StreamExt};
use thiserror::Error;

use crate::model::Ecosystem;

/// Default upper bound on concurrent registry requests.
pub const DEFAULT_CONCURRENCY: usize = 10;

/// Errors raised by a registry lookup.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// The HTTP request itself failed (DNS, TLS, timeout, non-2xx status, ...).
    #[error("request to {registry} for {package} failed: {source}")]
    Http {
        /// Registry identifier, e.g. `crates.io`.
        registry: &'static str,
        /// Package that was requested.
        package: String,
        /// Underlying transport error.
        #[source]
        source: reqwest::Error,
    },
    /// The registry has no package with that name.
    #[error("{registry} has no package named {package}")]
    NotFound {
        /// Registry identifier, e.g. `crates.io`.
        registry: &'static str,
        /// Package that was requested.
        package: String,
    },
    /// The registry responded, but not with the expected shape.
    #[error("{registry} returned an unexpected response for {package}: {message}")]
    InvalidResponse {
        /// Registry identifier, e.g. `crates.io`.
        registry: &'static str,
        /// Package that was requested.
        package: String,
        /// Human-readable explanation.
        message: String,
    },
}

/// A client that resolves the latest published version of a package.
///
/// The method is a native `async fn` in a trait; `#[allow(async_fn_in_trait)]`
/// silences the "cannot add `Send` bounds" advisory because concurrency here is
/// generic over the concrete client and never uses `dyn` dispatch.
#[allow(async_fn_in_trait)]
pub trait RegistryClient: Send + Sync {
    /// The ecosystem this client serves.
    fn ecosystem(&self) -> Ecosystem;

    /// Return the latest published version of `name`.
    async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError>;
}

/// Resolve every name in `names` against `client`, at most `concurrency`
/// requests in flight at once.
///
/// Results are returned as `(name, outcome)` pairs in completion order, so
/// callers should key by name rather than by position.
pub async fn fetch_latest_versions<C>(
    client: &C,
    names: &[String],
    concurrency: usize,
) -> Vec<(String, Result<String, RegistryError>)>
where
    C: RegistryClient,
{
    let limit = concurrency.max(1);

    stream::iter(names.iter().cloned())
        .map(|name| async move {
            let outcome = client.fetch_latest_version(&name).await;
            (name, outcome)
        })
        .buffer_unordered(limit)
        .collect()
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;

    /// Returns a fixed version for `known` and `NotFound` for anything else.
    struct MockClient;

    impl RegistryClient for MockClient {
        fn ecosystem(&self) -> Ecosystem {
            Ecosystem::Cargo
        }

        async fn fetch_latest_version(&self, name: &str) -> Result<String, RegistryError> {
            if name == "known" {
                Ok("1.2.3".to_string())
            } else {
                Err(RegistryError::NotFound {
                    registry: "mock",
                    package: name.to_string(),
                })
            }
        }
    }

    /// Sleeps while recording the peak number of concurrent calls.
    struct TrackingClient {
        active: AtomicUsize,
        max_active: AtomicUsize,
        delay: Duration,
    }

    impl RegistryClient for TrackingClient {
        fn ecosystem(&self) -> Ecosystem {
            Ecosystem::Cargo
        }

        async fn fetch_latest_version(&self, _name: &str) -> Result<String, RegistryError> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_active.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(self.delay).await;
            self.active.fetch_sub(1, Ordering::SeqCst);
            Ok("1.0.0".to_string())
        }
    }

    #[tokio::test]
    async fn returns_each_requested_name_once() {
        let names = vec!["known".to_string(), "missing".to_string()];

        let results = fetch_latest_versions(&MockClient, &names, DEFAULT_CONCURRENCY).await;

        assert_eq!(results.len(), 2);
        let known = results
            .iter()
            .find(|(name, _)| name == "known")
            .expect("known result");
        assert_eq!(known.1.as_ref().expect("version"), "1.2.3");

        let missing = results
            .iter()
            .find(|(name, _)| name == "missing")
            .expect("missing result");
        assert!(matches!(missing.1, Err(RegistryError::NotFound { .. })));
    }

    #[tokio::test]
    async fn bounds_concurrent_requests() {
        let client = TrackingClient {
            active: AtomicUsize::new(0),
            max_active: AtomicUsize::new(0),
            delay: Duration::from_millis(20),
        };
        let names: Vec<String> = (0..25).map(|index| format!("pkg-{index}")).collect();

        let results = fetch_latest_versions(&client, &names, 5).await;

        assert_eq!(results.len(), 25);
        assert!(results.iter().all(|(_, outcome)| outcome.is_ok()));
        let peak = client.max_active.load(Ordering::SeqCst);
        assert!(
            peak <= 5,
            "expected at most 5 concurrent requests, observed peak {peak}"
        );
    }
}
