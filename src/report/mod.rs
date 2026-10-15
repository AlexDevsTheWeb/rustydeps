//! Interactive HTML report engine.
//!
//! The report consumes the classified dependency set and renders a single
//! self-contained HTML file: CSS and JavaScript are embedded inline, so the
//! file opens offline in any browser with no CDN or network access. The visual
//! contract is `doc/wiki/html-report-spec.md` (already reflected in the
//! template).

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::Path;

use askama::Template;

use crate::classifier::ClassifiedDependency;
use crate::model::{Ecosystem, UpdateType};

/// Metadata about a scan, shown in the report header.
#[derive(Debug, Clone, Default)]
pub struct ReportMeta {
    /// Root that was scanned.
    pub scanned_root: Option<String>,
    /// Wall-clock duration of the scan and registry lookups, in milliseconds.
    pub scan_duration_ms: Option<u128>,
}

/// One dependency row in the report.
#[derive(Debug, Clone)]
pub struct DependencyReport {
    /// Package name.
    pub name: String,
    /// Declared version or requirement.
    pub current_version: String,
    /// Latest version, or `unknown`.
    pub latest_display: String,
    /// Whether the dependency is development-only.
    pub is_dev_dependency: bool,
    /// CSS/filter tag: `breaking`, `at-risk`, `safe`, `up-to-date`, or `unknown`.
    pub risk_class: &'static str,
    /// Human label for the risk, e.g. `major`.
    pub risk_label: &'static str,
    /// Whether the risk is one of the dotted levels.
    pub has_risk_dot: bool,
    /// Copyable CLI update command (empty when there is nothing to update).
    pub update_command: String,
    /// Whether [`Self::update_command`] is present.
    pub has_update_command: bool,
    /// One-line summary (empty when there is none).
    pub ai_summary: String,
    /// Whether [`Self::ai_summary`] is present.
    pub has_ai_summary: bool,
}

/// One project card (one manifest) in the report.
#[derive(Debug, Clone)]
pub struct ProjectReport {
    /// Project name, derived from the manifest's directory.
    pub name: String,
    /// Path to the manifest.
    pub manifest_path: String,
    /// Ecosystem badges present in the project.
    pub ecosystems: Vec<&'static str>,
    /// Number of dependencies in the project.
    pub total: usize,
    /// Count of breaking updates.
    pub breaking: usize,
    /// Count of at-risk updates.
    pub at_risk: usize,
    /// Count of safe updates.
    pub safe: usize,
    /// Combined update command for the whole manifest (empty when none).
    pub update_all_command: String,
    /// Whether [`Self::update_all_command`] is present.
    pub has_update_all: bool,
    /// The project's dependency rows.
    pub dependencies: Vec<DependencyReport>,
}

/// The full render context for the HTML report.
#[derive(Debug, Clone, Template)]
#[template(path = "report.html")]
pub struct ReportData {
    /// RustyDeps version shown in the header.
    pub version: String,
    /// Number of project cards.
    pub project_count: usize,
    /// Number of dependency rows.
    pub dependency_count: usize,
    /// Total breaking updates.
    pub breaking: usize,
    /// Total at-risk updates.
    pub at_risk: usize,
    /// Total safe updates.
    pub safe: usize,
    /// Total up-to-date dependencies.
    pub up_to_date: usize,
    /// Total unknown dependencies.
    pub unknown: usize,
    /// Root that was scanned (empty when unknown).
    pub scanned_root: String,
    /// Scan duration for display, e.g. `142 ms` (or `—`).
    pub duration_display: String,
    /// Whether the scan produced any dependency at all.
    pub has_dependencies: bool,
    /// Project cards, sorted by manifest path.
    pub projects: Vec<ProjectReport>,
}

/// Per-update-type tally.
#[derive(Debug, Default, Clone, Copy)]
struct Tally {
    breaking: usize,
    at_risk: usize,
    safe: usize,
    up_to_date: usize,
    unknown: usize,
}

impl Tally {
    fn add(&mut self, update_type: UpdateType) {
        match update_type {
            UpdateType::Breaking => self.breaking += 1,
            UpdateType::AtRisk => self.at_risk += 1,
            UpdateType::Safe => self.safe += 1,
            UpdateType::UpToDate => self.up_to_date += 1,
            UpdateType::Unknown => self.unknown += 1,
        }
    }

    fn total(self) -> usize {
        self.breaking + self.at_risk + self.safe + self.up_to_date + self.unknown
    }
}

