//! Command-line interface definitions for RustyDeps, built with `clap`'s
//! derive macros.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::config::AiProvider;

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

    /// Enable AI-driven per-library changelog summaries. Also enabled when the
    /// configuration file sets `ai.enabled`.
    #[arg(long)]
    pub ai: bool,

    /// Disable AI summaries for this scan, even when the configuration enables
    /// them.
    #[arg(long)]
    pub no_ai: bool,

    /// Override the configured AI model (for example `gpt-4o-mini`).
    #[arg(long, value_name = "MODEL")]
    pub ai_model: Option<String>,

    /// Override the configured AI provider. Switching providers also resets the
    /// model and endpoint to that provider's defaults unless they are given
    /// explicitly.
    #[arg(long, value_enum, value_name = "PROVIDER")]
    pub ai_provider: Option<AiProvider>,

    /// Override the configured AI endpoint (base URL or full chat-completions
    /// URL).
    #[arg(long, value_name = "URL")]
    pub ai_endpoint: Option<String>,

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

    #[test]
    fn parses_ai_overrides() {
        let cli = Cli::try_parse_from([
            "rustydeps",
            "scan",
            "--ai",
            "--ai-provider",
            "ollama",
            "--ai-model",
            "llama3.2:3b",
            "--ai-endpoint",
            "http://localhost:11434/v1",
        ])
        .expect("parse");
        match cli.command {
            Command::Scan(args) => {
                assert!(args.ai);
                assert!(!args.no_ai);
                assert_eq!(args.ai_provider, Some(AiProvider::Ollama));
                assert_eq!(args.ai_model.as_deref(), Some("llama3.2:3b"));
                assert_eq!(
                    args.ai_endpoint.as_deref(),
                    Some("http://localhost:11434/v1")
                );
            }
        }
    }

    #[test]
    fn no_ai_is_available_as_a_flag() {
        let cli = Cli::try_parse_from(["rustydeps", "scan", "--no-ai"]).expect("parse");
        match cli.command {
            Command::Scan(args) => assert!(args.no_ai),
        }
    }
}
