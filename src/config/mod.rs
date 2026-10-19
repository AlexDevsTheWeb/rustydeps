//! Layered configuration for the per-library AI summarizer.
//!
//! Configuration is resolved with the following precedence, most specific
//! first:
//!
//! 1. CLI flags (`--no-ai`, `--ai-model`).
//! 2. `RUSTYDEPS_AI_*` environment variables.
//! 3. `~/.config/rustydeps/config.json` (the user configuration).
//! 4. A project-local `.rustydeps.json` in the current directory, used as a
//!    fallback when the user configuration does not exist.
//! 5. Built-in defaults.
//!
//! A malformed file or environment value is reported to the caller, which
//! disables AI summaries for the run instead of failing the scan.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Directory under the platform config root that holds the user config.
pub const CONFIG_DIR: &str = "rustydeps";
/// Name of the user configuration file.
pub const CONFIG_FILE: &str = "config.json";
/// Name of the optional project-local configuration file.
pub const LOCAL_CONFIG_FILE: &str = ".rustydeps.json";
/// Directory under the platform cache root that holds generated summaries.
pub const CACHE_DIR: &str = "changelogs";

/// Default cap on generated tokens per summary.
pub const DEFAULT_MAX_TOKENS: u32 = 400;
/// Upper bound accepted for `ai.max_tokens`.
pub const MAX_MAX_TOKENS: u32 = 8192;

/// An OpenAI-compatible LLM provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    /// OpenAI's hosted chat-completions API.
    Openai,
    /// Gemini's OpenAI-compatible endpoint.
    Gemini,
    /// A local Ollama server.
    Ollama,
}

impl AiProvider {
    /// Short lowercase label used in logs and configuration.
    pub fn label(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
        }
    }

    /// Model used when neither the file nor the environment names one.
    pub fn default_model(self) -> &'static str {
        match self {
            Self::Openai => "gpt-4o-mini",
            Self::Gemini => "gemini-2.5-flash",
            Self::Ollama => "llama3.1",
        }
    }

    /// Base URL used when neither the file nor the environment names one.
    pub fn default_endpoint(self) -> &'static str {
        match self {
            Self::Openai => "https://api.openai.com/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
            Self::Ollama => "http://localhost:11434/v1",
        }
    }

    /// Whether the provider rejects unauthenticated requests.
    pub fn requires_api_key(self) -> bool {
        !matches!(self, Self::Ollama)
    }

    /// Parse a provider label, accepting a few common spellings.
    pub fn from_label(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" | "open-ai" | "open_ai" => Some(Self::Openai),
            "gemini" | "google" | "google-gemini" => Some(Self::Gemini),
            "ollama" => Some(Self::Ollama),
            _ => None,
        }
    }
}

/// Configuration file schema (`~/.config/rustydeps/config.json` or
/// `.rustydeps.json`). Unknown keys are ignored for forward compatibility.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// AI summarizer settings.
    pub ai: AiConfig,
}

/// AI summarizer settings as written in the configuration file.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Whether AI summaries are enabled by default.
    pub enabled: bool,
    /// The provider to call.
    pub provider: AiProvider,
    /// API key for hosted providers (Ollama does not require one).
    pub api_key: Option<String>,
    /// Model name; provider-specific when omitted.
    pub model: Option<String>,
    /// Base URL of an OpenAI-compatible API; provider-specific when omitted.
    pub endpoint: Option<String>,
    /// Maximum tokens to generate per package analysis.
    pub max_tokens: u32,
    /// Whether generated analyses are read from and written to the local cache.
    pub cache_enabled: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AiProvider::Openai,
            api_key: None,
            model: None,
            endpoint: None,
            max_tokens: DEFAULT_MAX_TOKENS,
            cache_enabled: true,
        }
    }
}

/// CLI overrides for the AI summarizer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AiOverrides {
    /// `--ai`: force AI summaries on.
    pub enable: bool,
    /// `--no-ai`: force AI summaries off; wins over [`Self::enable`].
    pub disable: bool,
    /// `--ai-model`: override the configured model.
    pub model: Option<String>,
}

