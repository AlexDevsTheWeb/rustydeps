//! Per-library AI summarizer orchestration.
//!
//! The summarizer turns every outdated, registry-resolved dependency into an
//! independent request (or cache lookup), bounded by [`MAX_CONCURRENCY`].
//! Failures are logged and leave `ai_summary` as `None`; they never fail the
//! scan.

use std::path::PathBuf;

use futures::stream::{self, StreamExt};
use tracing::{debug, info, warn};

use super::cache::{now_unix_seconds, CachedSummary, SummaryCache};
use super::client::AiClient;
use super::prompt::{user_prompt, UpgradeContext, SYSTEM_PROMPT};
use crate::classifier::ClassifiedDependency;
use crate::config::{self, AiOverrides, AiProvider, AiSettings};
use crate::model::UpdateType;

/// Maximum number of provider requests in flight at once.
pub const MAX_CONCURRENCY: usize = 4;

/// Produces `ai_summary` values for classified dependencies.
#[derive(Clone)]
pub struct Summarizer {
    client: Option<AiClient>,
    cache: SummaryCache,
    enabled: bool,
    provider: Option<AiProvider>,
    model: Option<String>,
}

impl Summarizer {
    /// Resolve configuration from the files, environment, and CLI overrides.
    pub fn from_overrides(overrides: &AiOverrides) -> Self {
        match config::load(overrides) {
            Ok(loaded) => Self::from_settings(loaded.settings, loaded.source),
            Err(error) => {
                warn!(
                    %error,
                    "AI configuration could not be loaded; AI analysis is disabled for this scan"
                );
                Self::disabled()
            }
        }
    }

    /// Build from already-resolved settings.
    pub fn from_settings(settings: AiSettings, source: Option<PathBuf>) -> Self {
        if !settings.enabled {
            info!("AI analysis is disabled (enable it in the config or pass --ai)");
            return Self::disabled();
        }

        if settings.provider.requires_api_key() && settings.api_key.is_none() {
            warn!(
                provider = settings.provider.label(),
                "AI analysis is enabled but no API key is configured; skipping AI analysis"
            );
            return Self {
                client: None,
                cache: SummaryCache::new(settings.cache_enabled),
                enabled: false,
                provider: Some(settings.provider),
                model: Some(settings.model),
            };
        }

        match AiClient::new(settings.clone()) {
            Ok(client) => {
                if let Some(path) = &source {
                    info!(config = %path.display(), "loaded the AI configuration");
                }
                info!(
                    provider = settings.provider.label(),
                    model = %settings.model,
                    endpoint = %settings.endpoint,
                    cache = settings.cache_enabled,
                    "per-library AI analysis enabled"
                );
                Self {
                    client: Some(client),
                    cache: SummaryCache::new(settings.cache_enabled),
                    enabled: true,
                    provider: Some(settings.provider),
                    model: Some(settings.model),
                }
            }
            Err(error) => {
                warn!(
                    %error,
                    "could not build the AI client; AI analysis is disabled for this scan"
                );
                Self {
                    client: None,
                    cache: SummaryCache::new(settings.cache_enabled),
                    enabled: false,
                    provider: Some(settings.provider),
                    model: Some(settings.model),
                }
            }
        }
    }

    /// A summarizer that does nothing.
    pub fn disabled() -> Self {
        Self {
            client: None,
            cache: SummaryCache::new(false),
            enabled: false,
            provider: None,
            model: None,
        }
    }

