//! Provider detection and templated summaries for the Phase 3 AI module.
//!
//! No outbound LLM calls are made yet: [`Summarizer::new`] inspects the
//! environment so the CLI can report which provider *would* be used, and every
//! at-risk/breaking dependency receives a templated one-line summary. Phase 5
//! replaces the template with a real changelog summary.

use std::env;

use tracing::info;

use crate::model::UpdateType;

/// Environment variable naming a custom LLM endpoint.
pub const CUSTOM_ENDPOINT_ENV: &str = "RUSTYDEPS_AI_ENDPOINT";

/// An LLM provider RustyDeps can be configured with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    /// Anthropic (`ANTHROPIC_API_KEY`).
    Anthropic,
    /// OpenAI (`OPENAI_API_KEY`).
    OpenAI,
    /// A custom endpoint (`RUSTYDEPS_AI_ENDPOINT`).
    Custom,
}

impl AiProvider {
    /// Short lowercase label used in logs.
    pub fn label(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAI => "openai",
            Self::Custom => "custom",
        }
    }
}

/// Which provider the environment configures, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiSummaryConfig {
    /// Whether `--ai` was passed.
    pub enabled: bool,
    /// The provider detected from the environment, if any.
    pub provider: Option<AiProvider>,
}

impl AiSummaryConfig {
    /// Detect provider configuration from the environment.
    ///
    /// Anthropic wins over OpenAI over a custom endpoint when several are set.
    pub fn from_env(enabled: bool) -> Self {
        let provider = if env::var_os("ANTHROPIC_API_KEY").is_some() {
            Some(AiProvider::Anthropic)
        } else if env::var_os("OPENAI_API_KEY").is_some() {
            Some(AiProvider::OpenAI)
        } else if env::var_os(CUSTOM_ENDPOINT_ENV).is_some() {
            Some(AiProvider::Custom)
        } else {
            None
        };

        Self { enabled, provider }
    }
}

/// Templated one-line summary for a classified update, if one is worthwhile.
pub fn fallback_summary(update_type: UpdateType) -> Option<&'static str> {
    match update_type {
        UpdateType::Breaking => {
            Some("Major version bump — potential breaking API changes; review before upgrading.")
        }
        UpdateType::AtRisk => {
            Some("Upgrade outside the declared range — verify compatibility before updating.")
        }
        UpdateType::Safe | UpdateType::UpToDate | UpdateType::Unknown => None,
    }
}

/// Produces the `ai_summary` value for a classified dependency.
#[derive(Debug, Clone, Copy)]
pub struct Summarizer {
    config: AiSummaryConfig,
}

impl Summarizer {
    /// Build a summarizer, detecting the provider from the environment and
    /// logging the outcome once.
    pub fn new(enabled: bool) -> Self {
        let config = AiSummaryConfig::from_env(enabled);
        if enabled {
            match config.provider {
                Some(provider) => info!(
                    provider = provider.label(),
                    "AI summaries enabled; live provider summaries arrive in Phase 5, using templated summaries for now"
                ),
                None => info!(
                    "AI summaries enabled but no provider configured; set ANTHROPIC_API_KEY, OPENAI_API_KEY, or RUSTYDEPS_AI_ENDPOINT (templated summaries are used until Phase 5)"
                ),
            }
        }
        Self { config }
    }

    /// Build a summarizer from an explicit configuration (used by tests).
    #[cfg(test)]
    pub fn from_config(config: AiSummaryConfig) -> Self {
        Self { config }
    }

    /// Whether `--ai` was requested.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// The detected provider, if any.
    pub fn provider(&self) -> Option<AiProvider> {
        self.config.provider
    }

    /// The summary for `update_type`, or `None` when none is worthwhile.
    pub fn summarize(&self, update_type: UpdateType) -> Option<String> {
        fallback_summary(update_type).map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_at_risk_and_breaking_get_summaries() {
        assert!(fallback_summary(UpdateType::Breaking).is_some());
        assert!(fallback_summary(UpdateType::AtRisk).is_some());
        assert!(fallback_summary(UpdateType::Safe).is_none());
        assert!(fallback_summary(UpdateType::UpToDate).is_none());
        assert!(fallback_summary(UpdateType::Unknown).is_none());
    }

    #[test]
    fn summarize_returns_a_templated_line() {
        let summarizer = Summarizer::from_config(AiSummaryConfig {
            enabled: false,
            provider: None,
        });

        let summary = summarizer
            .summarize(UpdateType::Breaking)
            .expect("breaking summary");
        assert!(summary.contains("breaking"));
        assert!(summarizer.summarize(UpdateType::Safe).is_none());
    }

    #[test]
    fn a_configured_provider_still_falls_back_until_phase_5() {
        let summarizer = Summarizer::from_config(AiSummaryConfig {
            enabled: true,
            provider: Some(AiProvider::Anthropic),
        });

        assert!(summarizer.is_enabled());
        assert_eq!(summarizer.provider(), Some(AiProvider::Anthropic));
        assert!(summarizer.summarize(UpdateType::AtRisk).is_some());
    }
}
