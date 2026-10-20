---
title: "RustyDeps — Architecture"
type: architecture
description: "Module breakdown and core DTOs for the RustyDeps CLI."
status: active
version: 0.3.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-20
tags: [architecture, rust, cli, dto]
---

# Architecture

RustyDeps is organized as a linear pipeline: walk the filesystem, parse
manifests, query registries, classify upgrades, and render a report. Each stage
is an isolated, testable module. See [[roadmap]] for when each stage lands and
[[index]] for the wider catalogue.

## Pipeline overview

```
CLI (clap)
  └─ Filesystem Walk (ignore)
       └─ Manifest Parsers (PackageParser trait)
            └─ Registry Clients (reqwest + tokio)
                 └─ SemVer Classification
                      └─ Report Generator (HTML)
                           └─ AI Changelog Summarizer (optional)
```

## Modules

> Status: the CLI, filesystem walk, manifest parsers, registry clients,
> SemVer classifier, HTML report engine, and the per-library AI summarizer
> below are implemented (Phases 1–6).

### CLI (`src/cli.rs`, `src/main.rs`)

Built with `clap` (derive pattern). The `scan` subcommand takes a root `path`
(default `.`), `--ai`, `--no-ai`, `--ai-model <MODEL>`, `--output`/`-o` (default
`rustydeps-report.html`), `--format html|json`, `--offline`/`--no-fetch`,
`--no-open`, and `--verbose`/`-v`.
`main` is `#[tokio::main]` and returns `anyhow::Result`; it initializes
`tracing-subscriber` (logs go to stderr so stdout stays clean for `--format
json`) and dispatches the subcommand. On text runs it prints the "Full Block
Solid" ASCII banner (Rust-orange logo, cyan version, grey tagline); the banner
is suppressed for `--format json` and its colours drop out when stdout is not a
terminal. `--offline` skips registry lookups. For `--format html` it writes the
self-contained report to `--output`, prints a `file://` URL, and (unless
`--no-open`) launches it in the default browser via the `open` crate — a failure
to open is logged as a warning and never fails the scan.

Scan progress is reported by `src/progress.rs` (`ScanProgress`, a wrapper over
`indicatif`): a spinner covers manifest discovery and registry resolution, and a
`pos/len` counter covers per-repository reporting, where `report_repositories`
prints one `✔ Scanned '<relative path>': N dependencies (<ecosystem>)` line per
manifest. All bars share one `indicatif::MultiProgress`; status lines and tracing
records are written with the bars *suspended* (`ScanProgress::println` and the
`ProgressWriter` `MakeWriter`) so they never leave orphan bar frames. Bars draw
to stderr and are hidden whenever stderr is not an interactive terminal;
`--format json` additionally suppresses the banner and the status lines so
stdout stays a single JSON document. See [[cli-scanning-ux]] and
[[report-ux-refinements]].

### Filesystem walk (`src/scanner.rs`)

Uses `ignore::WalkBuilder` so `.gitignore`, `.ignore`, and `.git/info/exclude`
rules are honoured and hidden entries are skipped. `require_git(false)` makes
ignore files apply even when the root is not a git repository, and the
`node_modules`, `target`, and `.git` directories are pruned explicitly.
`scan_directory` returns a path-sorted `Vec<DiscoveredManifest>`. *Implemented in
Phase 1.*

```rust
pub enum ManifestType {
    CargoToml,      // Cargo.toml
    PackageJson,    // package.json
    GoMod,          // go.mod
    RequirementsTxt,// requirements.txt
    PyprojectToml,  // pyproject.toml
    ComposerJson,   // composer.json
}

pub struct DiscoveredManifest {
    pub path: PathBuf,
    pub manifest_type: ManifestType,
}
```

### Parser trait (`src/parser/`)

The `PackageParser` trait is the seam that keeps ecosystem support open for
extension without touching the scanner:

```rust
pub trait PackageParser: Send + Sync {
    fn ecosystem(&self) -> Ecosystem;
    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool;
    fn parse(&self, manifest: &DiscoveredManifest)
        -> Result<Vec<ParsedDependency>, ParserError>;
}
```

`ParsedDependency` carries `name`, `current_version`, `ecosystem`,
`manifest_path`, and `is_dev_dependency`. A driver, `parse_manifests`, routes
each discovered manifest to the first parser that claims it; a manifest a
parser claims but cannot decode is recorded and skipped rather than aborting
the scan. Five parsers implement the trait — Cargo and npm (Phase 2), then
PyPI, Go and Composer (Phase 5).

- **`CargoParser`** decodes `[dependencies]`, `[dev-dependencies]`, and
  `[workspace.dependencies]`, resolving `dep = { workspace = true }`
  inheritance. Path- and git-only dependencies are skipped.
