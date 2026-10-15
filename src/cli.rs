//! Command-line interface definitions for RustyDeps, built with `clap`'s
//! derive macros.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Top-level CLI: `rustydeps [OPTIONS] <COMMAND>`.
#[derive(Debug, Parser)]
#[command(
    name = "rustydeps",
    version,
    about = "Scan local repositories for dependency manifests and report available upgrades.",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Walk a directory tree and list every supported dependency manifest.
    Scan(ScanArgs),
}

/// Arguments for the `scan` subcommand.
#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Directory to scan (defaults to the current directory).
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Enable AI-driven changelog summaries (templated until the Phase 5 AI
    /// module; live providers are detected from the environment).
    #[arg(long)]
    pub ai: bool,

    /// Path for the generated HTML report (used with `--format html`).
    #[arg(short, long, default_value = "rustydeps-report.html")]
    pub output: PathBuf,

    /// Output format for scan results.
    #[arg(long, value_enum, default_value_t = OutputFormat::Html)]
    pub format: OutputFormat,

    /// Skip registry lookups and classify from local manifest constraints only.
    #[arg(long, alias = "no-fetch")]
    pub offline: bool,

    /// Do not open the generated HTML report in the default browser.
    #[arg(long)]
    pub no_open: bool,

    /// Emit verbose (debug-level) logging.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Report/output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Interactive HTML report.
    Html,
    /// Machine-readable JSON.
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_the_report_by_default() {
        let cli = Cli::try_parse_from(["rustydeps", "scan"]).expect("parse");
        match cli.command {
            Command::Scan(args) => assert!(!args.no_open),
        }
    }

    #[test]
    fn no_open_suppresses_browser_launch() {
        let cli = Cli::try_parse_from(["rustydeps", "scan", "--no-open"]).expect("parse");
        match cli.command {
            Command::Scan(args) => assert!(args.no_open),
        }
    }
}
