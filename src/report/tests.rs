//! Unit tests for the HTML report engine.

use std::path::PathBuf;

use askama::Template;

use super::*;

/// Build a classified dependency for tests.
fn dependency(
    ecosystem: Ecosystem,
    manifest: &str,
    name: &str,
    current: &str,
    latest: Option<&str>,
    update_type: UpdateType,
) -> ClassifiedDependency {
    ClassifiedDependency {
        name: name.to_string(),
        ecosystem,
        current_version: current.to_string(),
        latest_version: latest.map(str::to_string),
        update_type,
        manifest_path: PathBuf::from(manifest),
        is_dev_dependency: false,
        ai_summary: None,
        ai_cached: false,
    }
}

fn render(dependencies: &[ClassifiedDependency]) -> String {
    let data = ReportData::from_classified(dependencies, &ReportMeta::default());
    data.render().expect("render report")
}

#[test]
fn renders_a_self_contained_document() {
    let html = render(&[dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    )]);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<style>"));
    assert!(html.contains("<script>"));
    // No external resources: the report must open offline. Changelog anchors
    // may point at a package page, but nothing is fetched to render the page.
    assert!(!html.contains("src=\"http"), "found an external resource");
    assert!(!html.contains("<script src"));
    assert!(!html.contains("@import"));
    // The only <link> is the embedded data-URI favicon; no external stylesheets.
    assert_eq!(html.matches("<link ").count(), 1, "unexpected <link> tags");
    assert!(
        html.contains("<link rel=\"icon\" type=\"image/svg+xml\" href=\"data:image/svg+xml,"),
        "favicon must be an embedded SVG data URI"
    );
}

#[test]
fn renders_brand_logo_and_favicon() {
    let html = render(&[dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    )]);

    // Inline SVG brand mark precedes the wordmark; no external image request.
    assert!(html.contains("class=\"logo-icon\""));
    assert!(html.contains(">RustyDeps</span>"));
    assert!(html.contains("<link rel=\"icon\""));
    assert!(!html.contains("<img"), "branding must not use an <img> tag");
}

#[test]
fn escapes_untrusted_dependency_data() {
    let mut evil = dependency(
        Ecosystem::Npm,
        "web/package.json",
        "<script>alert(1)</script>",
        "^1.0.0",
        Some("1.1.0"),
        UpdateType::Safe,
    );
    evil.ai_summary = Some("<img src=x onerror=alert(1)>".to_string());

    let html = render(&[evil]);

    // Askama HTML-escapes the package name; raw HTML in the AI Markdown is
    // dropped by the renderer instead of being injected into the report.
    assert!(!html.contains("<script>alert"));
    assert!(
        html.contains("&lt;script&gt;alert(1)&lt;/script&gt;")
            || html.contains("&#60;script&#62;alert(1)&#60;/script&#62;")
    );
    assert!(!html.contains("<img src=x"));
    assert!(!html.contains("onerror"));
}

#[test]
fn groups_dependencies_by_manifest() {
    let dependencies = vec![
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "tokio",
            "0.2",
            Some("1.53.3"),
            UpdateType::Breaking,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "serde",
            "1.0",
            Some("1.0.229"),
            UpdateType::Safe,
        ),
        dependency(
            Ecosystem::Npm,
            "web-dashboard/package.json",
            "express",
            "^4.18.0",
            Some("5.3.0"),
            UpdateType::Breaking,
        ),
    ];

    let data = ReportData::from_classified(&dependencies, &ReportMeta::default());

    assert_eq!(data.project_count, 2);
    assert_eq!(data.dependency_count, 3);
    assert_eq!(data.breaking, 2);
    assert_eq!(data.safe, 1);

    let backend = data
        .projects
        .iter()
        .find(|project| project.name == "backend-api")
        .expect("backend-api group");
    assert_eq!(backend.dependencies.len(), 2);
    assert_eq!(backend.breaking, 1);
    assert_eq!(backend.ecosystem_label, "Cargo");
    assert_eq!(backend.run_in, "backend-api");

    let web = data
        .projects
        .iter()
        .find(|project| project.name == "web-dashboard")
        .expect("web-dashboard group");
    assert_eq!(web.ecosystem_label, "npm");
}

#[test]
fn builds_update_commands_per_ecosystem() {
    let dependencies = vec![
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "tokio",
            "0.2",
            Some("1.53.3"),
            UpdateType::Breaking,
        ),
        dependency(
            Ecosystem::Npm,
            "web-dashboard/package.json",
            "express",
            "^4.18.0",
            Some("5.3.0"),
            UpdateType::Breaking,
        ),
        dependency(
            Ecosystem::Npm,
            "web-dashboard/package.json",
            "left-pad",
            "1.3.0",
            Some("1.3.0"),
            UpdateType::UpToDate,
        ),
    ];

    let data = ReportData::from_classified(&dependencies, &ReportMeta::default());

    let cargo = &data.projects[0];
    assert!(cargo
        .dependencies
        .iter()
        .any(|dep| dep.command == "cargo update -p tokio --precise 1.53.3"));

    let npm = &data.projects[1];
    assert!(npm
        .dependencies
        .iter()
        .any(|dep| dep.command == "npm install express@5.3.0"));
    let up_to_date = npm
        .dependencies
        .iter()
        .find(|dep| dep.name == "left-pad")
        .expect("left-pad");
    assert!(!up_to_date.has_command);
    assert!(up_to_date.command.is_empty());
}

