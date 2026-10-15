//! Parser for `Cargo.toml` manifests.
//!
//! Extracts `[dependencies]`, `[dev-dependencies]`, and, where present,
//! `[workspace.dependencies]`. A dependency that inherits its version from the
//! workspace (`serde = { workspace = true }`) is resolved against the workspace
//! table. Dependencies declared with no version at all (path- or git-only) are
//! skipped: they are not resolvable through a public registry, so reporting a
//! "latest version" for them would be misleading.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use tracing::debug;

use crate::model::Ecosystem;
use crate::parser::{PackageParser, ParsedDependency, ParserError};
use crate::scanner::{DiscoveredManifest, ManifestType};

/// Decodes `Cargo.toml` files.
#[derive(Debug, Default)]
pub struct CargoParser;

/// The subset of a `Cargo.toml` RustyDeps cares about.
#[derive(Debug, Default, Deserialize)]
struct CargoManifest {
    #[serde(default)]
    dependencies: BTreeMap<String, DependencySpec>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: BTreeMap<String, DependencySpec>,
    #[serde(default)]
    workspace: Option<Workspace>,
}

/// The `[workspace]` section, when the manifest is a workspace root.
#[derive(Debug, Default, Deserialize)]
struct Workspace {
    #[serde(default)]
    dependencies: BTreeMap<String, DependencySpec>,
}

/// A dependency value: either a bare version string (`serde = "1.0"`) or an
/// inline table (`serde = { version = "1.0", features = [...] }`).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DependencySpec {
    /// `dep = "version"`.
    Simple(String),
    /// `dep = { ... }`.
    Detailed(DetailedDependency),
}

/// An inline-table dependency. Unknown keys (`features`, `optional`, `path`,
/// `git`, ...) are ignored.
#[derive(Debug, Deserialize)]
struct DetailedDependency {
    version: Option<String>,
    #[serde(default)]
    workspace: bool,
}

impl CargoParser {
    /// Decode manifest `contents` that live at `path`.
    fn parse_contents(
        &self,
        path: &Path,
        contents: &str,
    ) -> Result<Vec<ParsedDependency>, ParserError> {
        let manifest: CargoManifest =
            toml::from_str(contents).map_err(|err| ParserError::Invalid {
                path: path.to_path_buf(),
                ecosystem: Ecosystem::Cargo,
                message: err.to_string(),
            })?;

        let workspace_dependencies = manifest
            .workspace
            .as_ref()
            .map(|workspace| &workspace.dependencies);

        let mut dependencies = Vec::new();
        collect_dependencies(
            &mut dependencies,
            path,
            &manifest.dependencies,
            workspace_dependencies,
            false,
        );
        collect_dependencies(
            &mut dependencies,
            path,
            &manifest.dev_dependencies,
            workspace_dependencies,
            true,
        );
        if let Some(workspace_dependencies) = workspace_dependencies {
            collect_dependencies(
                &mut dependencies,
                path,
                workspace_dependencies,
                Some(workspace_dependencies),
                false,
            );
        }

        Ok(dependencies)
    }
}