/// Fully resolved settings for one run.
#[derive(Clone, PartialEq, Eq)]
pub struct AiSettings {
    /// Whether AI analysis should run.
    pub enabled: bool,
    /// The selected provider.
    pub provider: AiProvider,
    /// API key, when the provider needs one.
    pub api_key: Option<String>,
    /// Resolved model name.
    pub model: String,
    /// Resolved OpenAI-compatible base URL.
    pub endpoint: String,
    /// Maximum tokens to generate per package analysis.
    pub max_tokens: u32,
    /// Whether the local summary cache is enabled.
    pub cache_enabled: bool,
}

/// Resolved configuration plus the file it came from, when one was read.
#[derive(Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    /// The effective settings.
    pub settings: AiSettings,
    /// The configuration file that was loaded, if any.
    pub source: Option<PathBuf>,
}

/// Errors raised while reading the layered configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The configuration file could not be read.
    #[error("could not read {path}: {source}")]
    Read {
        /// File that failed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The configuration file was not valid JSON or did not match the schema.
    #[error("invalid configuration in {path}: {source}")]
    Parse {
        /// File that failed.
        path: PathBuf,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// An environment override was not valid.
    #[error("invalid {name}={value:?}: {reason}")]
    InvalidEnv {
        /// Variable name.
        name: &'static str,
        /// Rejected value.
        value: String,
        /// What was expected.
        reason: &'static str,
    },
}

/// Load and resolve the effective AI configuration.
pub fn load(overrides: &AiOverrides) -> Result<LoadedConfig, ConfigError> {
    let (mut config, source) = load_file()?;
    apply_env(&mut config.ai)?;
    let settings = resolve(config.ai, overrides);
    Ok(LoadedConfig { settings, source })
}

/// Read the user configuration, falling back to the project-local file.
fn load_file() -> Result<(Config, Option<PathBuf>), ConfigError> {
    if let Some(path) = global_config_path() {
        if path.is_file() {
            return read_config(&path).map(|config| (config, Some(path)));
        }
    }

    if let Some(path) = local_config_path() {
        if path.is_file() {
            return read_config(&path).map(|config| (config, Some(path)));
        }
    }

    Ok((Config::default(), None))
}

/// Decode one configuration file.
fn read_config(path: &Path) -> Result<Config, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// Apply `RUSTYDEPS_AI_*` overrides to the file configuration.
fn apply_env(ai: &mut AiConfig) -> Result<(), ConfigError> {
    if let Some(value) = non_empty_env("RUSTYDEPS_AI_ENABLED") {
        ai.enabled = parse_bool(&value).ok_or(ConfigError::InvalidEnv {
            name: "RUSTYDEPS_AI_ENABLED",
            value,
            reason: "expected true/false, 1/0, yes/no, or on/off",
        })?;
    }

    if let Some(value) = non_empty_env("RUSTYDEPS_AI_PROVIDER") {
        ai.provider = AiProvider::from_label(&value).ok_or(ConfigError::InvalidEnv {
            name: "RUSTYDEPS_AI_PROVIDER",
            value,
            reason: "expected openai, gemini, or ollama",
        })?;
    }

    if let Some(value) = non_empty_env("RUSTYDEPS_AI_API_KEY") {
        ai.api_key = Some(value);
    }
    if let Some(value) = non_empty_env("RUSTYDEPS_AI_MODEL") {
        ai.model = Some(value);
    }
    if let Some(value) = non_empty_env("RUSTYDEPS_AI_ENDPOINT") {
        ai.endpoint = Some(value.trim_end_matches('/').to_string());
    }

    // Fall back to the provider's conventional environment variable so existing
    // OpenAI/Gemini setups keep working without duplicating the key.
    if ai.api_key.is_none() {
        ai.api_key = match ai.provider {
            AiProvider::Openai => non_empty_env("OPENAI_API_KEY"),
            AiProvider::Gemini => {
                non_empty_env("GEMINI_API_KEY").or_else(|| non_empty_env("GOOGLE_API_KEY"))
            }
            AiProvider::Ollama => None,
        };
    }

    Ok(())
}

/// Merge file/env configuration with CLI overrides and provider defaults.
fn resolve(ai: AiConfig, overrides: &AiOverrides) -> AiSettings {
    let enabled = if overrides.disable {
        false
    } else if overrides.enable {
        true
    } else {
        ai.enabled
    };

    let provider = ai.provider;
    let model = overrides
        .model
        .clone()
        .filter(|model| !model.trim().is_empty())
        .or_else(|| ai.model.filter(|model| !model.trim().is_empty()))
        .unwrap_or_else(|| provider.default_model().to_string());

    let endpoint = ai
        .endpoint
        .filter(|endpoint| !endpoint.trim().is_empty())
        .map(|endpoint| endpoint.trim_end_matches('/').to_string())
        .unwrap_or_else(|| provider.default_endpoint().to_string());

    AiSettings {
        enabled,
        provider,
        api_key: ai.api_key.filter(|key| !key.trim().is_empty()),
        model,
        endpoint,
        max_tokens: ai.max_tokens.clamp(1, MAX_MAX_TOKENS),
        cache_enabled: ai.cache_enabled,
    }
}

/// Parse a human-friendly boolean environment value.
pub fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Read a non-empty environment variable.
fn non_empty_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Path of the optional project-local configuration file.
pub fn local_config_path() -> Option<PathBuf> {
    env::current_dir()
        .ok()
        .map(|directory| directory.join(LOCAL_CONFIG_FILE))
}

/// Path of the user configuration file.
pub fn global_config_path() -> Option<PathBuf> {
    config_root().map(|root| root.join(CONFIG_DIR).join(CONFIG_FILE))
}

/// Directory holding the per-package summary cache.
pub fn cache_dir() -> Option<PathBuf> {
    cache_root().map(|root| root.join(CONFIG_DIR).join(CACHE_DIR))
}

/// Platform configuration root.
#[cfg(not(windows))]
fn config_root() -> Option<PathBuf> {
    non_empty_path_env("XDG_CONFIG_HOME").or_else(|| home_dir().map(|home| home.join(".config")))
}

/// Platform configuration root.
#[cfg(windows)]
fn config_root() -> Option<PathBuf> {
    non_empty_path_env("APPDATA").or_else(|| home_dir().map(|home| home.join(".config")))
}

/// Platform cache root.
#[cfg(not(windows))]
fn cache_root() -> Option<PathBuf> {
    non_empty_path_env("XDG_CACHE_HOME").or_else(|| home_dir().map(|home| home.join(".cache")))
}

/// Platform cache root.
#[cfg(windows)]
fn cache_root() -> Option<PathBuf> {
    non_empty_path_env("LOCALAPPDATA").or_else(|| home_dir().map(|home| home.join(".cache")))
}

/// The current user's home directory, if it can be determined.
fn home_dir() -> Option<PathBuf> {
    non_empty_path_env("HOME").or_else(|| non_empty_path_env("USERPROFILE"))
}

/// Read a non-empty environment variable as a path.
fn non_empty_path_env(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_disabled_with_provider_defaults() {
        let settings = resolve(AiConfig::default(), &AiOverrides::default());

        assert!(!settings.enabled);
        assert_eq!(settings.provider, AiProvider::Openai);
        assert_eq!(settings.model, "gpt-4o-mini");
        assert_eq!(settings.endpoint, "https://api.openai.com/v1");
        assert_eq!(settings.max_tokens, DEFAULT_MAX_TOKENS);
        assert!(settings.cache_enabled);
        assert_eq!(settings.api_key, None);
    }

    #[test]
    fn file_values_are_honored() {
        let config = AiConfig {
            enabled: true,
            provider: AiProvider::Ollama,
            api_key: None,
            model: Some("qwen2.5-coder".to_string()),
            endpoint: Some("http://localhost:9999/v1/".to_string()),
            max_tokens: 1200,
            cache_enabled: false,
        };

        let settings = resolve(config, &AiOverrides::default());

        assert!(settings.enabled);
        assert_eq!(settings.provider, AiProvider::Ollama);
        assert_eq!(settings.model, "qwen2.5-coder");
        assert_eq!(settings.endpoint, "http://localhost:9999/v1");
        assert_eq!(settings.max_tokens, 1200);
        assert!(!settings.cache_enabled);
    }

    #[test]
    fn cli_model_and_enable_override_the_file() {
        let config = AiConfig {
            enabled: false,
            provider: AiProvider::Openai,
            api_key: Some("from-file".to_string()),
            model: Some("file-model".to_string()),
            endpoint: None,
            max_tokens: 400,
            cache_enabled: true,
        };
        let overrides = AiOverrides {
            enable: true,
            disable: false,
            model: Some("cli-model".to_string()),
        };

        let settings = resolve(config, &overrides);

        assert!(settings.enabled);
        assert_eq!(settings.model, "cli-model");
        assert_eq!(settings.api_key.as_deref(), Some("from-file"));
    }

    #[test]
    fn no_ai_beats_every_enable_path() {
        let config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        let overrides = AiOverrides {
            enable: true,
            disable: true,
            model: None,
        };

        assert!(!resolve(config, &overrides).enabled);
    }

    #[test]
    fn token_cap_is_clamped() {
        let config = AiConfig {
            max_tokens: 0,
            ..AiConfig::default()
        };
        assert_eq!(resolve(config, &AiOverrides::default()).max_tokens, 1);

        let config = AiConfig {
            max_tokens: u32::MAX,
            ..AiConfig::default()
        };
        assert_eq!(
            resolve(config, &AiOverrides::default()).max_tokens,
            MAX_MAX_TOKENS
        );
    }

    #[test]
    fn provider_labels_round_trip() {
        for provider in [AiProvider::Openai, AiProvider::Gemini, AiProvider::Ollama] {
            assert_eq!(AiProvider::from_label(provider.label()), Some(provider));
        }
        assert_eq!(AiProvider::from_label("Google"), Some(AiProvider::Gemini));
        assert_eq!(AiProvider::from_label("anthropic"), None);
    }

    #[test]
    fn parses_boolean_environment_values() {
        for value in ["true", "TRUE", "1", "yes", "on"] {
            assert_eq!(parse_bool(value), Some(true), "{value}");
        }
        for value in ["false", "0", "no", "off"] {
            assert_eq!(parse_bool(value), Some(false), "{value}");
        }
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn reads_a_config_file_and_reports_invalid_json() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("config.json");
        fs::write(
            &path,
            r#"{ "ai": { "enabled": true, "provider": "ollama", "model": "llama3.2" } }"#,
        )
        .expect("write config");

        let config = read_config(&path).expect("parse config");
        assert!(config.ai.enabled);
        assert_eq!(config.ai.provider, AiProvider::Ollama);
        assert_eq!(config.ai.model.as_deref(), Some("llama3.2"));
        // Omitted fields fall back to the schema defaults.
        assert_eq!(config.ai.max_tokens, DEFAULT_MAX_TOKENS);
        assert!(config.ai.cache_enabled);

        fs::write(&path, "{ not json").expect("write invalid config");
        assert!(matches!(read_config(&path), Err(ConfigError::Parse { .. })));
    }

    #[test]
    fn config_and_cache_paths_are_provider_independent() {
        let config = global_config_path().expect("config path");
        assert!(config.ends_with(Path::new(CONFIG_DIR).join(CONFIG_FILE)));

        let cache = cache_dir().expect("cache path");
        assert!(cache.ends_with(Path::new(CONFIG_DIR).join(CACHE_DIR)));
    }
}
