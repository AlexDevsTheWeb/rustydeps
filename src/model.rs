//! Shared domain models used across the parsers, registry clients, and CLI.
//!
//! Phase 2 introduces [`Ecosystem`]; the unified `Dependency` and `UpdateType`
//! DTOs arrive in Phase 3 (see `doc/wiki/roadmap.md`).

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
}
