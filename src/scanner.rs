//! Filesystem scanner: discover dependency manifests while honouring ignore
//! rules.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use ignore::WalkBuilder;
use serde::Serialize;
use tracing::debug;

/// Directories never worth descending into for a dependency scan, even when
/// they are not covered by an ignore file.
const SKIPPED_DIRECTORIES: [&str; 3] = ["node_modules", "target", ".git"];

/// A dependency manifest format supported by the scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestType {
    /// `Cargo.toml`
    CargoToml,
    /// `package.json`
    PackageJson,
    /// `go.mod`
    GoMod,
    /// `requirements.txt`
    RequirementsTxt,
    /// `pyproject.toml`
    PyprojectToml,
    /// `composer.json`
    ComposerJson,
}

impl ManifestType {
    /// Return the manifest type recognised by `name`, or `None` when the file
    /// is not a supported manifest.
    pub fn from_file_name(name: &str) -> Option<Self> {
        match name {
            "Cargo.toml" => Some(Self::CargoToml),
            "package.json" => Some(Self::PackageJson),
            "go.mod" => Some(Self::GoMod),
            "requirements.txt" => Some(Self::RequirementsTxt),
            "pyproject.toml" => Some(Self::PyprojectToml),
            "composer.json" => Some(Self::ComposerJson),
            _ => None,
        }
    }

    /// The file name this manifest type is recognised by.
    pub fn file_name(self) -> &'static str {
        match self {
            Self::CargoToml => "Cargo.toml",
            Self::PackageJson => "package.json",
            Self::GoMod => "go.mod",
            Self::RequirementsTxt => "requirements.txt",
            Self::PyprojectToml => "pyproject.toml",
            Self::ComposerJson => "composer.json",
        }
    }
}

/// A manifest found on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveredManifest {
    /// Path to the manifest file.
    pub path: PathBuf,
    /// The manifest format.
    pub manifest_type: ManifestType,
}

/// Walk `root` and return every supported dependency manifest it contains.
///
/// `.gitignore`, `.ignore`, and `.git/info/exclude` rules are honoured, hidden
/// entries are skipped, and the well-known build/dependency directories in
/// [`SKIPPED_DIRECTORIES`] are pruned. Ignore files are respected even when
/// `root` is not itself inside a git repository.
pub fn scan_directory<P: AsRef<Path>>(root: P) -> Result<Vec<DiscoveredManifest>> {
    let root = root.as_ref();

    if !root.exists() {
        bail!("scan root does not exist: {}", root.display());
    }

    let mut walker = WalkBuilder::new(root);
    walker
        .hidden(true)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .ignore(true)
        .parents(true)
        .require_git(false)
        .filter_entry(|entry| !is_skipped_directory(entry));

    let mut manifests = Vec::new();
    for entry in walker.build() {
        let entry =
            entry.with_context(|| format!("failed to read entry under {}", root.display()))?;

        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        if let Some(manifest_type) = ManifestType::from_file_name(&file_name) {
            let path = entry.into_path();
            debug!(
                manifest = manifest_type.file_name(),
                path = %path.display(),
                "discovered manifest"
            );
            manifests.push(DiscoveredManifest {
                path,
                manifest_type,
            });
        }
    }

    manifests.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(manifests)
}

/// `filter_entry` predicate: prune the well-known directories listed in
/// [`SKIPPED_DIRECTORIES`], keep everything else.
fn is_skipped_directory(entry: &ignore::DirEntry) -> bool {
    let is_directory = entry
        .file_type()
        .is_some_and(|file_type| file_type.is_dir());
    is_directory
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| SKIPPED_DIRECTORIES.contains(&name))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    /// Write `contents` to `path`, creating parent directories as needed.
    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn manifest_type_round_trips_file_names() {
        let all = [
            ManifestType::CargoToml,
            ManifestType::PackageJson,
            ManifestType::GoMod,
            ManifestType::RequirementsTxt,
            ManifestType::PyprojectToml,
            ManifestType::ComposerJson,
        ];

        for manifest_type in all {
            assert_eq!(
                ManifestType::from_file_name(manifest_type.file_name()),
                Some(manifest_type)
            );
        }

        assert_eq!(ManifestType::from_file_name("README.md"), None);
    }

    #[test]
    fn discovers_supported_manifests() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("Cargo.toml"), "");
        write_file(&dir.path().join("app/package.json"), "{}");
        write_file(&dir.path().join("service/go.mod"), "module service");
        write_file(&dir.path().join("notes.md"), "not a manifest");

        let found = scan_directory(dir.path()).expect("scan");

        assert_eq!(found.len(), 3);
        let types: Vec<_> = found.iter().map(|m| m.manifest_type).collect();
        assert!(types.contains(&ManifestType::CargoToml));
        assert!(types.contains(&ManifestType::PackageJson));
        assert!(types.contains(&ManifestType::GoMod));
    }

    #[test]
    fn respects_gitignore_rules() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join(".gitignore"), "ignored/\n");
        write_file(&dir.path().join("kept/Cargo.toml"), "");
        write_file(&dir.path().join("ignored/package.json"), "{}");

        let found = scan_directory(dir.path()).expect("scan");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].manifest_type, ManifestType::CargoToml);
    }

    #[test]
    fn skips_well_known_directories() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("Cargo.toml"), "");
        write_file(&dir.path().join("node_modules/pkg/package.json"), "{}");
        write_file(&dir.path().join("target/debug/Cargo.toml"), "");

        let found = scan_directory(dir.path()).expect("scan");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].manifest_type, ManifestType::CargoToml);
    }

    #[test]
    fn skips_hidden_directories() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("Cargo.toml"), "");
        write_file(&dir.path().join(".hidden/package.json"), "{}");

        let found = scan_directory(dir.path()).expect("scan");

        assert_eq!(found.len(), 1);
    }

    #[test]
    fn errors_on_missing_root() {
        let dir = TempDir::new().expect("temp dir");
        let missing = dir.path().join("does-not-exist");

        assert!(scan_directory(&missing).is_err());
    }
}