impl ReportData {
    /// Build the render context from the classified dependencies.
    pub fn from_classified(dependencies: &[ClassifiedDependency], meta: &ReportMeta) -> Self {
        let mut groups: BTreeMap<String, Vec<&ClassifiedDependency>> = BTreeMap::new();
        for dependency in dependencies {
            groups
                .entry(dependency.manifest_path.display().to_string())
                .or_default()
                .push(dependency);
        }

        let mut overall = Tally::default();
        let mut projects = Vec::with_capacity(groups.len());

        for (manifest_path, dependencies) in &groups {
            let mut tally = Tally::default();
            let mut badges: Vec<&'static str> = Vec::new();
            let mut commands: Vec<String> = Vec::new();
            let mut rows = Vec::with_capacity(dependencies.len());

            for dependency in dependencies {
                tally.add(dependency.update_type);
                overall.add(dependency.update_type);

                let badge = ecosystem_badge(dependency.ecosystem);
                if !badges.contains(&badge) {
                    badges.push(badge);
                }

                let command = actionable_command(dependency);
                if !command.is_empty() {
                    commands.push(command.clone());
                }

                rows.push(DependencyReport {
                    name: dependency.name.clone(),
                    current_version: dependency.current_version.clone(),
                    latest_display: dependency
                        .latest_version
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    is_dev_dependency: dependency.is_dev_dependency,
                    risk_class: risk_class(dependency.update_type),
                    risk_label: risk_label(dependency.update_type),
                    has_risk_dot: has_risk_dot(dependency.update_type),
                    has_update_command: !command.is_empty(),
                    update_command: command,
                    has_ai_summary: dependency.ai_summary.is_some(),
                    ai_summary: dependency.ai_summary.clone().unwrap_or_default(),
                });
            }

            let update_all_command = commands.join(" && ");
            projects.push(ProjectReport {
                name: project_name(manifest_path),
                manifest_path: manifest_path.clone(),
                ecosystems: badges,
                total: tally.total(),
                breaking: tally.breaking,
                at_risk: tally.at_risk,
                safe: tally.safe,
                has_update_all: !update_all_command.is_empty(),
                update_all_command,
                dependencies: rows,
            });
        }

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_count: projects.len(),
            dependency_count: dependencies.len(),
            breaking: overall.breaking,
            at_risk: overall.at_risk,
            safe: overall.safe,
            up_to_date: overall.up_to_date,
            unknown: overall.unknown,
            scanned_root: meta.scanned_root.clone().unwrap_or_default(),
            duration_display: match meta.scan_duration_ms {
                Some(ms) => format!("{ms} ms"),
                None => "—".to_string(),
            },
            has_dependencies: !dependencies.is_empty(),
            projects,
        }
    }
}

impl ReportData {
    /// Render the report to a self-contained HTML document.
    pub fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}

/// The project name derives from the manifest's parent directory.
fn project_name(manifest_path: &str) -> String {
    Path::new(manifest_path)
        .parent()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| ".".to_string())
}

/// Ecosystem badge label.
fn ecosystem_badge(ecosystem: Ecosystem) -> &'static str {
    match ecosystem {
        Ecosystem::Cargo => "CARGO",
        Ecosystem::Npm => "NPM",
    }
}

/// CSS/filter tag for an update type.
fn risk_class(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "breaking",
        UpdateType::AtRisk => "at-risk",
        UpdateType::Safe => "safe",
        UpdateType::UpToDate => "up-to-date",
        UpdateType::Unknown => "unknown",
    }
}

/// Human label for an update type.
fn risk_label(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "major",
        UpdateType::AtRisk => "minor",
        UpdateType::Safe => "patch",
        UpdateType::UpToDate => "up-to-date",
        UpdateType::Unknown => "unknown",
    }
}

/// Whether the update type carries a solid risk dot.
fn has_risk_dot(update_type: UpdateType) -> bool {
    matches!(
        update_type,
        UpdateType::Breaking | UpdateType::AtRisk | UpdateType::Safe
    )
}

/// The update command for a dependency, when there is something to update.
fn actionable_command(dependency: &ClassifiedDependency) -> String {
    let actionable = matches!(
        dependency.update_type,
        UpdateType::Breaking | UpdateType::AtRisk | UpdateType::Safe
    );
    match (actionable, dependency.latest_version.as_deref()) {
        (true, Some(latest)) => update_command(dependency.ecosystem, &dependency.name, latest),
        _ => String::new(),
    }
}

/// The exact CLI command that upgrades `name` to `latest`.
fn update_command(ecosystem: Ecosystem, name: &str, latest: &str) -> String {
    match ecosystem {
        Ecosystem::Cargo => format!("cargo update -p {name} --precise {latest}"),
        Ecosystem::Npm => format!("npm install {name}@{latest}"),
    }
}
