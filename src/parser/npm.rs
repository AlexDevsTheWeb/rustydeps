//! Parser for `package.json` manifests.
//!
//! Extracts the `dependencies` and `devDependencies` maps. Specs that do not
//! point at the public npm registry (`file:`, `link:`, `portal:`, `workspace:`,
//! `npm:` aliases, and `git`/`http` URLs) are skipped because they cannot be
//! resolved against `registry.npmjs.org`.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use tracing::debug;

use crate::model::Ecosystem;
use crate::parser::{PackageParser, ParsedDependency, ParserError};
use crate::scanner::{DiscoveredManifest, ManifestType};

/// Decodes `package.json` files.
#[derive(Debug, Default)]
pub struct NpmParser;

/// The subset of a `package.json` RustyDeps cares about.
#[derive(Debug, Default, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
}

impl NpmParser {
    /// Decode manifest `contents` that live at `path`.
    fn parse_contents(
        &self,
        path: &Path,
        contents: &str,
    ) -> Result<Vec<ParsedDependency>, ParserError> {
        let manifest: PackageJson =
            serde_json::from_str(contents).map_err(|err| ParserError::Invalid {
                path: path.to_path_buf(),
                ecosystem: Ecosystem::Npm,
                message: err.to_string(),
            })?;

        let mut dependencies = Vec::new();
        collect_dependencies(&mut dependencies, path, &manifest.dependencies, false);
        collect_dependencies(&mut dependencies, path, &manifest.dev_dependencies, true);

        Ok(dependencies)
    }
}

impl PackageParser for NpmParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Npm
    }

    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool {
        manifest.manifest_type == ManifestType::PackageJson
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

/// Append every registry-resolvable dependency in `specs` to `out`.
fn collect_dependencies(
    out: &mut Vec<ParsedDependency>,
    manifest_path: &Path,
    specs: &BTreeMap<String, String>,
    is_dev_dependency: bool,
) {
    for (name, spec) in specs {
        if !is_registry_spec(spec) {
            debug!(
                package = name,
                spec = spec,
                manifest = %manifest_path.display(),
                "skipping npm dependency that does not resolve through the registry"
            );
            continue;
        }

        out.push(ParsedDependency {
            name: name.clone(),
            current_version: spec.clone(),
            ecosystem: Ecosystem::Npm,
            manifest_path: manifest_path.to_path_buf(),
            is_dev_dependency,
        });
    }
}

/// Whether `spec` names a package on the public npm registry.
fn is_registry_spec(spec: &str) -> bool {
    let spec = spec.trim();
    !spec.is_empty()
        && !spec.starts_with("file:")
        && !spec.starts_with("link:")
        && !spec.starts_with("portal:")
        && !spec.starts_with("workspace:")
        && !spec.starts_with("npm:")
        && !spec.starts_with("git")
        && !spec.starts_with("http")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn parse(contents: &str) -> Vec<ParsedDependency> {
        NpmParser
            .parse_contents(Path::new("package.json"), contents)
            .expect("parse package.json")
    }

    fn find<'a>(deps: &'a [ParsedDependency], name: &str) -> Option<&'a ParsedDependency> {
        deps.iter().find(|dep| dep.name == name)
    }

    #[test]
    fn can_parse_only_package_json() {
        let npm = DiscoveredManifest {
            path: PathBuf::from("package.json"),
            manifest_type: ManifestType::PackageJson,
        };
        let cargo = DiscoveredManifest {
            path: PathBuf::from("Cargo.toml"),
            manifest_type: ManifestType::CargoToml,
        };

        assert!(NpmParser.can_parse(&npm));
        assert!(!NpmParser.can_parse(&cargo));
        assert_eq!(NpmParser.ecosystem(), Ecosystem::Npm);
    }

    #[test]
    fn parses_dependencies_and_dev_dependencies() {
        let deps = parse(
            r#"{
                "dependencies": { "express": "^4.18.0", "react": "18.2.0" },
                "devDependencies": { "typescript": "~5.4.0" }
            }"#,
        );

        assert_eq!(deps.len(), 3);
        assert_eq!(
            find(&deps, "express").expect("express").current_version,
            "^4.18.0"
        );
        assert!(!find(&deps, "react").expect("react").is_dev_dependency);
        assert!(
            find(&deps, "typescript")
                .expect("typescript")
                .is_dev_dependency
        );
    }

    #[test]
    fn skips_non_registry_specs() {
        let deps = parse(
            r#"{
                "dependencies": {
                    "local": "file:../local",
                    "linked": "link:../linked",
                    "remote": "git+https://example.com/remote.git",
                    "aliased": "npm:other@^1.0.0",
                    "workspace-pkg": "workspace:*",
                    "express": "^4.18.0"
                }
            }"#,
        );

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "express");
    }

    #[test]
    fn rejects_malformed_json() {
        let error = NpmParser
            .parse_contents(Path::new("package.json"), "{ not json")
            .expect_err("malformed json should fail");

        assert!(matches!(error, ParserError::Invalid { .. }));
    }

    #[test]
    fn tolerates_missing_dependency_maps() {
        let deps = parse(r#"{ "name": "example", "version": "1.0.0" }"#);

        assert!(deps.is_empty());
    }
}
