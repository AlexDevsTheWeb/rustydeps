//! RustyDeps — CLI entrypoint.
//!
//! `scan` discovers manifests, parses their declared dependencies, resolves the
//! latest published version for each (unless `--offline`), classifies the
//! update, and attaches a summary. Report generation arrives in Phase 4.

mod ai;
mod classifier;
mod cli;
mod model;
mod parser;
mod registry;
mod scanner;

use std::collections::{BTreeSet, HashMap};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use tracing::{info, warn};

use crate::ai::Summarizer;
use crate::classifier::{classify, ClassifiedDependency};
use crate::cli::{Cli, Command, OutputFormat, ScanArgs};
use crate::model::{Ecosystem, UpdateType};
use crate::parser::{parse_manifests, ParsedDependency};
use crate::registry::crates_io::CratesIoClient;
use crate::registry::npm::NpmRegistryClient;
use crate::registry::{fetch_latest_versions, RegistryClient, RegistryError, DEFAULT_CONCURRENCY};

/// Map of `(ecosystem, package)` to `(resolved latest version, error message)`.
type VersionLookups = HashMap<(Ecosystem, String), (Option<String>, Option<String>)>;

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

/// Print the ASCII banner: Rust-orange logo, cyan version badge, light-grey
/// tagline. Colours are dropped automatically when stdout is not a terminal.
fn print_banner() {
    const LOGO: &str = " █▀█ █  █ █▀▀ ▀█▀ █  █ █▀▄ █▀▀ █▀█ █▀\n \
                        █▀▄ █▄▄█ ▄██  █  ▀▄▄█ █▄▀ ██▄ █▀▀ ▄█";
    const RUST_ORANGE: (u8, u8, u8) = (0xea, 0x58, 0x0c);
    const CYAN: (u8, u8, u8) = (0x06, 0xb6, 0xd4);
    const LIGHT_GREY: (u8, u8, u8) = (0xe4, 0xe4, 0xe7);

    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "{} {}",
        LOGO.truecolor(RUST_ORANGE.0, RUST_ORANGE.1, RUST_ORANGE.2),
        version.as_str().truecolor(CYAN.0, CYAN.1, CYAN.2)
    );
    println!();
    println!(
        "{}",
        " 🦀 Smart cross-registry dependency intelligence for Rust, Node & Python".truecolor(
            LIGHT_GREY.0,
            LIGHT_GREY.1,
            LIGHT_GREY.2
        )
    );
    println!();
}

/// Run the `scan` subcommand: discover manifests, parse them, resolve latest
/// registry versions (unless offline), and classify each dependency.
async fn run_scan(args: ScanArgs) -> Result<()> {
    if args.format != OutputFormat::Json {
        print_banner();
    }

    let manifests = scanner::scan_directory(&args.path)?;
    info!(
        root = %args.path.display(),
        count = manifests.len(),
        "manifest scan complete"
    );

    let outcome = parse_manifests(&manifests, &parser::default_parsers());
    for (path, error) in &outcome.failures {
        warn!(
            path = %path.display(),
            error = %error,
            "skipping manifest that could not be parsed"
        );
    }

    let dependencies = outcome.dependencies;
    info!(count = dependencies.len(), "parsed dependencies");

    let lookups = if args.offline {
        info!("offline mode: skipping registry lookups; updates classify as unknown");
        VersionLookups::new()
    } else {
        resolve_lookups(&dependencies).await
    };

    let summarizer = Summarizer::new(args.ai);
    let classified = classify_all(&dependencies, &lookups, &summarizer);

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&classified)?),
        OutputFormat::Html => print_summary(&classified),
    }

    info!(
        output = %args.output.display(),
        ai = summarizer.is_enabled(),
        ai_provider = summarizer.provider().map(|provider| provider.label()).unwrap_or("none"),
        offline = args.offline,
        "report generation (--output) arrives in Phase 4; this run resolves and classifies dependencies"
    );

    Ok(())
}

