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
    // No external resources: the report must open offline.
    assert!(!html.contains("http://"), "found an http:// reference");
    assert!(!html.contains("https://"), "found an https:// reference");
    assert!(!html.contains("<link "));
    assert!(!html.contains("<script src"));
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

    // Askama HTML-escapes automatically; accept either named or numeric entities.
    assert!(!html.contains("<script>alert"));
    assert!(
        html.contains("&lt;script&gt;alert(1)&lt;/script&gt;")
            || html.contains("&#60;script&#62;alert(1)&#60;/script&#62;")
    );
    assert!(!html.contains("<img src=x"));
    assert!(html.contains("&lt;img") || html.contains("&#60;img"));
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
    assert_eq!(backend.total, 2);
    assert_eq!(backend.breaking, 1);
    assert_eq!(backend.ecosystems, vec!["CARGO"]);

    let web = data
        .projects
        .iter()
        .find(|project| project.name == "web-dashboard")
        .expect("web-dashboard group");
    assert_eq!(web.ecosystems, vec!["NPM"]);
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
        .any(|dep| dep.update_command == "cargo update -p tokio --precise 1.53.3"));

    let npm = &data.projects[1];
    assert!(npm
        .dependencies
        .iter()
        .any(|dep| dep.update_command == "npm install express@5.3.0"));
    let up_to_date = npm
        .dependencies
        .iter()
        .find(|dep| dep.name == "left-pad")
        .expect("left-pad");
    assert!(!up_to_date.has_update_command);
    assert!(up_to_date.update_command.is_empty());
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
    assert_eq!(
        project.update_all_command,
        "cargo update -p anyhow --precise 1.0.104 && cargo update -p tokio --precise 1.53.3"
    );
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

    assert!(html.contains("risk-breaking"));
    assert!(html.contains("risk-at-risk"));
    assert!(html.contains("risk-safe"));
    assert!(html.contains(">major<"));
    assert!(html.contains(">minor<"));
    assert!(html.contains(">patch<"));
    // Filter pills and the empty-state block are part of the design contract.
    assert!(html.contains("data-risk=\"all\""));
    assert!(html.contains("At-Risk"));
    assert!(html.contains("id=\"empty-state\""));
    assert!(html.contains("No dependencies match your filter"));
}

#[test]
fn renders_the_ai_tag_when_a_summary_is_present() {
    let mut dep = dependency(
        Ecosystem::Cargo,
        "backend-api/Cargo.toml",
        "tokio",
        "0.2",
        Some("1.53.3"),
        UpdateType::Breaking,
    );
    dep.ai_summary = Some("Major version bump — potential breaking API changes.".to_string());

    let html = render(&[dep]);

    assert!(html.contains("class=\"ai-tag\">AI<"));
    assert!(html.contains("potential breaking API changes"));
}

#[test]
fn reports_an_empty_scan() {
    let data = ReportData::from_classified(&[], &ReportMeta::default());

    assert_eq!(data.project_count, 0);
    assert_eq!(data.dependency_count, 0);
    assert!(!data.has_dependencies);

    let html = data.render().expect("render empty report");
    assert!(html.contains("No dependencies found"));
    // With nothing to hide, the filter empty state is not shown.
    assert!(!html.contains("id=\"empty-state\" hidden"));
}

#[test]
fn surfaces_scan_metadata() {
    let meta = ReportMeta {
        scanned_root: Some("/work/monorepo".to_string()),
        scan_duration_ms: Some(142),
    };
    let data = ReportData::from_classified(&[], &meta);

    assert_eq!(data.scanned_root, "/work/monorepo");
    assert_eq!(data.duration_display, "142 ms");

    let html = data.render().expect("render");
    assert!(html.contains("/work/monorepo"));
    assert!(html.contains("142 ms"));
}
