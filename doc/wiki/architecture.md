---
title: "RustyDeps — Architecture"
type: architecture
description: "Module breakdown and core DTOs for the RustyDeps CLI."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-15
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
> SemVer classifier, and HTML report engine below are implemented (Phases 1–4);
> the AI module is a Phase 3 scaffold with live providers still to come.

### CLI (`src/cli.rs`, `src/main.rs`)

Built with `clap` (derive pattern). The `scan` subcommand takes a root `path`
(default `.`), `--ai`, `--output`/`-o` (default `rustydeps-report.html`),
`--format html|json`, `--offline`/`--no-fetch`, and `--verbose`/`-v`. `main` is
`#[tokio::main]` and returns `anyhow::Result`; it initializes
`tracing-subscriber` (logs go to stderr so stdout stays clean for `--format
json`) and dispatches the subcommand. On text runs it prints the "Full Block
Solid" ASCII banner (Rust-orange logo, cyan version, grey tagline); the banner
is suppressed for `--format json` and its colours drop out when stdout is not a
terminal. `--offline` skips registry lookups. For `--format html` it writes the
self-contained report to `--output` and prints a `file://` URL to open it.

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
the scan. `CargoParser` (`Cargo.toml`) and `NpmParser` (`package.json`)
implement the trait first; later phases add `PyPI`, `Go`, and `Composer`.

- **`CargoParser`** decodes `[dependencies]`, `[dev-dependencies]`, and
  `[workspace.dependencies]`, resolving `dep = { workspace = true }`
  inheritance. Path- and git-only dependencies are skipped.
- **`NpmParser`** decodes `dependencies` and `devDependencies`; specs that do
  not point at `registry.npmjs.org` (`file:`, `link:`, `workspace:`, `npm:`
  aliases, git/http URLs) are skipped.

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

Response parsing is split from the HTTP calls into pure
`parse_crates_io_response` / `parse_npm_response` functions so it is unit
tested without network access. Client failures (transport error, 404,
unexpected body) degrade to "unknown latest" and are recorded on the
dependency; they never abort the whole scan. The AI module's changelog
metadata is still to come.

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
by project/manifest (project name from the manifest's directory) and
`render_html` renders the Askama template (`templates/report.html`) into a
single self-contained HTML file: all CSS and vanilla JavaScript are inline, so
it opens offline with no network access. Rows carry a copyable per-ecosystem
update command and each project exposes an "Update All" command. See
[[phase-4-html-report]]; the template design (palette, project-centric component
hierarchy, risk indicators, inline drawer, filter/search, empty state) is fixed
by [[html-report-spec]].

### AI module (`src/ai/`)

Optional and flag-gated. Phase 3 scaffolds it: `Summarizer` detects a provider
from `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or `RUSTYDEPS_AI_ENDPOINT`, and every
`AtRisk`/`Breaking` dependency receives a templated one-line summary. Live
provider calls that fetch the relevant changelog/release notes and summarize
breaking changes and migration notes are Phase 5. Any failure here is non-fatal
— the scan and report still complete.

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
`manifest_path`, `is_dev_dependency`, `ai_summary`); it is the input to the
Phase 4 report.

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
- [[okf-compliance]] — how this documentation is licensed and validated.
- [[index]] — knowledge-graph entrypoint.