impl PackageParser for CargoParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Cargo
    }

    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool {
        manifest.manifest_type == ManifestType::CargoToml
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

/// Append every resolvable dependency in `specs` to `out`.
fn collect_dependencies(
    out: &mut Vec<ParsedDependency>,
    manifest_path: &Path,
    specs: &BTreeMap<String, DependencySpec>,
    workspace_dependencies: Option<&BTreeMap<String, DependencySpec>>,
    is_dev_dependency: bool,
) {
    for (name, spec) in specs {
        let Some(current_version) = resolve_version(name, spec, workspace_dependencies) else {
            debug!(
                package = name,
                manifest = %manifest_path.display(),
                "skipping Cargo dependency with no version constraint"
            );
            continue;
        };

        out.push(ParsedDependency {
            name: name.clone(),
            current_version,
            ecosystem: Ecosystem::Cargo,
            manifest_path: manifest_path.to_path_buf(),
            is_dev_dependency,
        });
    }
}

/// Resolve the declared version for `spec`, following workspace inheritance.
fn resolve_version(
    name: &str,
    spec: &DependencySpec,
    workspace_dependencies: Option<&BTreeMap<String, DependencySpec>>,
) -> Option<String> {
    if let Some(version) = direct_version(spec) {
        return Some(version.to_string());
    }

    match spec {
        DependencySpec::Detailed(detail) if detail.workspace => workspace_dependencies
            .and_then(|workspace| workspace.get(name))
            .and_then(direct_version)
            .map(str::to_string),
        _ => None,
    }
}

/// The version declared directly on `spec`, if any (ignoring inheritance).
fn direct_version(spec: &DependencySpec) -> Option<&str> {
    let version = match spec {
        DependencySpec::Simple(version) => version.as_str(),
        DependencySpec::Detailed(detail) => detail.version.as_deref()?,
    };
    (!version.is_empty()).then_some(version)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn parse(contents: &str) -> Vec<ParsedDependency> {
        CargoParser
            .parse_contents(Path::new("Cargo.toml"), contents)
            .expect("parse Cargo.toml")
    }

    fn find<'a>(deps: &'a [ParsedDependency], name: &str) -> Option<&'a ParsedDependency> {
        deps.iter().find(|dep| dep.name == name)
    }

    #[test]
    fn can_parse_only_cargo_manifests() {
        let cargo = DiscoveredManifest {
            path: PathBuf::from("Cargo.toml"),
            manifest_type: ManifestType::CargoToml,
        };
        let npm = DiscoveredManifest {
            path: PathBuf::from("package.json"),
            manifest_type: ManifestType::PackageJson,
        };

        assert!(CargoParser.can_parse(&cargo));
        assert!(!CargoParser.can_parse(&npm));
        assert_eq!(CargoParser.ecosystem(), Ecosystem::Cargo);
    }

    #[test]
    fn parses_dependencies_and_dev_dependencies() {
        let deps = parse(
            r#"
            [dependencies]
            serde = "1.0"
            tokio = { version = "1.35", features = ["full"] }

            [dev-dependencies]
            tempfile = "3"
            "#,
        );

        assert_eq!(deps.len(), 3);
        assert_eq!(find(&deps, "serde").expect("serde").current_version, "1.0");
        assert_eq!(find(&deps, "tokio").expect("tokio").current_version, "1.35");
        assert!(!find(&deps, "serde").expect("serde").is_dev_dependency);
        assert!(find(&deps, "tempfile").expect("tempfile").is_dev_dependency);
    }

    #[test]
    fn resolves_workspace_inherited_versions() {
        let deps = parse(
            r#"
            [dependencies]
            serde = { workspace = true }
            local = { workspace = true }

            [workspace.dependencies]
            serde = "1.0.200"
            "#,
        );

        assert_eq!(
            find(&deps, "serde").expect("serde").current_version,
            "1.0.200"
        );
        // Not defined in the workspace table, so it cannot be resolved.
        assert!(find(&deps, "local").is_none());
    }

    #[test]
    fn includes_workspace_dependencies_declared_at_the_root() {
        let deps = parse(
            r#"
            [workspace.dependencies]
            anyhow = "1.0"
            "#,
        );

        let anyhow = find(&deps, "anyhow").expect("anyhow");
        assert_eq!(anyhow.current_version, "1.0");
        assert!(!anyhow.is_dev_dependency);
    }

    #[test]
    fn skips_path_and_git_only_dependencies() {
        let deps = parse(
            r#"
            [dependencies]
            local = { path = "../local" }
            remote = { git = "https://example.com/remote.git" }
            "#,
        );

        assert!(deps.is_empty());
    }

    #[test]
    fn rejects_malformed_toml() {
        let error = CargoParser
            .parse_contents(Path::new("Cargo.toml"), "[dependencies")
            .expect_err("malformed toml should fail");

        assert!(matches!(error, ParserError::Invalid { .. }));
    }
}
