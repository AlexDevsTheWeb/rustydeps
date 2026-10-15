//! Parser for Composer manifests (`composer.json`).
//!
//! Reads the `require` and `require-dev` objects. PHP platform packages
//! (`php`, `ext-*`, `lib-*`, `composer-*`, `hhvm`) are skipped: they describe
//! the runtime, not a package resolvable through Packagist.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::model::Ecosystem;
use crate::parser::{PackageParser, ParsedDependency, ParserError};
use crate::scanner::{DiscoveredManifest, ManifestType};

/// Decodes `composer.json` files.
#[derive(Debug, Default)]
pub struct ComposerParser;

#[derive(Debug, Default, Deserialize)]
struct ComposerJson {
    #[serde(default)]
    require: BTreeMap<String, String>,
    #[serde(default, rename = "require-dev")]
    require_dev: BTreeMap<String, String>,
}

impl ComposerParser {
    fn parse_contents(
        &self,
        path: &Path,
        contents: &str,
    ) -> Result<Vec<ParsedDependency>, ParserError> {
        let manifest: ComposerJson =
            serde_json::from_str(contents).map_err(|err| ParserError::Invalid {
                path: path.to_path_buf(),
                ecosystem: Ecosystem::Composer,
                message: err.to_string(),
            })?;

        let mut dependencies = Vec::new();
        collect(&mut dependencies, path, &manifest.require, false);
        collect(&mut dependencies, path, &manifest.require_dev, true);
        Ok(dependencies)
    }
}

impl PackageParser for ComposerParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }

    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool {
        manifest.manifest_type == ManifestType::ComposerJson
    }

    fn parse(&self, manifest: &DiscoveredManifest) -> Result<Vec<ParsedDependency>, ParserError> {
        let contents =
            std::fs::read_to_string(&manifest.path).map_err(|source| ParserError::Io {
                path: manifest.path.clone(),
                source,
            })?;
        self.parse_contents(&manifest.path, &contents)
    }
}

fn collect(
    out: &mut Vec<ParsedDependency>,
    path: &Path,
    specs: &BTreeMap<String, String>,
    is_dev: bool,
) {
    for (name, constraint) in specs {
        if is_platform_package(name) {
            continue;
        }
        out.push(ParsedDependency {
            name: name.clone(),
            current_version: constraint.clone(),
            ecosystem: Ecosystem::Composer,
            manifest_path: path.to_path_buf(),
            is_dev_dependency: is_dev,
        });
    }
}

/// Whether `name` is a PHP platform/runtime pseudo-package.
fn is_platform_package(name: &str) -> bool {
    name == "php"
        || name == "hhvm"
        || name.starts_with("php-")
        || name.starts_with("ext-")
        || name.starts_with("lib-")
        || name.starts_with("composer-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(contents: &str) -> Vec<ParsedDependency> {
        ComposerParser
            .parse_contents(Path::new("composer.json"), contents)
            .expect("parse composer.json")
    }

    fn names(deps: &[ParsedDependency]) -> Vec<&str> {
        deps.iter().map(|dep| dep.name.as_str()).collect()
    }

    #[test]
    fn can_parse_only_composer_json() {
        assert!(ComposerParser.can_parse(&DiscoveredManifest {
            path: "composer.json".into(),
            manifest_type: ManifestType::ComposerJson,
        }));
        assert!(!ComposerParser.can_parse(&DiscoveredManifest {
            path: "package.json".into(),
            manifest_type: ManifestType::PackageJson,
        }));
        assert_eq!(ComposerParser.ecosystem(), Ecosystem::Composer);
    }

    #[test]
    fn parses_require_and_require_dev_skipping_platform_packages() {
        let deps = parse(
            r#"{
                "require": {
                    "php": ">=8.1",
                    "ext-json": "*",
                    "lib-curl": "*",
                    "composer-runtime-api": "^2.0",
                    "monolog/monolog": "^3.0",
                    "guzzlehttp/guzzle": "^7.8"
                },
                "require-dev": { "phpunit/phpunit": "^10.5" }
            }"#,
        );

        assert_eq!(
            names(&deps),
            vec!["guzzlehttp/guzzle", "monolog/monolog", "phpunit/phpunit"]
        );
        assert!(!deps[0].is_dev_dependency);
        assert!(deps[2].is_dev_dependency);
        assert!(deps.iter().all(|dep| dep.ecosystem == Ecosystem::Composer));
    }

    #[test]
    fn rejects_malformed_json() {
        let error = ComposerParser
            .parse_contents(Path::new("composer.json"), "{ not json")
            .expect_err("malformed json should fail");
        assert!(matches!(error, ParserError::Invalid { .. }));
    }
}
