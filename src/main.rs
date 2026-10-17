//! RustyDeps — CLI entrypoint.
//!
//! `scan` discovers manifests, parses their declared dependencies, resolves the
//! latest published version for each (unless `--offline`), classifies the
//! update, attaches a summary, and renders a self-contained HTML report (or
//! JSON).

mod ai;
mod classifier;
mod cli;
mod model;
mod parser;
mod progress;
mod registry;
mod report;
mod scanner;

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use tracing::{info, warn};

use crate::ai::Summarizer;
use crate::classifier::{classify, ClassifiedDependency};
use crate::cli::{Cli, Command, OutputFormat, ScanArgs};
use crate::model::{Ecosystem, UpdateType};
use crate::parser::{parse_manifests, ParsedDependency};
use crate::registry::composer::ComposerClient;
use crate::registry::crates_io::CratesIoClient;
use crate::registry::go::GoRegistryClient;
use crate::registry::npm::NpmRegistryClient;
use crate::registry::pypi::PypiClient;
use crate::registry::{fetch_latest_versions, RegistryClient, RegistryError, DEFAULT_CONCURRENCY};
use crate::report::{ReportData, ReportMeta};

/// Map of `(ecosystem, package)` to `(resolved latest version, error message)`.
type VersionLookups = HashMap<(Ecosystem, String), (Option<String>, Option<String>)>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // `--format json` must keep stdout a single JSON document: no banner, no
    // progress animations, no per-repository status lines.
    let machine_readable = matches!(
        &cli.command,
        Command::Scan(args) if args.format == OutputFormat::Json
    );
    let progress = progress::ScanProgress::new(!machine_readable);
    init_tracing(
        verbose_requested(&cli),
        progress::ProgressWriter::new(progress.multi().clone()),
    );

    match cli.command {
        Command::Scan(args) => run_scan(args, progress).await,
    }
}

/// Configure `tracing-subscriber` for readable console logging.
///
/// `RUST_LOG` wins when set; otherwise the default level is `debug` when
/// `--verbose` is passed and `info` otherwise. Logs go to stderr so that
/// machine-readable output on stdout (for example `--format json`) stays clean,
/// and they are written through `writer`, which clears any active progress bar
/// before each line and redraws it afterwards.
fn init_tracing(verbose: bool, writer: progress::ProgressWriter) {
    let default_level = if verbose { "debug" } else { "info" };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
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
async fn run_scan(args: ScanArgs, progress: progress::ScanProgress) -> Result<()> {
    let machine_readable = args.format == OutputFormat::Json;
    if !machine_readable {
        print_banner();
    }

    let started = Instant::now();

    let discovery = progress.spinner("Discovering dependency manifests…");
    let manifests = scanner::scan_directory(&args.path)?;
    discovery.finish_and_clear();
    info!(
        root = %args.path.display(),
        count = manifests.len(),
        "manifest scan complete"
    );

    let parsing = progress.spinner("Parsing manifests…");
    let outcome = parse_manifests(&manifests, &parser::default_parsers());
    parsing.finish_and_clear();
    for (path, error) in &outcome.failures {
        warn!(
            path = %path.display(),
            error = %error,
            "skipping manifest that could not be parsed"
        );
    }

    let dependencies = outcome.dependencies;
    info!(count = dependencies.len(), "parsed dependencies");

    if !machine_readable {
        report_repositories(&manifests, &dependencies, &args.path, &progress);
    }

    let lookups = if args.offline {
        info!("offline mode: skipping registry lookups; updates classify as unknown");
        VersionLookups::new()
    } else {
        let resolving = progress.spinner("Resolving latest versions from registries…");
        let lookups = resolve_lookups(&dependencies).await;
        resolving.finish_and_clear();
        lookups
    };

    let summarizer = Summarizer::new(args.ai);
    let classified = classify_all(&dependencies, &lookups, &summarizer);
    let duration_ms = started.elapsed().as_millis();

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&classified)?),
        OutputFormat::Html => {
            print_summary(&classified);

            let meta = ReportMeta {
                scan_duration_ms: Some(duration_ms),
            };
            let report = ReportData::from_classified(&classified, &meta);
            let html = report
                .render_html()
                .context("failed to render the HTML report")?;
            write_report(&args.output, &html)?;

            let absolute =
                std::fs::canonicalize(&args.output).unwrap_or_else(|_| args.output.clone());
            println!();
            println!("HTML report written to {}", args.output.display());
            println!("Open it in your browser: file://{}", absolute.display());

            if !args.no_open {
                match open::that(&absolute) {
                    Ok(()) => info!(
                        report = %absolute.display(),
                        "opened the report in the default browser"
                    ),
                    Err(error) => warn!(
                        %error,
                        report = %absolute.display(),
                        "could not open the report in the browser"
                    ),
                }
            }
        }
    }

    info!(
        output = %args.output.display(),
        ai = summarizer.is_enabled(),
        ai_provider = summarizer.provider().map(|provider| provider.label()).unwrap_or("none"),
        offline = args.offline,
        duration_ms,
        "scan complete"
    );

    Ok(())
}