#[test]
fn combines_the_update_all_command() {
    let dependencies = vec![
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "anyhow",
            "1.0",
            Some("1.0.104"),
            UpdateType::Safe,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "tokio",
            "0.2",
            Some("1.53.3"),
            UpdateType::Breaking,
        ),
    ];

    let data = ReportData::from_classified(&dependencies, &ReportMeta::default());
    let project = &data.projects[0];

    assert!(project.has_update_all);
    // Rows are ordered most-severe-first, so the breaking `tokio` command comes
    // before the safe `anyhow` one.
    assert_eq!(
        project.update_all_command,
        "cargo update -p tokio --precise 1.53.3 && cargo update -p anyhow --precise 1.0.104"
    );
}

#[test]
fn sorts_dependencies_by_severity_then_name() {
    let dependencies = vec![
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "serde",
            "1.0",
            Some("1.0.219"),
            UpdateType::Safe,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "tokio",
            "0.2",
            Some("1.53.3"),
            UpdateType::Breaking,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "reqwest",
            "0.11",
            Some("0.13.5"),
            UpdateType::AtRisk,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "left-pad",
            "1.3.0",
            Some("1.3.0"),
            UpdateType::UpToDate,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "anyhow",
            "1.0",
            Some("2.0.0"),
            UpdateType::Breaking,
        ),
    ];

    let data = ReportData::from_classified(&dependencies, &ReportMeta::default());
    let names: Vec<&str> = data.projects[0]
        .dependencies
        .iter()
        .map(|dep| dep.name.as_str())
        .collect();

    // Breaking (alphabetical), then At-Risk, Safe, Up-To-Date.
    assert_eq!(names, ["anyhow", "tokio", "reqwest", "serde", "left-pad"]);
}

#[test]
fn builds_drawer_badges_and_ids() {
    let dependencies = vec![dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "1.35.1",
        Some("2.0.0"),
        UpdateType::Breaking,
    )];

    let data = ReportData::from_classified(&dependencies, &ReportMeta::default());
    let row = &data.projects[0].dependencies[0];

    assert!(row.has_bump_badge);
    assert_eq!(row.bump_label, "Major Bump: 1.35.1 → 2.0.0");
    assert_eq!(row.drawer_id, "drawer-0-0");
    assert_eq!(row.status, "breaking");
    assert_eq!(row.dot_class, "dot-breaking");
}

#[test]
fn renders_risk_indicators_and_filter_pills() {
    let html = render(&[
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "tokio",
            "0.2",
            Some("1.53.3"),
            UpdateType::Breaking,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "reqwest",
            "0.11",
            Some("0.13.5"),
            UpdateType::AtRisk,
        ),
        dependency(
            Ecosystem::Cargo,
            "backend-api/Cargo.toml",
            "serde",
            "1.0",
            Some("1.0.229"),
            UpdateType::Safe,
        ),
    ]);

    assert!(html.contains("dot-breaking"));
    assert!(html.contains("dot-risk"));
    assert!(html.contains("dot-safe"));
    assert!(html.contains("risk-label"));
    assert!(html.contains("data-status=\"breaking\""));
    assert!(html.contains("data-status=\"risk\""));
    // Filter tabs and the empty-state block are part of the design contract.
    assert!(html.contains("data-filter=\"all\""));
    assert!(html.contains("data-filter=\"breaking\""));
    assert!(html.contains("At-Risk"));
    assert!(html.contains("id=\"emptyState\""));
    assert!(html.contains("No dependencies match your query"));
    // The batch-update modal from the design is present too.
    assert!(html.contains("id=\"batchModal\""));
}

#[test]
fn renders_the_ai_tag_and_analysis_card() {
    let mut dep = dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    );
    dep.ai_summary = Some(
        "### 🚨 Breaking Changes & Deprecations\n\n\
         Use `tokio::runtime` instead.\n\n\
         ### 💡 Actionable Migration Advice\n\n\
         Update the manifest."
            .to_string(),
    );

    let html = render(&[dep]);

    assert!(html.contains("class=\"ai-prefix-icon\">AI<"));
    assert!(html.contains("🤖 AI Library Release Analysis"));
    assert!(html.contains("class=\"ai-analysis\""));
    assert!(html.contains("<h3>🚨 Breaking Changes &amp; Deprecations</h3>"));
    assert!(html.contains("Use <code>tokio::runtime</code> instead."));
    assert!(html.contains("View package changelog"));
    assert!(html.contains("https://crates.io/crates/tokio/versions"));
    assert!(!html.contains("⚡ Cached analysis"));
    assert!(!html.contains("AI analysis unavailable"));
}