/// Resolve the latest published version for every dependency, degrading to
/// "unknown" (with the reason recorded) when a registry lookup fails.
async fn resolve_lookups(dependencies: &[ParsedDependency]) -> VersionLookups {
    let mut lookups = VersionLookups::new();

    let cargo_names = unique_names(dependencies, Ecosystem::Cargo);
    if !cargo_names.is_empty() {
        match CratesIoClient::new() {
            Ok(client) => {
                record(
                    &mut lookups,
                    client.ecosystem(),
                    fetch_latest_versions(&client, &cargo_names, DEFAULT_CONCURRENCY).await,
                );
            }
            Err(error) => warn!(
                %error,
                "could not build the crates.io client; Cargo versions will be unknown"
            ),
        }
    }

    let npm_names = unique_names(dependencies, Ecosystem::Npm);
    if !npm_names.is_empty() {
        match NpmRegistryClient::new() {
            Ok(client) => {
                record(
                    &mut lookups,
                    client.ecosystem(),
                    fetch_latest_versions(&client, &npm_names, DEFAULT_CONCURRENCY).await,
                );
            }
            Err(error) => warn!(
                %error,
                "could not build the npm registry client; npm versions will be unknown"
            ),
        }
    }

    lookups
}

/// Classify every dependency against its resolved latest version and attach a
/// summary.
fn classify_all(
    dependencies: &[ParsedDependency],
    lookups: &VersionLookups,
    summarizer: &Summarizer,
) -> Vec<ClassifiedDependency> {
    dependencies
        .iter()
        .map(|dependency| {
            let (latest, error) = lookups
                .get(&(dependency.ecosystem, dependency.name.clone()))
                .cloned()
                .unwrap_or((None, None));

            if let Some(error) = &error {
                warn!(
                    package = %dependency.name,
                    ecosystem = %dependency.ecosystem,
                    %error,
                    "registry lookup failed"
                );
            }

            let mut classified = classify(dependency, latest.as_deref());
            classified.ai_summary = summarizer.summarize(classified.update_type);
            classified
        })
        .collect()
}

/// Distinct dependency names for one ecosystem.
fn unique_names(dependencies: &[ParsedDependency], ecosystem: Ecosystem) -> Vec<String> {
    dependencies
        .iter()
        .filter(|dependency| dependency.ecosystem == ecosystem)
        .map(|dependency| dependency.name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Fold registry lookups into the resolution map, flattening each result into
/// an optional version plus an optional error message.
fn record(
    lookups: &mut VersionLookups,
    ecosystem: Ecosystem,
    results: Vec<(String, Result<String, RegistryError>)>,
) {
    for (name, result) in results {
        let entry = match result {
            Ok(version) => (Some(version), None),
            Err(error) => (None, Some(error.to_string())),
        };
        lookups.insert((ecosystem, name), entry);
    }
}

/// Print a human-readable summary table of classified dependencies.
fn print_summary(dependencies: &[ClassifiedDependency]) {
    if dependencies.is_empty() {
        println!("No supported dependencies found.");
        return;
    }

    println!(
        "{:<28} {:<6} {:<14} {:<14} {:<10} DEV",
        "PACKAGE", "ECO", "CURRENT", "LATEST", "UPDATE"
    );
    for dependency in dependencies {
        let latest = dependency.latest_version.as_deref().unwrap_or("unknown");
        let dev = if dependency.is_dev_dependency {
            "yes"
        } else {
            ""
        };
        println!(
            "{:<28} {:<6} {:<14} {:<14} {:<10} {}",
            dependency.name,
            dependency.ecosystem,
            dependency.current_version,
            latest,
            dependency.update_type,
            dev
        );
    }

    println!();
    println!("{}", tally(dependencies));

    let summaries: Vec<&ClassifiedDependency> = dependencies
        .iter()
        .filter(|dependency| dependency.ai_summary.is_some())
        .collect();
    if !summaries.is_empty() {
        println!();
        println!("Summaries:");
        for dependency in summaries {
            println!(
                "  {} — {}",
                dependency.name,
                dependency.ai_summary.as_deref().unwrap_or_default()
            );
        }
    }
}

/// One-line tally of update types across the report.
fn tally(dependencies: &[ClassifiedDependency]) -> String {
    let count = |update_type: UpdateType| {
        dependencies
            .iter()
            .filter(|dependency| dependency.update_type == update_type)
            .count()
    };

    format!(
        "{} dependencies: {} up-to-date, {} safe, {} at-risk, {} breaking, {} unknown",
        dependencies.len(),
        count(UpdateType::UpToDate),
        count(UpdateType::Safe),
        count(UpdateType::AtRisk),
        count(UpdateType::Breaking),
        count(UpdateType::Unknown),
    )
}
