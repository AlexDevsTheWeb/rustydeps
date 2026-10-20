//! Interactive HTML report engine.
//!
//! The report consumes the classified dependency set and renders a single
//! self-contained HTML file: CSS and JavaScript are embedded inline, so the
//! file opens offline in any browser with no CDN or network access.
//!
//! The template (`templates/report.html`) is the Google Stitch "UI v4" export
//! (`src/templates/report_stitch.html`) with its static mock rows replaced by
//! Askama loops over the data below. The visual contract is
//! `doc/wiki/html-report-spec.md`.

#[cfg(test)]
mod tests;

mod markdown;

use std::collections::BTreeMap;
use std::path::Path;

use askama::Template;

use crate::classifier::ClassifiedDependency;
use crate::model::{Ecosystem, UpdateType};

/// Metadata about a scan, shown in the report header.
#[derive(Debug, Clone, Default)]
pub struct ReportMeta {
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
    /// Whether the target version is a breaking (major) jump.
    pub is_breaking: bool,
    /// Filter tag consumed by the report JS: `breaking`, `risk`, `safe`,
    /// `up-to-date`, or `unknown`.
    pub status: &'static str,
    /// Whether the row carries a solid risk dot.
    pub has_dot: bool,
    /// Dot CSS class (`dot-breaking`/`dot-risk`/`dot-safe`).
    pub dot_class: &'static str,
    /// Risk label (`major`/`minor`/`patch`/`up-to-date`/`unknown`).
    pub risk_label: &'static str,
    /// Whether the insight column has text.
    pub has_insight: bool,
    /// Whether the row carries a full AI analysis in its drawer.
    pub has_ai_analysis: bool,
    /// Whether the AI analysis came from the local cache.
    pub ai_cached: bool,
    /// Safe HTML rendering of the AI Markdown analysis (empty without AI).
    pub ai_analysis_html: String,
    /// One-line insight shown in the row.
    pub insight: String,
    /// Link to the package's published versions/changelog.
    pub changelog_url: String,
    /// Whether there is an update command to copy.
    pub has_command: bool,
    /// Copyable CLI update command (empty when there is nothing to update).
    pub command: String,
    /// Unique DOM id for the row's inline drawer.
    pub drawer_id: String,
    /// Whether the drawer shows a semver bump badge.
    pub has_bump_badge: bool,
    /// Semver bump badge text, e.g. `Major Bump: 1.35.1 → 2.0.0`.
    pub bump_label: String,
    /// Drawer summary title.
    pub drawer_title: &'static str,
    /// Drawer summary body.
    pub drawer_desc: String,
}

/// One project card (one manifest) in the report.
#[derive(Debug, Clone)]
pub struct ProjectReport {
    /// Project name, derived from the manifest's directory.
    pub name: String,
    /// Path to the manifest.
    pub manifest_path: String,
    /// Ecosystem pill label, e.g. `Cargo` or `npm`.
    pub ecosystem_label: &'static str,
    /// Directory the update command should be run in.
    pub run_in: String,
    /// Count of breaking updates.
    pub breaking: usize,
    /// Count of at-risk updates.
    pub at_risk: usize,
    /// Count of safe updates.
    pub safe: usize,
    /// Whether the project has a combined update command.
    pub has_update_all: bool,
    /// Combined update command for the whole manifest (empty when none).
    pub update_all_command: String,
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
    /// Scan duration for display, e.g. `142 ms` (or `—`).
    pub duration_display: String,
    /// Total breaking updates.
    pub breaking: usize,
    /// Total at-risk updates.
    pub at_risk: usize,
    /// Total safe updates.
    pub safe: usize,
    /// Whether the scan produced any dependency at all.
    pub has_dependencies: bool,
    /// Project cards, sorted by manifest path.
    pub projects: Vec<ProjectReport>,
}

/// Risk tally restricted to the three levels the report displays.
#[derive(Debug, Default, Clone, Copy)]
struct Tally {
    breaking: usize,
    at_risk: usize,
    safe: usize,
}

