//! Prompt construction for the per-library changelog analysis.
//!
//! Every request analyzes exactly one upgrade tuple
//! `(ecosystem, package, current_version, target_version)`. The system prompt
//! fixes the Markdown structure the HTML report expects.

use crate::model::Ecosystem;
use crate::registry::changelog::{truncate_chars, MAX_CHANGELOG_CHARS};

/// System prompt shared by every per-package request.
pub const SYSTEM_PROMPT: &str = "\
You are RustyDeps' per-library release analyst. You analyze exactly one \
dependency upgrade at a time and return a concise, package-specific briefing \
for a developer deciding whether to upgrade.

Return GitHub-flavored Markdown only: no preamble, no closing offer to help, \
and no repetition of these instructions. Use exactly these four level-3 \
headings, in this order:

### 🚨 Breaking Changes & Deprecations
### 🛡️ Security & Vulnerability Context
### ⚡ Performance & Architectural Impact
### 💡 Actionable Migration Advice

Rules:
- Be specific to this package and this version jump. Do not give generic advice.
- Under each heading use short bullets. Put API names, flags, and snippets in \
inline code or fenced code blocks.
- Name concrete removed/renamed APIs, signature changes, MSRV or runtime \
changes, and resolved CVEs only when you are confident they apply to this \
release. Never invent a CVE identifier or a changelog entry.
- When official release notes are supplied, treat them as the primary source \
and prefer their facts over your internal knowledge.
- When there is nothing notable for a section, write one short sentence such \
as `No known security advisories for this release.`
- End the migration advice with the smallest concrete code or manifest change \
a developer should make.
- Keep the whole answer under 220 words.";

/// The upgrade being analyzed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeContext<'a> {
    /// Package ecosystem.
    pub ecosystem: Ecosystem,
    /// Package name.
    pub package: &'a str,
    /// Declared version or requirement.
    pub current_version: &'a str,
    /// Target published version.
    pub target_version: &'a str,
    /// Official release notes fetched from the package repository, when any.
    pub changelog: Option<&'a str>,
}

impl<'a> UpgradeContext<'a> {
    /// Build a context for one dependency upgrade.
    pub fn new(
        ecosystem: Ecosystem,
        package: &'a str,
        current_version: &'a str,
        target_version: &'a str,
    ) -> Self {
        Self {
            ecosystem,
            package,
            current_version,
            target_version,
            changelog: None,
        }
    }

    /// Attach official release notes to the context.
    pub fn with_changelog(mut self, changelog: Option<&'a str>) -> Self {
        self.changelog = changelog;
        self
    }
}

/// User prompt for one package upgrade.
///
/// When `context.changelog` holds text, a bounded excerpt is injected as the
/// authoritative source; otherwise the model is asked to use its internal
/// knowledge and to say when it is unsure.
pub fn user_prompt(context: &UpgradeContext<'_>) -> String {
    let mut prompt = format!(
        "Analyze this single dependency upgrade for the RustyDeps HTML report.\n\n\
         - Ecosystem: {ecosystem}\n\
         - Package: `{package}`\n\
         - Current version / requirement: `{current}`\n\
         - Target version: `{target}`\n",
        ecosystem = context.ecosystem,
        package = context.package,
        current = context.current_version,
        target = context.target_version,
    );

    match context
        .changelog
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        Some(changelog) => {
            let excerpt = truncate_chars(changelog, MAX_CHANGELOG_CHARS);
            prompt.push_str(&format!(
                "\nOfficial Release Notes / Changelog Excerpt:\n\
                 ---\n\
                 {excerpt}\n\
                 ---\n\n\
                 Use this excerpt as the primary source for what changed. Do not \
                 invent details that the excerpt does not contain; if it omits a \
                 section, say so briefly. Focus on the jump from `{current}` to \
                 `{target}`.",
                current = context.current_version,
                target = context.target_version,
            ));
        }
        None => prompt.push_str(&format!(
            "\nFocus on what changes between `{current}` and `{target}`. If the two \
             versions are close, prefer the concrete minor changes over general \
             ecosystem advice. If you are not certain about a specific release \
             note, say so briefly instead of guessing.",
            current = context.current_version,
            target = context.target_version,
        )),
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_fixes_the_report_sections() {
        for heading in [
            "### 🚨 Breaking Changes & Deprecations",
            "### 🛡️ Security & Vulnerability Context",
            "### ⚡ Performance & Architectural Impact",
            "### 💡 Actionable Migration Advice",
        ] {
            assert!(SYSTEM_PROMPT.contains(heading), "missing {heading}");
        }
        assert!(SYSTEM_PROMPT.contains("Never invent a CVE"));
    }

    #[test]
    fn user_prompt_names_the_package_and_both_versions() {
        let context = UpgradeContext::new(Ecosystem::Cargo, "tokio", "1.20.0", "2.0.0");
        let prompt = user_prompt(&context);

        assert!(prompt.contains("Ecosystem: cargo"));
        assert!(prompt.contains("Package: `tokio`"));
        assert!(prompt.contains("Current version / requirement: `1.20.0`"));
        assert!(prompt.contains("Target version: `2.0.0`"));
    }

    #[test]
    fn user_prompt_injects_official_release_notes() {
        let context = UpgradeContext::new(Ecosystem::Cargo, "tokio", "1.20.0", "2.0.0")
            .with_changelog(Some("### Breaking\n- `spawn` returns a handle."));
        let prompt = user_prompt(&context);

        assert!(prompt.contains("Official Release Notes / Changelog Excerpt:"));
        assert!(prompt.contains("---"));
        assert!(prompt.contains("`spawn` returns a handle."));
        assert!(prompt.contains("primary source"));
        assert!(!prompt.contains("If you are not certain about a specific release"));
    }

    #[test]
    fn user_prompt_bounds_the_changelog_excerpt() {
        let long = "x".repeat(MAX_CHANGELOG_CHARS * 3);
        let context = UpgradeContext::new(Ecosystem::Npm, "express", "^4.18.0", "5.0.0")
            .with_changelog(Some(&long));
        let prompt = user_prompt(&context);

        assert!(prompt.contains('…'));
        assert!(!prompt.contains(&long));
        let excerpt = prompt
            .split("Official Release Notes / Changelog Excerpt:\n---\n")
            .nth(1)
            .expect("excerpt")
            .split("\n---")
            .next()
            .expect("excerpt body");
        assert_eq!(excerpt.chars().count(), MAX_CHANGELOG_CHARS + 1);
    }

    #[test]
    fn user_prompt_falls_back_without_release_notes() {
        let context = UpgradeContext::new(Ecosystem::Pypi, "requests", "2.30.0", "2.31.0");
        let prompt = user_prompt(&context);

        assert!(!prompt.contains("Official Release Notes / Changelog Excerpt:"));
        assert!(prompt.contains("If you are not certain about a specific release"));
    }
}
