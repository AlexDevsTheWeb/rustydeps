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

> Status: the CLI, filesystem walk, manifest parsers, and registry clients
> below are implemented (Phases 1–2). The classification, report, and AI
> modules remain planned.

### CLI (`src/cli.rs`, `src/main.rs`)

Built with `clap` (derive pattern). The `scan` subcommand takes a root `path`
(default `.`), `--ai`, `--output`/`-o` (default `report.html`), `--format
html|json`, and `--verbose`/`-v`. `main` is `#[tokio::main]` and returns
`anyhow::Result`; it initializes `tracing-subscriber` (logs go to stderr so
stdout stays clean for `--format json`) and dispatches the subcommand. *Implemented
in Phase 1; `--output`/`--ai` are accepted but not yet wired to a report engine.*

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
  `User-Agent` crates.io requires and reads `crate.max_version`.
- **`NpmRegistryClient`** queries `registry.npmjs.org/{name}` (scoped names
  percent-encoded) and reads `dist-tags.latest`.

Response parsing is split from the HTTP calls into pure
`parse_crates_io_response` / `parse_npm_response` functions so it is unit
tested without network access. Client failures (transport error, 404,
unexpected body) degrade to "unknown latest" and are recorded on the
dependency; they never abort the whole scan. The AI module's changelog
metadata is still to come.

### SemVer classification

A pure function of (declared requirement, current version, latest version) →
`UpdateType`. Keeping it pure makes it the highest-value unit-test target.
Versions are parsed with the `semver` crate; a manifest's declared range governs
what counts as inside or outside the allowed upgrade window.

### Report generator (`src/report/`)

Renders a single self-contained, interactive HTML file from the in-memory
result set — filter and sort by ecosystem, severity, and repository. Requires no
network access to view and opens straight from disk.

The template design (palette, project-centric component hierarchy, risk
indicators, inline drawer, filter/search, empty state) is fixed by
[[html-report-spec]]; treat that page as the report generator's design contract.

### AI module (`src/ai/`)

Optional and flag-gated. Given a dependency and its version jump, fetches the
relevant changelog/release notes and summarizes breaking changes and migration
notes. Any failure here is non-fatal — the scan and report still complete.

## Core DTOs (`src/model.rs`)

`Ecosystem` is implemented (Phase 2); the unified `Dependency` and `UpdateType`
DTOs arrive in Phase 3.

```rust
pub enum Ecosystem { Cargo, Npm /*, PyPI, Go, Composer */ }

pub struct Dependency {
    pub name: String,
    pub ecosystem: Ecosystem,
    pub current: String,
    pub latest: Option<String>,
    pub manifest_path: PathBuf,
    pub repository: PathBuf,
}

pub enum UpdateType {
    Safe,     // patch/minor within the declared range
    AtRisk,   // outside the range, not obviously breaking
    Breaking, // major jump or a violated requirement
}
```

## Quality bars

- `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` clean.
- `thiserror` in modules, `anyhow` at the binary boundary.
- No `unwrap()` in production paths.
- Unit tests in every module; classifier and parsers first.

## Related

- [[roadmap]] — phase plan that builds these modules in order.
- [[phase-2-parsers-registry]] — the implemented parser and registry clients.
- [[okf-compliance]] — how this documentation is licensed and validated.
- [[index]] — knowledge-graph entrypoint.
