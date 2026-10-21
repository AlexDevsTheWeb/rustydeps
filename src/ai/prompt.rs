//! Prompt construction for the per-library changelog analysis.
//!
//! Every request analyzes exactly one upgrade tuple
//! `(ecosystem, package, current_version, target_version)`. The system prompt
//! fixes the Markdown structure the HTML report expects.

use crate::model::Ecosystem;

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
        }
    }
}

/// User prompt for one package upgrade.
pub fn user_prompt(context: &UpgradeContext<'_>) -> String {
    format!(
        "Analyze this single dependency upgrade for the RustyDeps HTML report.\n\n\
         - Ecosystem: {ecosystem}\n\
         - Package: `{package}`\n\
         - Current version / requirement: `{current}`\n\
         - Target version: `{target}`\n\n\
         Focus on what changes between `{current}` and `{target}`. If the two \
         versions are close, prefer the concrete minor changes over general \
         ecosystem advice. If you are not certain about a specific release \
         note, say so briefly instead of guessing.",
        ecosystem = context.ecosystem,
        package = context.package,
        current = context.current_version,
        target = context.target_version,
    )
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
}
