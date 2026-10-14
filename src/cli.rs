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

    /// Enable AI-driven changelog analysis (arrives in a later phase).
    #[arg(long)]
    pub ai: bool,

    /// Path for the generated report (arrives in a later phase).
    #[arg(short, long, default_value = "report.html")]
    pub output: PathBuf,

    /// Output format for scan results.
    #[arg(long, value_enum, default_value_t = OutputFormat::Html)]
    pub format: OutputFormat,

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
