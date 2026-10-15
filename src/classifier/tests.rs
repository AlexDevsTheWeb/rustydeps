//! Unit tests for the SemVer classifier.

use std::path::PathBuf;

use super::*;
use crate::parser::ParsedDependency;

#[test]
fn patch_bump_is_safe() {
    assert_eq!(classify_update("1.0.0", "1.0.1"), UpdateType::Safe);
}

#[test]
fn minor_bump_is_safe() {
    assert_eq!(classify_update("1.0.0", "1.1.0"), UpdateType::Safe);
}

#[test]
fn major_bump_is_breaking() {
    assert_eq!(classify_update("1.35.1", "2.0.0"), UpdateType::Breaking);
}

#[test]
fn equal_versions_are_up_to_date() {
    assert_eq!(classify_update("1.0.0", "1.0.0"), UpdateType::UpToDate);
}

#[test]
fn minor_bump_outside_a_tilde_requirement_is_at_risk() {
    assert_eq!(classify_update("~1.0.0", "1.1.0"), UpdateType::AtRisk);
}

#[test]
fn patch_bump_inside_a_tilde_requirement_is_safe() {
    assert_eq!(classify_update("~5.4.0", "5.4.9"), UpdateType::Safe);
}

#[test]
fn caret_requirement_allows_minor_bumps() {
    assert_eq!(classify_update("^4.18.0", "4.19.2"), UpdateType::Safe);
    assert_eq!(classify_update("^4.18.0", "5.0.0"), UpdateType::Breaking);
}

#[test]
fn prerelease_transitions_are_handled() {
    // Prerelease graduating to its release stays within the same version.
    assert_eq!(classify_update("1.0.0-rc.1", "1.0.0"), UpdateType::Safe);
    // A stable version is never "upgraded" to a prerelease.
    assert_eq!(classify_update("1.0.0", "1.1.0-rc.1"), UpdateType::Unknown);
    // Prerelease to a later prerelease is comparable.
    assert_eq!(
        classify_update("2.0.0-alpha.1", "2.0.0-beta.1"),
        UpdateType::Safe
    );
}

#[test]
fn partial_versions_are_normalized() {
    assert_eq!(classify_update("1.0", "1.0.5"), UpdateType::Safe);
    assert_eq!(classify_update("1", "1.1.0"), UpdateType::Safe);
}

#[test]
fn invalid_versions_degrade_to_unknown() {
    assert_eq!(
        classify_update("not-a-version", "1.2.3"),
        UpdateType::Unknown
    );
    assert_eq!(classify_update("1.0.0", "garbage"), UpdateType::Unknown);
    assert_eq!(classify_update("*", "1.2.3"), UpdateType::Unknown);
    assert_eq!(classify_update("latest", "1.2.3"), UpdateType::Unknown);
}

#[test]
fn downgrades_are_unknown() {
    assert_eq!(classify_update("2.0.0", "1.9.0"), UpdateType::Unknown);
}

#[test]
fn latest_stable_ignores_prereleases_and_invalid_entries() {
    let picked = latest_stable(["1.0.0", "1.1.0", "2.0.0-rc.1", "1.0.5", "garbage"]);
    assert_eq!(picked.as_deref(), Some("1.1.0"));

    assert_eq!(latest_stable(["1.0.0-rc.1", "2.0.0-beta.1"]), None);
    assert_eq!(latest_stable(std::iter::empty::<&str>()), None);
}

#[test]
fn classify_builds_the_dto() {
    let dependency = ParsedDependency {
        name: "serde".to_string(),
        current_version: "^1.0.0".to_string(),
        ecosystem: Ecosystem::Cargo,
        manifest_path: PathBuf::from("Cargo.toml"),
        is_dev_dependency: false,
    };

    let classified = classify(&dependency, Some("1.0.229"));
    assert_eq!(classified.name, "serde");
    assert_eq!(classified.update_type, UpdateType::Safe);
    assert_eq!(classified.latest_version.as_deref(), Some("1.0.229"));
    assert_eq!(classified.ai_summary, None);
    assert!(!classified.is_dev_dependency);

    let unknown = classify(&dependency, None);
    assert_eq!(unknown.update_type, UpdateType::Unknown);
    assert_eq!(unknown.latest_version, None);
}