impl Tally {
    fn add(&mut self, update_type: UpdateType) {
        match update_type {
            UpdateType::Breaking => self.breaking += 1,
            UpdateType::AtRisk => self.at_risk += 1,
            UpdateType::Safe => self.safe += 1,
            UpdateType::UpToDate | UpdateType::Unknown => {}
        }
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

        for (project_index, (manifest_path, dependencies)) in groups.iter().enumerate() {
            // Sort rows by risk severity (most severe first), then by name, so a
            // project card leads with what needs attention.
            let mut dependencies = dependencies.clone();
            dependencies.sort_by(|a, b| {
                severity_rank(a.update_type)
                    .cmp(&severity_rank(b.update_type))
                    .then_with(|| a.name.cmp(&b.name))
            });

            let mut tally = Tally::default();
            let mut commands: Vec<String> = Vec::new();
            let mut rows = Vec::with_capacity(dependencies.len());

            for (dep_index, dependency) in dependencies.iter().enumerate() {
                tally.add(dependency.update_type);
                overall.add(dependency.update_type);

                let command = actionable_command(dependency);
                if !command.is_empty() {
                    commands.push(command.clone());
                }

                let latest = dependency.latest_version.clone();
                let ai_analysis_html = dependency
                    .ai_summary
                    .as_deref()
                    .map(markdown::render)
                    .unwrap_or_default();
                let has_ai_analysis = !ai_analysis_html.trim().is_empty();
                let insight = if has_ai_analysis {
                    row_insight(dependency.ai_summary.as_deref().unwrap_or_default())
                } else {
                    default_insight(dependency.update_type).to_string()
                };
                let is_breaking = dependency.update_type == UpdateType::Breaking;

                rows.push(DependencyReport {
                    name: dependency.name.clone(),
                    current_version: dependency.current_version.clone(),
                    latest_display: latest.clone().unwrap_or_else(|| "unknown".to_string()),
                    is_breaking,
                    status: stitch_status(dependency.update_type),
                    has_dot: has_risk_dot(dependency.update_type),
                    dot_class: dot_class(dependency.update_type),
                    risk_label: risk_label(dependency.update_type),
                    has_insight: !insight.is_empty(),
                    has_ai_analysis,
                    ai_cached: has_ai_analysis && dependency.ai_cached,
                    ai_analysis_html,
                    insight: insight.clone(),
                    changelog_url: changelog_url(dependency.ecosystem, &dependency.name),
                    has_command: !command.is_empty(),
                    command,
                    drawer_id: format!("drawer-{project_index}-{dep_index}"),
                    has_bump_badge: is_breaking && latest.is_some(),
                    bump_label: match &latest {
                        Some(target) => {
                            format!("Major Bump: {} → {}", dependency.current_version, target)
                        }
                        None => String::new(),
                    },
                    drawer_title: drawer_title(dependency.update_type),
                    drawer_desc: insight,
                });
            }

            let update_all_command = commands.join(" && ");
            projects.push(ProjectReport {
                name: project_name(manifest_path),
                manifest_path: manifest_path.clone(),
                ecosystem_label: ecosystem_label(dependencies[0].ecosystem),
                run_in: run_in(manifest_path),
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
            duration_display: match meta.scan_duration_ms {
                Some(ms) => format!("{ms} ms"),
                None => "—".to_string(),
            },
            breaking: overall.breaking,
            at_risk: overall.at_risk,
            safe: overall.safe,
            has_dependencies: !dependencies.is_empty(),
            projects,
        }
    }

    /// Render the report to a self-contained HTML document.
    pub fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}

/// Risk ordering for dependency rows: most severe first, unknown last.
fn severity_rank(update_type: UpdateType) -> u8 {
    match update_type {
        UpdateType::Breaking => 0,
        UpdateType::AtRisk => 1,
        UpdateType::Safe => 2,
        UpdateType::UpToDate => 3,
        UpdateType::Unknown => 4,
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

/// Directory the update command should be run in.
fn run_in(manifest_path: &str) -> String {
    Path::new(manifest_path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.display().to_string())
        .unwrap_or_else(|| ".".to_string())
}

/// Ecosystem pill label for the project header.
fn ecosystem_label(ecosystem: Ecosystem) -> &'static str {
    match ecosystem {
        Ecosystem::Cargo => "Cargo",
        Ecosystem::Npm => "npm",
        Ecosystem::Pypi => "PyPI",
        Ecosystem::Go => "Go",
        Ecosystem::Composer => "Composer",
    }
}

/// Filter tag used by the report's JavaScript (`data-status`).
fn stitch_status(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "breaking",
        UpdateType::AtRisk => "risk",
        UpdateType::Safe => "safe",
        UpdateType::UpToDate => "up-to-date",
        UpdateType::Unknown => "unknown",
    }
}

/// Human label for the risk column.
fn risk_label(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "major",
        UpdateType::AtRisk => "minor",
        UpdateType::Safe => "patch",
        UpdateType::UpToDate => "up-to-date",
        UpdateType::Unknown => "unknown",
    }
}

/// CSS class for the solid risk dot.
fn dot_class(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "dot-breaking",
        UpdateType::AtRisk => "dot-risk",
        UpdateType::Safe => "dot-safe",
        UpdateType::UpToDate | UpdateType::Unknown => "",
    }
}