/// Print one status line per discovered repository while advancing a progress
/// bar, so a large scan visibly makes progress instead of appearing frozen.
///
/// The bar's total is the manifest count, which is known only once discovery
/// finishes — hence the spinner during discovery and the counter here. The
/// status lines are printed through the reporter so they appear *above* the bar
/// without leaving duplicated bar frames behind.
fn report_repositories(
    manifests: &[scanner::DiscoveredManifest],
    dependencies: &[ParsedDependency],
    root: &Path,
    progress: &progress::ScanProgress,
) {
    if manifests.is_empty() {
        progress.println(format!(
            "No supported dependency manifest found under {}.",
            root.display()
        ));
        return;
    }

    let bar = progress.bar(manifests.len() as u64, "Scanning repositories");

    let mut counts: HashMap<&Path, usize> = HashMap::new();
    for dependency in dependencies {
        *counts
            .entry(dependency.manifest_path.as_path())
            .or_default() += 1;
    }

    for manifest in manifests {
        let count = counts.get(manifest.path.as_path()).copied().unwrap_or(0);
        let label = progress::project_label(root, &manifest.path);
        progress.println(format!(
            "{} {}",
            "✔".green(),
            progress::repository_summary(&label, count, manifest.manifest_type.ecosystem_label())
        ));
        bar.inc(1);
    }

    bar.finish_and_clear();
}

/// Write the rendered report, creating parent directories when needed.
fn write_report(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
    }

    std::fs::write(path, contents)
        .with_context(|| format!("failed to write report to {}", path.display()))?;
    Ok(())
}

/// Resolve the latest published version for every dependency, degrading to
/// "unknown" (with the reason recorded) when a registry lookup fails.
async fn resolve_lookups(dependencies: &[ParsedDependency]) -> VersionLookups {
    let mut lookups = VersionLookups::new();

    resolve_ecosystem(
        &mut lookups,
        dependencies,
        Ecosystem::Cargo,
        CratesIoClient::new,
        "crates.io",
    )
    .await;
    resolve_ecosystem(
        &mut lookups,
        dependencies,
        Ecosystem::Npm,
        NpmRegistryClient::new,
        "npm registry",
    )
    .await;
    resolve_ecosystem(
        &mut lookups,
        dependencies,
        Ecosystem::Pypi,
        PypiClient::new,
        "PyPI",
    )
    .await;
    resolve_ecosystem(
        &mut lookups,
        dependencies,
        Ecosystem::Go,
        GoRegistryClient::new,
        "Go module proxy",
    )
    .await;
    resolve_ecosystem(
        &mut lookups,
        dependencies,
        Ecosystem::Composer,
        ComposerClient::new,
        "Packagist",
    )
    .await;

    lookups
}

/// Fetch the latest versions for one ecosystem, degrading gracefully when the
/// client cannot be built or a lookup fails.
async fn resolve_ecosystem<C, F>(
    lookups: &mut VersionLookups,
    dependencies: &[ParsedDependency],
    ecosystem: Ecosystem,
    build: F,
    label: &str,
) where
    C: RegistryClient,
    F: FnOnce() -> Result<C, reqwest::Error>,
{
    let names = unique_names(dependencies, ecosystem);
    if names.is_empty() {
        return;
    }

    match build() {
        Ok(client) => record(
            lookups,
            client.ecosystem(),
            fetch_latest_versions(&client, &names, DEFAULT_CONCURRENCY).await,
        ),
        Err(error) => warn!(
            %error,
            "could not build the {label} client; versions will be unknown"
        ),
    }
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
