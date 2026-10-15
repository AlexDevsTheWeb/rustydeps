//! Parser for Python manifests: `pyproject.toml` (PEP 621 and Poetry) and
//! `requirements.txt`.
//!
//! PEP 508 strings (`requests >= 2.28.0`, `flask[async]==3.0.0`,
//! `pydantic~=2.5 ; python_version >= "3.9"`) are reduced to a name plus a
//! semver-ish specifier. Poetry's `[tool.poetry.dependencies]` and
//! `[tool.poetry.dev-dependencies]` tables are read too; the `python`
//! interpreter entry and path/git-only dependencies are skipped.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use tracing::debug;

use crate::model::Ecosystem;
use crate::parser::{PackageParser, ParsedDependency, ParserError};
use crate::scanner::{DiscoveredManifest, ManifestType};

/// Decodes `pyproject.toml` and `requirements.txt`.
#[derive(Debug, Default)]
pub struct PypiParser;

#[derive(Debug, Default, Deserialize)]
struct PyProject {
    #[serde(default)]
    project: Option<Project>,
    #[serde(default)]
    tool: Option<Tool>,
}

#[derive(Debug, Default, Deserialize)]
struct Project {
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Tool {
    #[serde(default)]
    poetry: Option<Poetry>,
}

#[derive(Debug, Default, Deserialize)]
struct Poetry {
    #[serde(default)]
    dependencies: BTreeMap<String, PoetrySpec>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: BTreeMap<String, PoetrySpec>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PoetrySpec {
    Simple(String),
    Detailed(PoetryDetailed),
}

#[derive(Debug, Deserialize)]
struct PoetryDetailed {
    version: Option<String>,
}

impl PypiParser {
    fn parse_pyproject(
        &self,
        path: &Path,
        contents: &str,
    ) -> Result<Vec<ParsedDependency>, ParserError> {
        let manifest: PyProject = toml::from_str(contents).map_err(|err| ParserError::Invalid {
            path: path.to_path_buf(),
            ecosystem: Ecosystem::Pypi,
            message: err.to_string(),
        })?;

        let mut dependencies = Vec::new();

        if let Some(project) = &manifest.project {
            for spec in &project.dependencies {
                if let Some((name, version)) = parse_pep508(spec) {
                    dependencies.push(new_dependency(path, name, version, false));
                }
            }
        }

        if let Some(poetry) = manifest.tool.as_ref().and_then(|tool| tool.poetry.as_ref()) {
            collect_poetry(&mut dependencies, path, &poetry.dependencies, false);
            collect_poetry(&mut dependencies, path, &poetry.dev_dependencies, true);
        }

        Ok(dependencies)
    }

    fn parse_requirements(
        &self,
        path: &Path,
        contents: &str,
    ) -> Result<Vec<ParsedDependency>, ParserError> {
        let mut dependencies = Vec::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
                continue;
            }
            // Drop a trailing inline comment.
            let line = line.split('#').next().unwrap_or(line).trim();
            if line.is_empty() {
                continue;
            }
            if let Some((name, version)) = parse_pep508(line) {
                dependencies.push(new_dependency(path, name, version, false));
            }
        }

        Ok(dependencies)
    }
}

impl PackageParser for PypiParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Pypi
    }

    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool {
        matches!(
            manifest.manifest_type,
            ManifestType::PyprojectToml | ManifestType::RequirementsTxt
        )
    }

    fn parse(&self, manifest: &DiscoveredManifest) -> Result<Vec<ParsedDependency>, ParserError> {
        let contents =
            std::fs::read_to_string(&manifest.path).map_err(|source| ParserError::Io {
                path: manifest.path.clone(),
                source,
            })?;

        match manifest.manifest_type {
            ManifestType::RequirementsTxt => self.parse_requirements(&manifest.path, &contents),
            _ => self.parse_pyproject(&manifest.path, &contents),
        }
    }
}

fn new_dependency(path: &Path, name: String, version: String, is_dev: bool) -> ParsedDependency {
    ParsedDependency {
        name,
        current_version: version,
        ecosystem: Ecosystem::Pypi,
        manifest_path: path.to_path_buf(),
        is_dev_dependency: is_dev,
    }
}

fn collect_poetry(
    out: &mut Vec<ParsedDependency>,
    path: &Path,
    specs: &BTreeMap<String, PoetrySpec>,
    is_dev: bool,
) {
    for (name, spec) in specs {
        // `python` pins the interpreter, not a package.
        if name == "python" {
            continue;
        }
        let version = match spec {
            PoetrySpec::Simple(version) => version.clone(),
            PoetrySpec::Detailed(detail) => match &detail.version {
                Some(version) => version.clone(),
                None => {
                    debug!(
                        package = name,
                        manifest = %path.display(),
                        "skipping Poetry dependency without a version (path/git)"
                    );
                    continue;
                }
            },
        };
        out.push(new_dependency(path, name.clone(), version, is_dev));
    }
}