    /// Whether per-library analysis will run.
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.client.is_some()
    }

    /// The selected provider, when one was configured.
    pub fn provider(&self) -> Option<AiProvider> {
        self.provider
    }

    /// Analyze every outdated, registry-resolved dependency in place.
    pub async fn summarize_all(&self, dependencies: &mut [ClassifiedDependency]) {
        if !self.is_enabled() {
            return;
        }

        let tasks = dependencies
            .iter_mut()
            .filter(|dependency| needs_analysis(dependency))
            .map(|dependency| self.analyze(dependency));

        stream::iter(tasks)
            .buffer_unordered(MAX_CONCURRENCY)
            .for_each(|_| async {})
            .await;
    }

    /// Analyze one package upgrade, preferring the local cache.
    async fn analyze(&self, dependency: &mut ClassifiedDependency) {
        let Some(target) = dependency.latest_version.clone() else {
            return;
        };
        let ecosystem = dependency.ecosystem;
        let package = dependency.name.as_str();

        if let Some(cached) = self.cache.load(ecosystem, package, &target) {
            debug!(package, target = %target, "using the cached AI analysis");
            dependency.ai_summary = Some(cached.summary);
            dependency.ai_cached = true;
            return;
        }

        let Some(client) = &self.client else {
            return;
        };

        let context = UpgradeContext::new(ecosystem, package, &dependency.current_version, &target);

        match client.complete(SYSTEM_PROMPT, &user_prompt(&context)).await {
            Ok(summary) if !summary.trim().is_empty() => {
                dependency.ai_summary = Some(summary.clone());
                dependency.ai_cached = false;

                let entry = CachedSummary {
                    ecosystem,
                    package: package.to_string(),
                    target_version: target.clone(),
                    model: self.model.clone().unwrap_or_default(),
                    summary,
                    created_at: now_unix_seconds(),
                };
                if let Err(error) = self.cache.store(&entry) {
                    warn!(package, %error, "could not write the AI summary cache");
                }
            }
            Ok(_) => warn!(
                package,
                target = %target,
                "AI provider returned an empty analysis; continuing without a summary"
            ),
            Err(error) => warn!(
                package,
                target = %target,
                %error,
                "AI analysis unavailable; continuing without a summary"
            ),
        }
    }
}

/// Only outdated packages with a resolved target are worth a request.
fn needs_analysis(dependency: &ClassifiedDependency) -> bool {
    dependency.latest_version.is_some()
        && matches!(
            dependency.update_type,
            UpdateType::Breaking | UpdateType::AtRisk | UpdateType::Safe
        )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::Ecosystem;

    fn dependency(update_type: UpdateType, latest: Option<&str>) -> ClassifiedDependency {
        ClassifiedDependency {
            name: "tokio".to_string(),
            ecosystem: Ecosystem::Cargo,
            current_version: "1.20.0".to_string(),
            latest_version: latest.map(str::to_string),
            update_type,
            manifest_path: PathBuf::from("Cargo.toml"),
            is_dev_dependency: false,
            ai_summary: None,
            ai_cached: false,
        }
    }

    fn cache_only_summarizer(root: PathBuf) -> Summarizer {
        Summarizer {
            client: None,
            cache: SummaryCache::with_root(true, root),
            enabled: true,
            provider: Some(AiProvider::Openai),
            model: Some("test-model".to_string()),
        }
    }

    #[test]
    fn only_outdated_resolved_packages_are_analyzed() {
        assert!(needs_analysis(&dependency(
            UpdateType::Breaking,
            Some("2.0.0")
        )));
        assert!(needs_analysis(&dependency(
            UpdateType::AtRisk,
            Some("1.30.0")
        )));
        assert!(needs_analysis(&dependency(
            UpdateType::Safe,
            Some("1.20.1")
        )));
        assert!(!needs_analysis(&dependency(
            UpdateType::UpToDate,
            Some("1.20.0")
        )));
        assert!(!needs_analysis(&dependency(UpdateType::Unknown, None)));
    }

    #[tokio::test]
    async fn a_disabled_summarizer_leaves_summaries_empty() {
        let mut dependency = dependency(UpdateType::Breaking, Some("2.0.0"));
        Summarizer::disabled()
            .summarize_all(std::slice::from_mut(&mut dependency))
            .await;

        assert!(dependency.ai_summary.is_none());
        assert!(!dependency.ai_cached);
    }

    #[tokio::test]
    async fn cached_analyses_are_applied_without_a_client() {
        let directory = tempfile::tempdir().expect("tempdir");
        let summarizer = cache_only_summarizer(directory.path().to_path_buf());
        summarizer
            .cache
            .store(&CachedSummary {
                ecosystem: Ecosystem::Cargo,
                package: "tokio".to_string(),
                target_version: "2.0.0".to_string(),
                model: "test-model".to_string(),
                summary: "### 🚨 Breaking Changes\nUse `tokio::runtime`.".to_string(),
                created_at: now_unix_seconds(),
            })
            .expect("store cache");

        let mut dependency = dependency(UpdateType::Breaking, Some("2.0.0"));
        summarizer.analyze(&mut dependency).await;

        assert_eq!(
            dependency.ai_summary.as_deref(),
            Some("### 🚨 Breaking Changes\nUse `tokio::runtime`.")
        );
        assert!(dependency.ai_cached);
    }
}