- **`NpmParser`** decodes `dependencies` and `devDependencies`; specs that do
  not point at `registry.npmjs.org` (`file:`, `link:`, `workspace:`, `npm:`
  aliases, git/http URLs) are skipped.
- **`PypiParser`** decodes `pyproject.toml` (PEP 621 `[project.dependencies]`
  and Poetry `[tool.poetry.dependencies]`) and `requirements.txt`; PEP 508
  specifiers are reduced to a name plus a specifier, and the Poetry `python`
  entry is skipped.
- **`GoParser`** decodes `go.mod` direct `require` directives, keeping the `v`
  prefix and skipping `// indirect`, `replace`, and `exclude`.
- **`ComposerParser`** decodes `composer.json` `require`/`require-dev`, skipping
  PHP platform pseudo-packages (`php`, `ext-*`, `lib-*`, `composer-*`, `hhvm`).

### Registry clients (`src/registry/`)

One async client per registry, built on `reqwest` and driven by a `tokio`
runtime. `RegistryClient` is a native `async fn` trait (no `async-trait`
dependency, no `dyn` dispatch) with `fetch_latest_version(name)`; the helper
`fetch_latest_versions` keeps at most `DEFAULT_CONCURRENCY` (10) requests in
flight using `futures` `buffer_unordered`.

- **`CratesIoClient`** queries `/api/v1/crates/{name}` with the descriptive
  `User-Agent` crates.io requires and returns the latest *stable* version
  (`crate.max_stable_version`, else the highest non-prerelease non-yanked
  `versions` entry, else `max_version`).
- **`NpmRegistryClient`** queries `registry.npmjs.org/{name}` (scoped names
  percent-encoded) and reads `dist-tags.latest`.
- **`PypiClient`** queries `pypi.org/pypi/{name}/json` and returns the highest
  stable release (skipping fully yanked files; falls back to `info.version`).
- **`GoRegistryClient`** queries `proxy.golang.org/{module}/@v/list` (uppercase
  module-path letters encoded as `!x`) and returns the highest stable tag.
- **`ComposerClient`** queries `repo.packagist.org/p2/{name}.json` and returns
  the highest stable tag, skipping `dev-*` and prereleases.

Response parsing is split from the HTTP calls into pure `parse_*` functions
(one per client) so every client is unit tested without network access. Client
failures (transport error, 404, unexpected body) degrade to "unknown latest"
and are recorded on the dependency; they never abort the whole scan.

### SemVer classification (`src/classifier/`)

`classify_update(current, latest) -> UpdateType` is a pure function — the
highest-value unit-test target. It parses the declared value leniently (`^1.2.3`,
`~1.2`, `1`) and the registry version with the `semver` crate, then buckets the
jump: major difference → `Breaking`; minor difference inside the declared
requirement → `Safe`, outside it → `AtRisk`; patch-only → `Safe`; equal →
`UpToDate`; unparseable, a downgrade, or a stable version facing a prerelease →
`Unknown`. `latest_stable` keeps the upgrade target on the newest stable
release. `classify` builds the `ClassifiedDependency` result set. See
[[phase-3-semver-classifier]].

### Report generator (`src/report/`)

