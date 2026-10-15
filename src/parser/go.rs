//! Parser for Go modules (`go.mod`).
//!
//! Extracts direct `require` directives (block and single-line forms). Indirect
//! requirements (`// indirect`) and `replace`/`exclude` directives are ignored.
//! Versions keep their `v` prefix, which is an exact pin in `go.mod` and is
//! understood by the classifier.

use std::path::Path;

use crate::model::Ecosystem;
use crate::parser::{PackageParser, ParsedDependency, ParserError};
use crate::scanner::{DiscoveredManifest, ManifestType};

/// Decodes `go.mod` files.
#[derive(Debug, Default)]
pub struct GoParser;

impl GoParser {
    fn parse_contents(&self, path: &Path, contents: &str) -> Vec<ParsedDependency> {
        let mut dependencies = Vec::new();
        let mut in_require_block = false;

        for raw in contents.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if in_require_block {
                if line.starts_with(')') {
                    in_require_block = false;
                } else if let Some(dep) = require_dependency(path, line) {
                    dependencies.push(dep);
                }
                continue;
            }

            if line == "require (" || line.starts_with("require(") {
                in_require_block = true;
            } else if let Some(rest) = line.strip_prefix("require ") {
                if let Some(dep) = require_dependency(path, rest.trim()) {
                    dependencies.push(dep);
                }
            }
        }

        dependencies
    }
}

impl PackageParser for GoParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Go
    }

    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool {
        manifest.manifest_type == ManifestType::GoMod
    }

    fn parse(&self, manifest: &DiscoveredManifest) -> Result<Vec<ParsedDependency>, ParserError> {
        let contents =
            std::fs::read_to_string(&manifest.path).map_err(|source| ParserError::Io {
                path: manifest.path.clone(),
                source,
            })?;
        Ok(self.parse_contents(&manifest.path, &contents))
    }
}

/// Parse a single `require` body: `module version [// indirect]`.
fn require_dependency(path: &Path, line: &str) -> Option<ParsedDependency> {
    if line.contains("// indirect") {
        return None;
    }

    let body = line.split("//").next().unwrap_or("").trim();
    let mut parts = body.split_whitespace();
    let module = parts.next()?;
    let version = parts.next().unwrap_or("*");

    Some(ParsedDependency {
        name: module.to_string(),
        current_version: version.to_string(),
        ecosystem: Ecosystem::Go,
        manifest_path: path.to_path_buf(),
        is_dev_dependency: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(contents: &str) -> Vec<ParsedDependency> {
        GoParser.parse_contents(Path::new("go.mod"), contents)
    }

    fn names(deps: &[ParsedDependency]) -> Vec<&str> {
        deps.iter().map(|dep| dep.name.as_str()).collect()
    }

    #[test]
    fn can_parse_only_go_mod() {
        assert!(GoParser.can_parse(&DiscoveredManifest {
            path: "go.mod".into(),
            manifest_type: ManifestType::GoMod,
        }));
        assert!(!GoParser.can_parse(&DiscoveredManifest {
            path: "Cargo.toml".into(),
            manifest_type: ManifestType::CargoToml,
        }));
        assert_eq!(GoParser.ecosystem(), Ecosystem::Go);
    }

    #[test]
    fn parses_require_blocks_and_single_lines() {
        let deps = parse(
            r#"
module github.com/acme/service

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
    golang.org/x/text v0.14.0 // indirect
    github.com/stretchr/testify v1.8.0
)

require github.com/google/uuid v1.6.0
"#,
        );

        assert_eq!(
            names(&deps),
            vec![
                "github.com/gin-gonic/gin",
                "github.com/stretchr/testify",
                "github.com/google/uuid"
            ]
        );
        assert_eq!(deps[0].current_version, "v1.9.1");
        assert!(deps.iter().all(|dep| dep.ecosystem == Ecosystem::Go));
    }

    #[test]
    fn ignores_replace_and_exclude_directives() {
        let deps = parse(
            r#"
module example.com/app

require example.com/dep v1.2.3

replace example.com/dep => ../dep
exclude example.com/other v1.0.0
"#,
        );

        assert_eq!(names(&deps), vec!["example.com/dep"]);
    }
}