/// Whether the update type carries a solid risk dot.
fn has_risk_dot(update_type: UpdateType) -> bool {
    matches!(
        update_type,
        UpdateType::Breaking | UpdateType::AtRisk | UpdateType::Safe
    )
}

/// Drawer summary title for an update type.
fn drawer_title(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "Major version bump — review before upgrading",
        UpdateType::AtRisk => "At-risk update — outside the declared range",
        UpdateType::Safe => "Safe update — inside the declared range",
        UpdateType::UpToDate => "Already up to date",
        UpdateType::Unknown => "No update data available",
    }
}

/// Fallback row insight when there is no AI summary.
fn default_insight(update_type: UpdateType) -> &'static str {
    match update_type {
        UpdateType::Breaking => "Major version bump — potential breaking API changes.",
        UpdateType::AtRisk => "Upgrade outside the declared range — verify compatibility.",
        UpdateType::Safe => "Patch or minor update inside the declared requirement.",
        UpdateType::UpToDate => "Declared version already matches the latest release.",
        UpdateType::Unknown => "Registry data unavailable; versions could not be compared.",
    }
}

/// Condense a Markdown analysis into one row-friendly line.
fn row_insight(markdown: &str) -> String {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("```")
            || trimmed.starts_with('|')
            || trimmed.starts_with("---")
            || trimmed.starts_with('<')
        {
            continue;
        }

        let text = strip_inline_markdown(trimmed);
        if !text.is_empty() {
            return truncate(&text, 160);
        }
    }

    "AI analysis available in the drawer.".to_string()
}

/// Remove the most common inline Markdown decorations from a row insight.
fn strip_inline_markdown(input: &str) -> String {
    let without_links = strip_links(input);
    without_links
        .trim_start_matches(['-', '*', '>', ' '])
        .replace(['*', '_', '`'], "")
        .trim()
        .to_string()
}

/// Replace `[label](url)` with `label`.
fn strip_links(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(open) = rest.find('[') {
        output.push_str(&rest[..open]);
        let after = &rest[open + 1..];

        if let Some(close) = after.find("](") {
            let label = &after[..close];
            let tail = &after[close + 2..];
            if let Some(end) = tail.find(')') {
                output.push_str(label);
                rest = &tail[end + 1..];
                continue;
            }
        }

        output.push('[');
        rest = after;
    }

    output.push_str(rest);
    output
}

/// Truncate on a character boundary, appending an ellipsis.
fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(limit.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

/// Standard published-versions page for a package.
fn changelog_url(ecosystem: Ecosystem, name: &str) -> String {
    match ecosystem {
        Ecosystem::Cargo => format!("https://crates.io/crates/{name}/versions"),
        Ecosystem::Npm => format!("https://www.npmjs.com/package/{name}?activeTab=versions"),
        Ecosystem::Pypi => format!("https://pypi.org/project/{name}/#history"),
        Ecosystem::Go => format!("https://pkg.go.dev/{name}?tab=versions"),
        Ecosystem::Composer => format!("https://packagist.org/packages/{name}#versions"),
    }
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
        Ecosystem::Pypi => format!("pip install {name}=={latest}"),
        Ecosystem::Go => format!("go get {name}@{latest}"),
        Ecosystem::Composer => format!("composer require {name}:{latest}"),
    }
}