Implemented in Phase 4. `ReportData::from_classified` groups the classified set
by project/manifest (project name from the manifest's directory), sorts each
group's rows by risk severity then name, and `render_html` renders the Askama
template (`templates/report.html` — the Google Stitch UI v4 export with its mock
rows replaced by loops) into a single self-contained HTML file: all CSS and
vanilla JavaScript are inline, so it opens offline with no network access. Rows
carry a copyable per-ecosystem update command and each project exposes an
"Update All" command in its expanded body. Cards render collapsed by default.
When an AI analysis exists, the drawer renders the Markdown briefing in a
**🤖 AI Library Release Analysis** card, with a `⚡ Cached analysis` badge on a
cache hit; otherwise it shows the generic insight, an "AI analysis unavailable"
note, and a standard package-changelog anchor. Anchors are the only external
URLs in the document; CSS, JavaScript, the logo, and the favicon remain inline.
See [[phase-4-html-report]]; the template design (palette, project-centric
component hierarchy, risk indicators, inline drawer, filter/search, empty state)
is fixed by [[html-report-spec]]. The card disclosure and row ordering are
refined by [[report-ux-refinements]], and the AI card by
[[ai-changelog-summarizer]].

The report carries branding and a readable type scale: an inline SVG logo in the
header and an embedded data-URI SVG favicon (both self-contained — no external
image request), on a 12/13/14/16px scale. See [[ui-font-branding]].

### AI module (`src/ai/`, `src/config/`)

Optional and BYOK-configured (Milestone 0.3.0, hardened by
[[ai-provider-compat]]). `src/config/mod.rs` resolves settings in layers: CLI
(`--ai`, `--no-ai`, `--ai-model`), `RUSTYDEPS_AI_*` environment variables,
`~/.config/rustydeps/config.json` (XDG on Unix, `%APPDATA%` on Windows), then a
project `.rustydeps.json` fallback when the user file is absent, then provider
defaults. Malformed files or values disable AI for the run instead of failing
the scan.

`AiProvider` covers OpenAI (`gpt-4o-mini` at `https://api.openai.com/v1`), Gemini
(`gemini-3.5-flash` at the OpenAI-compatible
`https://generativelanguage.googleapis.com/v1beta/openai`), OpenRouter
(`google/gemini-2.5-flash` at `https://openrouter.ai/api/v1`), Ollama
(`llama3.1` at `http://localhost:11434/v1`, no key), and a custom proxy that
requires an explicit endpoint and model and accepts an optional key. OpenAI,
Gemini, and OpenRouter require keys; keys fall back to `OPENAI_API_KEY`,
`GEMINI_API_KEY`/`GOOGLE_API_KEY`, and `OPENROUTER_API_KEY` respectively.

`Summarizer` (`src/ai/summary.rs`) analyzes only `Safe`/`AtRisk`/`Breaking` rows
with a resolved target. It checks `SummaryCache` first, then calls `AiClient`
(`src/ai/client.rs`), an async `reqwest` chat-completions client with a
10-second timeout, bounded to four concurrent packages. `chat_endpoint`
normalizes each provider base URL — bare origins, `/v1` or `/v1beta` roots,
trailing slashes, full `/chat/completions` URLs, and query strings all resolve to
one valid endpoint — and OpenRouter requests add `HTTP-Referer`/`X-Title`
attribution headers. `src/ai/prompt.rs` fixes the four-section Markdown
contract; one request is made per
`(ecosystem, package, current_version, target_version)`.

`SummaryCache` (`src/ai/cache.rs`) stores one JSON entry per target version at
`~/.cache/rustydeps/changelogs/{ecosystem}_{package}_{target_version}.json`, so
projects sharing a dependency reuse the analysis for free. Cache entries are
validated against ecosystem, package, and target before use; `cache_enabled:
false` skips disk entirely.

The report converts the Markdown with `src/report/markdown.rs`
(`pulldown-cmark`, raw HTML events dropped) and inserts it with Askama's
`|safe` filter. Every failure — disabled, missing key, timeout, HTTP error,
empty response, cache write error — is logged and leaves `ai_summary` as
`None`; the scan and report still complete. HTTP failures log the resolved
request URL, status, and a single-line body excerpt. See
[[ai-changelog-summarizer]] and [[ai-provider-compat]].

## Core DTOs

`src/model.rs` holds the shared enums — `Ecosystem` (Phase 2) and `UpdateType`
(Phase 3):

```rust
pub enum Ecosystem { Cargo, Npm /*, PyPI, Go, Composer */ }

pub enum UpdateType {
    Safe,     // patch/minor bump inside the declared requirement
    AtRisk,   // bump outside the declared requirement
    Breaking, // major jump
    UpToDate, // declared version already latest
    Unknown,  // unparseable, downgrade, or missing registry data
}
```

The classified result set is `ClassifiedDependency` in `src/classifier/mod.rs`
(`name`, `ecosystem`, `current_version`, `latest_version`, `update_type`,
`manifest_path`, `is_dev_dependency`, `ai_summary`, `ai_cached`); it is the input
to the Phase 4 report. `ai_summary` holds the Markdown briefing (or `None`), and
`ai_cached` marks a cache hit for the report's ⚡ badge.

## Quality bars

- `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` clean.
- `thiserror` in modules, `anyhow` at the binary boundary.
- No `unwrap()` in production paths.
- Unit tests in every module; classifier and parsers first.

## Related

- [[roadmap]] — phase plan that builds these modules in order.
- [[phase-2-parsers-registry]] — the implemented parser and registry clients.
- [[phase-3-semver-classifier]] — the classifier, AI scaffold, and CLI polish.
- [[phase-4-html-report]] — the self-contained HTML report engine.
- [[phase-5-ecosystems]] — the PyPI/Go/Composer parsers and clients.
- [[ai-changelog-summarizer]] — the per-library summarizer, config, and cache.
- [[okf-compliance]] — how this documentation is licensed and validated.
- [[index]] — knowledge-graph entrypoint.
