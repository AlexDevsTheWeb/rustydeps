//! RustyDeps — CLI entrypoint.
//!
//! Phase 1 wires the argument parser and the filesystem scanner. Registry
//! queries, SemVer classification, and report generation arrive in later
//! phases.

mod cli;
mod scanner;

use anyhow::Result;
use clap::Parser;
use tracing::info;

use crate::cli::{Cli, Command, OutputFormat, ScanArgs};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(verbose_requested(&cli));

    match cli.command {
        Command::Scan(args) => run_scan(args).await,
    }
}

/// Configure `tracing-subscriber` for readable console logging.
///
/// `RUST_LOG` wins when set; otherwise the default level is `debug` when
/// `--verbose` is passed and `info` otherwise. Logs go to stderr so that
/// machine-readable output on stdout (for example `--format json`) stays clean.
fn init_tracing(verbose: bool) {
    let default_level = if verbose { "debug" } else { "info" };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

/// Whether the parsed invocation asked for verbose logging.
fn verbose_requested(cli: &Cli) -> bool {
    match &cli.command {
        Command::Scan(args) => args.verbose,
    }
}

/// Run the `scan` subcommand: discover manifests and report what was found.
async fn run_scan(args: ScanArgs) -> Result<()> {
    let manifests = scanner::scan_directory(&args.path)?;

    info!(
        root = %args.path.display(),
        count = manifests.len(),
        "manifest scan complete"
    );

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&manifests)?),
        OutputFormat::Html => {
            for manifest in &manifests {
                println!("{}", manifest.path.display());
            }
        }
    }

    info!(
        output = %args.output.display(),
        ai = args.ai,
        "report generation (--output/--ai) arrives in a later phase; this run only discovers manifests"
    );

    Ok(())
}
