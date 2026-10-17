//! Console progress reporting for scans.
//!
//! Thin wrapper over `indicatif` so the scanner can show an animated spinner
//! while it discovers manifests and queries registries, and a per-repository
//! progress bar while it reports each manifest. Everything degrades to silent
//! no-ops when stderr is not an interactive terminal or when the caller is
//! emitting machine-readable output (for example `--format json`), so CI logs
//! and piped output stay clean.

use std::io::IsTerminal;
use std::path::Path;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Spinner template: a cyan spinner followed by the message.
const SPINNER_TEMPLATE: &str = "{spinner:.cyan} {msg}";
/// Progress-bar template: message, bar, and `pos/len` counter.
const BAR_TEMPLATE: &str = "{msg} [{bar:32.cyan/blue}] {pos}/{len}";
/// How often the spinner advances while a step is in flight.
const TICK_INTERVAL: Duration = Duration::from_millis(80);

/// Maps onto `indicatif` so progress reporting can be switched off without the
/// call sites branching on interactivity.
#[derive(Debug, Clone, Copy)]
pub struct ScanProgress {
    interactive: bool,
}

impl ScanProgress {
    /// Create a reporter. Animations are drawn only when `enabled` is true *and*
    /// stderr is an interactive terminal, so non-TTY and CI runs stay quiet.
    pub fn new(enabled: bool) -> Self {
        Self {
            interactive: enabled && std::io::stderr().is_terminal(),
        }
    }

    /// An animated spinner for an indeterminate step (discovery, resolution).
    pub fn spinner(self, message: impl Into<String>) -> ProgressBar {
        if !self.interactive {
            return ProgressBar::hidden();
        }

        let bar = ProgressBar::new_spinner();
        bar.set_style(spinner_style());
        bar.set_message(message.into());
        bar.enable_steady_tick(TICK_INTERVAL);
        bar
    }

    /// A determinate progress bar over `length` items, with a `pos/len` counter.
    pub fn bar(self, length: u64, message: impl Into<String>) -> ProgressBar {
        if !self.interactive {
            return ProgressBar::hidden();
        }

        let bar = ProgressBar::new(length);
        bar.set_style(bar_style());
        bar.set_message(message.into());
        bar
    }
}

/// Spinner style, falling back to `indicatif`'s default if the template fails.
fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template(SPINNER_TEMPLATE)
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
}

/// Progress-bar style, falling back to `indicatif`'s default if it fails.
fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template(BAR_TEMPLATE).unwrap_or_else(|_| ProgressStyle::default_bar())
}

/// Human-readable project name for a manifest path, relative to the scan root
/// when the two share a prefix (`.` for a manifest sitting at the root).
pub fn project_label(root: &Path, manifest_path: &Path) -> String {
    let directory = manifest_path.parent().unwrap_or(manifest_path);
    let relative = directory.strip_prefix(root).unwrap_or(directory);
    if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.display().to_string()
    }
}

/// One-line per-repository summary, e.g.
/// `Scanned 'mono/backend': 18 dependencies (Cargo)`.
pub fn repository_summary(label: &str, dependencies: usize, ecosystem: &str) -> String {
    format!(
        "Scanned '{label}': {dependencies} {} ({ecosystem})",
        dependency_noun(dependencies)
    )
}

/// `dependency` for a single item, `dependencies` otherwise.
fn dependency_noun(count: usize) -> &'static str {
    if count == 1 {
        "dependency"
    } else {
        "dependencies"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_summary_pluralizes_dependencies() {
        assert_eq!(
            repository_summary("backend", 0, "Cargo"),
            "Scanned 'backend': 0 dependencies (Cargo)"
        );
        assert_eq!(
            repository_summary("backend", 1, "npm"),
            "Scanned 'backend': 1 dependency (npm)"
        );
        assert_eq!(
            repository_summary("my-monorepo/backend", 18, "Cargo"),
            "Scanned 'my-monorepo/backend': 18 dependencies (Cargo)"
        );
    }

    #[test]
    fn project_label_is_relative_to_the_scan_root() {
        let root = Path::new("/work/mono");
        assert_eq!(
            project_label(root, Path::new("/work/mono/backend/Cargo.toml")),
            "backend"
        );
        assert_eq!(project_label(root, Path::new("/work/mono/Cargo.toml")), ".");
        assert_eq!(
            project_label(root, Path::new("/work/mono/a/b/package.json")),
            "a/b"
        );
    }

    #[test]
    fn project_label_falls_back_when_paths_diverge() {
        let root = Path::new("/work/mono");
        assert_eq!(project_label(root, Path::new("Cargo.toml")), ".");
    }
}