#[test]
fn marks_analyses_served_from_the_cache() {
    let mut dep = dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    );
    dep.ai_summary =
        Some("### 🛡️ Security & Vulnerability Context\n\nNo known advisories.".to_string());
    dep.ai_cached = true;

    let data = ReportData::from_classified(std::slice::from_ref(&dep), &ReportMeta::default());
    assert!(data.projects[0].dependencies[0].ai_cached);

    let html = render(&[dep]);
    assert!(html.contains("⚡ Cached analysis"));
}

#[test]
fn renders_a_standard_changelog_link_without_ai() {
    let html = render(&[dependency(
        Ecosystem::Npm,
        "web-dashboard/package.json",
        "express",
        "^4.18.0",
        Some("5.3.0"),
        UpdateType::Breaking,
    )]);

    assert!(html.contains("AI analysis unavailable"));
    assert!(html.contains("View package changelog"));
    assert!(html.contains("https://www.npmjs.com/package/express?activeTab=versions"));
    assert!(!html.contains("class=\"ai-analysis\""));
}

#[test]
fn condenses_markdown_into_a_single_row_insight() {
    let mut dep = dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    );
    dep.ai_summary = Some(
        "### 🚨 Breaking Changes\n\n\
         - **Replace** `old` with [new](https://example.test)."
            .to_string(),
    );

    let data = ReportData::from_classified(&[dep], &ReportMeta::default());
    assert_eq!(
        data.projects[0].dependencies[0].insight,
        "Replace old with new."
    );
}

#[test]
fn reports_an_empty_scan() {
    let data = ReportData::from_classified(&[], &ReportMeta::default());

    assert_eq!(data.project_count, 0);
    assert_eq!(data.dependency_count, 0);
    assert!(!data.has_dependencies);

    let html = data.render().expect("render empty report");
    assert!(html.contains("No dependencies found"));
    assert!(html.contains("id=\"emptyState\" style=\"display: block;\""));
}

#[test]
fn surfaces_the_scan_duration() {
    let meta = ReportMeta {
        scan_duration_ms: Some(142),
    };
    let data = ReportData::from_classified(&[], &meta);

    assert_eq!(data.duration_display, "142 ms");
    assert!(data.render().expect("render").contains("142 ms"));
}

#[test]
fn maps_ecosystem_labels_and_update_commands() {
    let cases = [
        (
            Ecosystem::Pypi,
            "PyPI",
            "requests",
            "2.31.0",
            "pip install requests==2.31.0",
        ),
        (
            Ecosystem::Go,
            "Go",
            "github.com/gin-gonic/gin",
            "v1.10.0",
            "go get github.com/gin-gonic/gin@v1.10.0",
        ),
        (
            Ecosystem::Composer,
            "Composer",
            "monolog/monolog",
            "3.5.0",
            "composer require monolog/monolog:3.5.0",
        ),
    ];

    for (ecosystem, label, name, latest, expected) in cases {
        let dep = dependency(
            ecosystem,
            "proj/manifest",
            name,
            "1.0.0",
            Some(latest),
            UpdateType::Safe,
        );
        let data = ReportData::from_classified(&[dep], &ReportMeta::default());
        let project = &data.projects[0];
        assert_eq!(project.ecosystem_label, label);
        assert_eq!(project.dependencies[0].command, expected);
    }
}

#[test]
fn renders_project_cards_collapsed_by_default() {
    let html = render(&[dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    )]);

    // The card opens compact; the global toggle is synced to "Expand All" by
    // `updateCollapseBtnText()` on load.
    assert!(html.contains("class=\"project-card collapsed\""));
    assert!(html.contains("updateCollapseBtnText();"));
}

#[test]
fn keeps_collapsed_header_clean_and_moves_path_and_action_into_the_body() {
    let html = render(&[dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "1.53.0",
        Some("1.53.3"),
        UpdateType::Safe,
    )]);

    // Slice the first card's collapsed header (from the header element up to the
    // expanded body header) and check it stays compact.
    let header_start = html
        .find("class=\"project-header\"")
        .expect("project header markup");
    let body_header = html
        .find("class=\"project-deps-header\"")
        .expect("expanded body header");
    assert!(body_header > header_start);
    let collapsed_header = &html[header_start..body_header];

    assert!(
        !collapsed_header.contains("backend-api/Cargo.toml"),
        "collapsed header must not show the filesystem path"
    );
    assert!(
        !collapsed_header.contains("update-all-btn"),
        "Update All must not be in the collapsed header"
    );

    // Both the path and the batch action live in the expanded body header.
    let path_at = html.find("backend-api/Cargo.toml").expect("manifest path");
    assert!(path_at > body_header, "path must be in the body header");
    assert!(html.contains("class=\"update-all-btn\""));
}
