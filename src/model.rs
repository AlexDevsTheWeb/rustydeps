//! Shared domain models used across the parsers, registry clients, classifier,
//! and CLI.
//!
//! Phase 2 introduced [`Ecosystem`]; Phase 3 adds [`UpdateType`], the outcome of
//! comparing a declared version against the latest published one.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A package ecosystem RustyDeps understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ecosystem {
    /// Rust crates published to <https://crates.io>.
    Cargo,
    /// Node packages published to <https://registry.npmjs.org>.
    Npm,
}

impl Ecosystem {
    /// Short lowercase label used in console and JSON output.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Npm => "npm",
        }
    }
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// How a dependency's declared version relates to the latest published version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateType {
    /// A patch or minor bump that stays inside the declared requirement.
    Safe,
    /// A bump outside the declared requirement that is not obviously breaking
    /// (for example a minor bump past a `~` requirement).
    AtRisk,
    /// A major version jump, or a requirement change that breaks the contract.
    Breaking,
    /// The declared version already matches the latest published version.
    UpToDate,
    /// The versions could not be compared (unparseable, downgrade, or missing
    /// registry data).
    Unknown,
}

impl UpdateType {
    /// Short lowercase label used in console and JSON output.
    pub fn label(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::AtRisk => "at-risk",
            Self::Breaking => "breaking",
            Self::UpToDate => "up-to-date",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for UpdateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_serializes_to_lowercase_labels() {
        assert_eq!(Ecosystem::Cargo.to_string(), "cargo");
        assert_eq!(Ecosystem::Npm.to_string(), "npm");
        assert_eq!(
            serde_json::to_string(&Ecosystem::Npm).expect("serialize"),
            "\"npm\""
        );
    }

    #[test]
    fn update_type_labels_and_serialization() {
        assert_eq!(UpdateType::AtRisk.to_string(), "at-risk");
        assert_eq!(UpdateType::UpToDate.to_string(), "up-to-date");
        assert_eq!(
            serde_json::to_string(&UpdateType::Breaking).expect("serialize"),
            "\"breaking\""
        );
        assert_eq!(
            serde_json::from_str::<UpdateType>("\"at_risk\"").expect("deserialize"),
            UpdateType::AtRisk
        );
    }
}
