//! RustyDeps — CLI entrypoint.
//!
//! Phase 2 wires the manifest parsers and the registry clients into `scan`:
//! discover manifests, parse their declared dependencies, then resolve the
//! latest published version for each. SemVer classification and report
//! generation arrive in later phases.

mod cli;
mod model;
mod parser;
mod registry;
mod scanner;

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use tracing::{info, warn};

use crate::cli::{Cli, Command, OutputFormat, ScanArgs};
use crate::model::Ecosystem;
use crate::parser::{parse_manifests, ParsedDependency};
use crate::registry::crates_io::CratesIoClient;
use crate::registry::npm::NpmRegistryClient;
use crate::registry::{fetch_latest_versions, RegistryClient, RegistryError, DEFAULT_CONCURRENCY};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(verbose_requested(&cli));

    match cli.command {
        Command::Scan(args) => run_scan(args).await,
    }
}

/// A dependency after its latest registry version has been resolved.
#[derive(Debug, Serialize)]
struct ResolvedDependency {
    name: String,
    ecosystem: Ecosystem,
    current_version: String,
    latest_version: Option<String>,
    is_dev_dependency: bool,
    manifest_path: PathBuf,
    /// Populated when the registry lookup failed (network, 404, bad response).
    error: Option<String>,
}

/// Map of `(ecosystem, package)` to `(resolved latest version, error message)`.
type VersionLookups = HashMap<(Ecosystem, String), (Option<String>, Option<String>)>;

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

/// Run the `scan` subcommand: discover manifests, parse them, resolve latest
/// registry versions, and report the result.
async fn run_scan(args: ScanArgs) -> Result<()> {
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

    let resolved = resolve_versions(&dependencies).await;

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&resolved)?),
        OutputFormat::Html => print_summary(&resolved),
    }

    info!(
        output = %args.output.display(),
        ai = args.ai,
        "report generation (--output/--ai) arrives in a later phase; this run parses manifests and resolves latest versions"
    );

    Ok(())
}

/// Resolve the latest published version for every dependency, degrading to
/// "unknown" (with the reason recorded) when a registry lookup fails.
async fn resolve_versions(dependencies: &[ParsedDependency]) -> Vec<ResolvedDependency> {
    let mut lookups: VersionLookups = HashMap::new();

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

    dependencies
        .iter()
        .map(|dependency| {
            let (latest_version, error) = lookups
                .get(&(dependency.ecosystem, dependency.name.clone()))
                .cloned()
                .unwrap_or((None, None));

            ResolvedDependency {
                name: dependency.name.clone(),
                ecosystem: dependency.ecosystem,
                current_version: dependency.current_version.clone(),
                latest_version,
                is_dev_dependency: dependency.is_dev_dependency,
                manifest_path: dependency.manifest_path.clone(),
                error,
            }
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

/// Print a human-readable summary table of resolved dependencies.
fn print_summary(dependencies: &[ResolvedDependency]) {
    if dependencies.is_empty() {
        println!("No supported dependencies found.");
        return;
    }

    println!(
        "{:<30} {:<6} {:<16} {:<16} DEV",
        "PACKAGE", "ECO", "CURRENT", "LATEST"
    );
    for dependency in dependencies {
        let latest = dependency.latest_version.as_deref().unwrap_or("unknown");
        let dev = if dependency.is_dev_dependency {
            "yes"
        } else {
            ""
        };
        println!(
            "{:<30} {:<6} {:<16} {:<16} {}",
            dependency.name, dependency.ecosystem, dependency.current_version, latest, dev
        );
    }

    let resolved = dependencies
        .iter()
        .filter(|dependency| dependency.latest_version.is_some())
        .count();
    println!();
    println!(
        "{} dependencies checked, {} resolved against a registry, {} unknown",
        dependencies.len(),
        resolved,
        dependencies.len() - resolved
    );
}