/// Reduce a PEP 508 requirement to `(name, specifier)`.
///
/// The specifier falls back to `*` when the requirement has no version pin, so
/// the dependency is still surfaced (it classifies as `Unknown`).
fn parse_pep508(raw: &str) -> Option<(String, String)> {
    let spec = strip_extras(raw.split(';').next().unwrap_or(raw).trim());
    if spec.is_empty() {
        return None;
    }

    let (name, rest) = match spec.find(['<', '>', '=', '!', '~', ' ', '\t']) {
        Some(index) => (&spec[..index], spec[index..].trim()),
        None => (spec.as_str(), ""),
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let version = normalize_specifier(rest).unwrap_or_else(|| "*".to_string());
    Some((name.to_string(), version))
}

/// Remove a PEP 508 extras group: `flask[async]` -> `flask`.
fn strip_extras(spec: &str) -> String {
    match (spec.find('['), spec.find(']')) {
        (Some(open), Some(close)) if open < close => {
            format!("{}{}", &spec[..open], &spec[close + 1..])
        }
        _ => spec.to_string(),
    }
}

/// Normalize a specifier to a semver-ish form (`>= 2.28.0` -> `>=2.28.0`,
/// `~=2.5` -> `~2.5`, `==3.0` -> `=3.0`).
fn normalize_specifier(rest: &str) -> Option<String> {
    let clause = rest.split(',').next().unwrap_or("").trim();
    if clause.is_empty() {
        return None;
    }
    let clause: String = clause.chars().filter(|c| !c.is_whitespace()).collect();
    let clause = clause
        .replace("~=", "~")
        .replace("===", "=")
        .replace("==", "=");
    (!clause.is_empty()).then_some(clause)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn manifest(path: &str, manifest_type: ManifestType) -> DiscoveredManifest {
        DiscoveredManifest {
            path: PathBuf::from(path),
            manifest_type,
        }
    }

    fn names(deps: &[ParsedDependency]) -> Vec<&str> {
        deps.iter().map(|dep| dep.name.as_str()).collect()
    }

    fn find<'a>(deps: &'a [ParsedDependency], name: &str) -> &'a ParsedDependency {
        deps.iter()
            .find(|dep| dep.name == name)
            .expect("dependency")
    }

    #[test]
    fn can_parse_python_manifests() {
        assert!(PypiParser.can_parse(&manifest("pyproject.toml", ManifestType::PyprojectToml)));
        assert!(PypiParser.can_parse(&manifest("requirements.txt", ManifestType::RequirementsTxt)));
        assert!(!PypiParser.can_parse(&manifest("Cargo.toml", ManifestType::CargoToml)));
        assert_eq!(PypiParser.ecosystem(), Ecosystem::Pypi);
    }

    #[test]
    fn parses_pep621_dependencies() {
        let deps = PypiParser
            .parse_pyproject(
                Path::new("pyproject.toml"),
                r#"
                [project]
                name = "svc"
                dependencies = [
                    "requests >= 2.28.0",
                    "flask[async]==3.0.0",
                    "pydantic~=2.5 ; python_version >= \"3.9\"",
                ]
                "#,
            )
            .expect("parse");

        assert_eq!(names(&deps), vec!["requests", "flask", "pydantic"]);
        assert_eq!(find(&deps, "requests").current_version, ">=2.28.0");
        assert_eq!(find(&deps, "flask").current_version, "=3.0.0");
        assert_eq!(find(&deps, "pydantic").current_version, "~2.5");
        assert!(!find(&deps, "requests").is_dev_dependency);
    }

    #[test]
    fn parses_poetry_dependencies_and_skips_python() {
        let deps = PypiParser
            .parse_pyproject(
                Path::new("pyproject.toml"),
                r#"
                [tool.poetry.dependencies]
                python = "^3.11"
                requests = "^2.28"
                local = { path = "../local" }

                [tool.poetry.dev-dependencies]
                pytest = "^8.0"
                "#,
            )
            .expect("parse");

        assert_eq!(names(&deps), vec!["requests", "pytest"]);
        assert_eq!(find(&deps, "requests").current_version, "^2.28");
        assert!(find(&deps, "pytest").is_dev_dependency);
    }

    #[test]
    fn parses_requirements_txt() {
        let deps = PypiParser
            .parse_requirements(
                Path::new("requirements.txt"),
                "# comment\nrequests==2.31.0\nflask>=3.0  # inline\n-r other.txt\n\nnumpy\n",
            )
            .expect("parse");

        assert_eq!(names(&deps), vec!["requests", "flask", "numpy"]);
        assert_eq!(find(&deps, "requests").current_version, "=2.31.0");
        assert_eq!(find(&deps, "flask").current_version, ">=3.0");
        assert_eq!(find(&deps, "numpy").current_version, "*");
    }

    #[test]
    fn rejects_malformed_pyproject() {
        let error = PypiParser
            .parse_pyproject(Path::new("pyproject.toml"), "[project")
            .expect_err("malformed toml should fail");
        assert!(matches!(error, ParserError::Invalid { .. }));
    }
}
