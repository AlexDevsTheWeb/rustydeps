//! Console progress reporting for scans.
//!
//! Wraps `indicatif` so the scanner can show an animated spinner while it
//! discovers manifests and queries registries, and a per-repository progress bar
//! while it reports each manifest. All bars share a single [`MultiProgress`], and
//! human/log output is bracketed with a *suspend*: the bars are cleared before
//! the text is written and redrawn afterwards. That is what keeps a scan from
//! leaving orphaned `Scanning repositories …` frames behind in the terminal
//! scrollback.
//!
//! Everything degrades to silent no-ops when stderr is not an interactive
//! terminal or when the caller is emitting machine-readable output (for example
//! `--format json`), so CI logs and piped output stay clean.

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

/// Spinner template: a cyan spinner followed by the message.
const SPINNER_TEMPLATE: &str = "{spinner:.cyan} {msg}";
/// Progress-bar template: message, bar, and `pos/len` counter.
const BAR_TEMPLATE: &str = "{msg} [{bar:32.cyan/blue}] {pos}/{len}";
/// How often the spinner advances while a step is in flight.
const TICK_INTERVAL: Duration = Duration::from_millis(80);

/// Coordinates the scan's progress bars and the text printed around them.
///
/// Cloning is cheap (the underlying [`MultiProgress`] is shared), so the same
/// reporter drives every phase of a scan and the tracing writer.
#[derive(Clone)]
pub struct ScanProgress {
    interactive: bool,
    multi: MultiProgress,
}

impl ScanProgress {
    /// Create a reporter. Animations are drawn only when `enabled` is true *and*
    /// stderr is an interactive terminal, so non-TTY and CI runs stay quiet.
    pub fn new(enabled: bool) -> Self {
        let interactive = enabled && io::stderr().is_terminal();
        let multi = if interactive {
            MultiProgress::new()
        } else {
            MultiProgress::with_draw_target(ProgressDrawTarget::hidden())
        };
        Self { interactive, multi }
    }

    /// The shared [`MultiProgress`], for wiring a suspending tracing writer.
    pub fn multi(&self) -> &MultiProgress {
        &self.multi
    }
    /// An animated spinner for an indeterminate step (discovery, resolution).
    pub fn spinner(&self, message: impl Into<String>) -> ProgressBar {
        if !self.interactive {
            return ProgressBar::hidden();
        }

        let bar = ProgressBar::new_spinner();
        bar.set_style(spinner_style());
        bar.set_message(message.into());
        bar.enable_steady_tick(TICK_INTERVAL);
        self.multi.add(bar)
    }

    /// A determinate progress bar over `length` items, with a `pos/len` counter.
    pub fn bar(&self, length: u64, message: impl Into<String>) -> ProgressBar {
        if !self.interactive {
            return ProgressBar::hidden();
        }

        let bar = ProgressBar::new(length);
        bar.set_style(bar_style());
        bar.set_message(message.into());
        self.multi.add(bar)
    }

    /// Print a human-readable line to stdout *above* any active progress bar,
    /// clearing and redrawing the bars so no orphan frame is left behind. In a
    /// non-interactive run this is a plain `println!`.
    pub fn println(&self, message: impl AsRef<str>) {
        let message = message.as_ref();
        if self.interactive {
            self.multi.suspend(|| println!("{message}"));
        } else {
            println!("{message}");
        }
    }
}

/// A `tracing-subscriber` writer that brackets each log record with a progress
/// suspend, clearing the bars while the line is written and redrawing them
/// afterwards. Buffering the record and flushing on drop keeps it to a single
/// suspend per event.
#[derive(Clone)]
pub struct ProgressWriter {
    multi: MultiProgress,
}

impl ProgressWriter {
    /// Build a writer backed by the reporter's shared bars.
    pub fn new(multi: MultiProgress) -> Self {
        Self { multi }
    }
}

/// One buffered log record; writes the buffer to stderr on drop, with the bars
/// suspended.
pub struct BufferedEvent {
    multi: MultiProgress,
    buffer: Vec<u8>,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for ProgressWriter {
    type Writer = BufferedEvent;

    fn make_writer(&'a self) -> Self::Writer {
        BufferedEvent {
            multi: self.multi.clone(),
            buffer: Vec::with_capacity(128),
        }
    }
}

impl Write for BufferedEvent {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for BufferedEvent {
    fn drop(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        self.multi.suspend(|| {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            let _ = handle.write_all(&self.buffer);
            let _ = handle.flush();
        });
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

    #[test]
    fn hidden_progress_bars_never_draw() {
        // A disabled reporter must hand back hidden bars, not stderr-backed ones.
        let progress = ScanProgress::new(false);
        let bar = progress.bar(3, "scanning");
        assert!(bar.is_hidden());
    }
}
