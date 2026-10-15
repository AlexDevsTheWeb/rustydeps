//! Manifest parsing: turn a discovered manifest into the dependencies it
//! declares, independent of the ecosystem.
//!
//! Every supported ecosystem plugs in behind the [`PackageParser`] trait, so the
//! scanner never needs to know how a manifest is decoded and later ecosystems
//! (PyPI, Go, Composer) are added without touching it.

pub mod cargo;
pub mod npm;

use std::path::PathBuf;

use serde::Serialize;
use thiserror::Error;
use tracing::debug;

use crate::model::Ecosystem;
use crate::scanner::DiscoveredManifest;

/// A dependency as declared in a manifest, before any registry lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedDependency {
    /// Package name.
    pub name: String,
    /// Declared version constraint or exact version, verbatim from the manifest.
    pub current_version: String,
    /// The ecosystem the package belongs to.
    pub ecosystem: Ecosystem,
    /// Manifest the dependency was declared in.
    pub manifest_path: PathBuf,
    /// Whether the dependency is declared as development-only.
    pub is_dev_dependency: bool,
}

/// Errors raised while decoding a manifest.
#[derive(Debug, Error)]
pub enum ParserError {
    /// The manifest could not be read from disk.
    #[error("failed to read manifest {path}: {source}")]
    Io {
        /// Manifest path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The manifest was readable but not valid for its ecosystem.
    #[error("invalid {ecosystem} manifest {path}: {message}")]
    Invalid {
        /// Manifest path.
        path: PathBuf,
        /// Ecosystem the parser expected.
        ecosystem: Ecosystem,
        /// Human-readable explanation from the decoder.
        message: String,
    },
}

/// A decoder for one ecosystem's manifest format.
pub trait PackageParser: Send + Sync {
    /// The ecosystem this parser decodes.
    fn ecosystem(&self) -> Ecosystem;

    /// Whether this parser handles `manifest`.
    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool;

    /// Decode `manifest` into its declared dependencies.
    fn parse(&self, manifest: &DiscoveredManifest) -> Result<Vec<ParsedDependency>, ParserError>;
}

/// The parsers RustyDeps ships in Phase 2, in priority order.
pub fn default_parsers() -> Vec<Box<dyn PackageParser>> {
    vec![Box::new(cargo::CargoParser), Box::new(npm::NpmParser)]
}

/// Result of parsing a batch of manifests.
#[derive(Debug, Default)]
pub struct ParseOutcome {
    /// Dependencies parsed across every manifest a parser claimed.
    pub dependencies: Vec<ParsedDependency>,
    /// Manifests that were claimed by a parser but could not be decoded,
    /// paired with the error so the caller can log and skip them.
    pub failures: Vec<(PathBuf, ParserError)>,
}

/// Parse every manifest with the first parser that claims it.
///
/// Manifests no parser supports are skipped silently. A manifest that a parser
/// claims but cannot decode is recorded in [`ParseOutcome::failures`] and does
/// not abort the scan, so one malformed manifest never hides the rest.
pub fn parse_manifests(
    manifests: &[DiscoveredManifest],
    parsers: &[Box<dyn PackageParser>],
) -> ParseOutcome {
    let mut outcome = ParseOutcome::default();

    for manifest in manifests {
        let Some(parser) = parsers.iter().find(|parser| parser.can_parse(manifest)) else {
            continue;
        };

        debug!(
            ecosystem = %parser.ecosystem(),
            manifest = %manifest.path.display(),
            "parsing manifest"
        );

        match parser.parse(manifest) {
            Ok(dependencies) => outcome.dependencies.extend(dependencies),
            Err(error) => outcome.failures.push((manifest.path.clone(), error)),
        }
    }

    outcome
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::scanner::ManifestType;

    fn manifest(path: PathBuf, manifest_type: ManifestType) -> DiscoveredManifest {
        DiscoveredManifest {
            path,
            manifest_type,
        }
    }

    #[test]
    fn routes_each_manifest_to_its_parser() {
        let dir = TempDir::new().expect("temp dir");
        let cargo_path = dir.path().join("Cargo.toml");
        let npm_path = dir.path().join("package.json");
        let go_path = dir.path().join("go.mod");
        fs::write(&cargo_path, "[dependencies]\nserde = \"1.0\"\n").expect("write cargo");
        fs::write(&npm_path, r#"{"dependencies":{"express":"^4.0.0"}}"#).expect("write npm");
        fs::write(&go_path, "module example").expect("write go");

        let manifests = vec![
            manifest(cargo_path, ManifestType::CargoToml),
            manifest(npm_path, ManifestType::PackageJson),
            manifest(go_path, ManifestType::GoMod),
        ];

        let outcome = parse_manifests(&manifests, &default_parsers());

        assert!(outcome.failures.is_empty());
        assert_eq!(outcome.dependencies.len(), 2);
        let ecosystems: Vec<_> = outcome
            .dependencies
            .iter()
            .map(|dep| dep.ecosystem)
            .collect();
        assert!(ecosystems.contains(&Ecosystem::Cargo));
        assert!(ecosystems.contains(&Ecosystem::Npm));
    }

    #[test]
    fn records_failures_without_aborting() {
        let dir = TempDir::new().expect("temp dir");
        let broken = dir.path().join("package.json");
        let good = dir.path().join("other/package.json");
        fs::create_dir_all(good.parent().expect("parent")).expect("create dir");
        fs::write(&broken, "{ not json").expect("write broken");
        fs::write(&good, r#"{"dependencies":{"left-pad":"1.3.0"}}"#).expect("write good");

        let manifests = vec![
            manifest(broken, ManifestType::PackageJson),
            manifest(good, ManifestType::PackageJson),
        ];

        let outcome = parse_manifests(&manifests, &default_parsers());

        assert_eq!(outcome.failures.len(), 1);
        assert_eq!(outcome.dependencies.len(), 1);
        assert_eq!(outcome.dependencies[0].name, "left-pad");
    }
}
