# Opencode Prompt 15 — Update GitHub Pages Landing Page for v0.3.1

You are Opencode, an expert Rust developer and frontend designer.
Your task is to update the **RustyDeps single-page landing site** (`https://alexdevstheweb.github.io/rustydeps/`) so that it accurately reflects version **v0.3.1** and all major features introduced up to this release.

---

## 🎯 Context & Goal

The GitHub release for `rustydeps` is currently at **v0.3.1**, but the static landing page hosted on GitHub Pages (located in `docs/index.html` or the relevant site directory in the repository) still shows an outdated version number and lacks the latest major features (such as per-library AI changelog summaries, Ollama local AI support, BYOK config, and cache management).

You need to update the landing page HTML/assets to reflect **v0.3.1**, ensure all version badges, code snippets, installation commands, and feature descriptions are completely up to date.

---

## 📋 Detailed Implementation Steps

### 1. Locate and Audit Landing Page Source Files
* Check the repository for the landing page files (e.g., `docs/index.html`, `site/`, or relevant landing page template).
* Audit all version references, hero section badges, installation code snippets, and feature highlights.

### 2. Update Version Badges & Hero Section
* Change the release version badge/text to **`v0.3.1`** (e.g., `Latest Release: v0.3.1`).
* Update the quick installation command snippets:
  * Cargo: `cargo install --git https://github.com/AlexDevsTheWeb/rustydeps --tag v0.3.1`
  * Docker: `docker pull ghcr.io/alexdevstheweb/rustydeps:v0.3.1`

### 3. Highlight Key Features (v0.3.0 & v0.3.1 Updates)
Ensure the landing page prominently features the core capabilities of RustyDeps v0.3.1:
* 🚀 **Multi-Ecosystem Local Scanning:** Parallel inspection for Cargo (`Cargo.toml`), npm (`package.json`), PyPI (`requirements.txt`), Go (`go.mod`), and Composer (`composer.json`).
* 🤖 **Per-Library AI Summarizer:** Granular release briefings per outdated dependency (Breaking Changes, Security/CVE Context, Performance Impact, and Migration Tips).
* 🦙 **100% Free & Local AI (Ollama):** Zero-cost, privacy-first offline summaries via local Ollama (`llama3.2:3b`), as well as BYOK for Google Gemini, OpenAI, and OpenRouter.
* ⚡ **Disk Caching & Circuit Breaker:** Zero-cost repeat scans via `~/.cache/rustydeps/changelogs/`, `rustydeps cache clear`, `--no-cache`, and automatic rate-limit protection.
* 🎨 **Interactive HTML Dashboards:** Modern dark-theme report with copy-ready upgrade commands, model provenance badges, and expandable dependency drawers.

### 4. Code & CLI Example Showcase
* Ensure terminal preview code blocks and screenshots/diagrams accurately demonstrate:
  * Scanning commands (`rustydeps scan .`)
  * Cache control flags (`rustydeps cache clear`, `rustydeps scan --no-cache`)
  * CLI progress spinners and tally output.

### 5. Verification & Quality Gates
* Validate that `index.html` (and associated CSS/JS assets) renders cleanly in local browser.
* Ensure all links (GitHub release, Docker GHCR, docs) point to `v0.3.1` assets.
* Verify there are no broken links, invalid HTML markup, or outdated version strings.

---

## 🔀 Git & Pull Request

* Create feature branch `fix/landing-page-v0.3.1` off `development`.
* Commit changes with message: `docs(site): update landing page to v0.3.1 features and badges`.
* Open a Pull Request targeting `development`.
