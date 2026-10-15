---
title: "Phase 2 — Manifest Parsers & Registry Clients"
type: feature
description: "Parses Cargo.toml and package.json into declared dependencies and resolves their latest versions from crates.io and npm."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [feature, phase-2, parser, registry, rust]
---

# Phase 2 — Manifest parsers & registry clients

## Motivation

Phase 1 could only walk the filesystem and list manifest *paths*; it knew
nothing about the dependencies inside them. Before RustyDeps can classify
upgrades or render a report, it needs to turn a manifest into a list of
declared dependencies and discover what version each dependency has most
recently published. Phase 2 builds exactly that data layer: a trait-based
parser subsystem and bounded-concurrency registry clients for crates.io and
the npm registry.

## What shipped

- **Parser subsystem (`src/parser/`)** — a `PackageParser` trait with
  `CargoParser` and `NpmParser`, plus the `ParsedDependency` DTO and a
  `parse_manifests` driver that tolerates per-manifest failures.
- **Registry subsystem (`src/registry/`)** — a `RegistryClient` trait with
  `CratesIoClient` and `NpmRegistryClient`, and a `fetch_latest_versions`
  helper that bounds in-flight requests.
- **Shared model (`src/model.rs`)** — the `Ecosystem` enum, used by both the
  parsers and the clients.
- **CLI wiring (`src/main.rs`)** — `scan` now parses and resolves, printing a
  table or JSON summary of current versus latest versions.

## Components

### `PackageParser` (`src/parser/mod.rs`)

```rust
pub trait PackageParser: Send + Sync {
    fn ecosystem(&self) -> Ecosystem;
    fn can_parse(&self, manifest: &DiscoveredManifest) -> bool;
    fn parse(&self, manifest: &DiscoveredManifest)
        -> Result<Vec<ParsedDependency>, ParserError>;
}
```

`ParsedDependency` carries `name`, `current_version`, `ecosystem`,
`manifest_path`, and `is_dev_dependency`. `parse_manifests` routes each
`DiscoveredManifest` to the first parser that claims it; a manifest a parser
claims but cannot decode is recorded in `ParseOutcome::failures` and skipped,
so one malformed file never aborts the scan.

### Cargo parser (`src/parser/cargo.rs`)

Decodes `[dependencies]`, `[dev-dependencies]`, and `[workspace.dependencies]`.
A dependency that inherits its constraint from the workspace
(`serde = { workspace = true }`) is resolved against the workspace table.
Path- and git-only dependencies (no version constraint) are skipped — they are
not resolvable through a public registry.

### npm parser (`src/parser/npm.rs`)

Decodes `dependencies` and `devDependencies` from `package.json` with
`serde_json`. Specs that do not point at `registry.npmjs.org` (`file:`,
`link:`, `portal:`, `workspace:`, `npm:` aliases, and `git`/`http` URLs) are
skipped.

### Registry clients (`src/registry/`)

`RegistryClient::fetch_latest_version` is a native `async fn` trait method (no
`async-trait` dependency, no `dyn` dispatch). `CratesIoClient` calls
`/api/v1/crates/{name}` with the descriptive `User-Agent` crates.io requires
and reads `crate.max_version`; `NpmRegistryClient` calls
`registry.npmjs.org/{name}` (scoped names percent-encoded) and reads
`dist-tags.latest`. `fetch_latest_versions` keeps at most
`DEFAULT_CONCURRENCY` (10) requests in flight using `futures`
`buffer_unordered`. Response parsing is split into pure
`parse_crates_io_response` / `parse_npm_response` functions so they are unit
tested without network access.

## Design decisions

- Parsed with the `toml` crate plus serde structs rather than `cargo_toml`:
  fewer dependencies and direct access to `[workspace.dependencies]`.
- Bounded concurrency via `buffer_unordered` rather than `tokio::spawn` plus a
  `Semaphore`: identical request limit, no `'static` requirement.
- Registry failures (network error, 404, unexpected body) degrade to an
  `error` on that dependency and a `latest_version` of `unknown`; the scan
  always completes.

## Verification

`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` (28 tests) all pass. A live `scan` resolved real versions from
crates.io and npm and recorded a 404 for a non-existent crate without
aborting. Full detail in the [implementation report](implementation-report.md).

## Related

- [[architecture]] — the modules this feature implements.
- [[roadmap]] — Phase 2 of six.
- [implementation report](implementation-report.md) — file-by-file report,
  tests, and follow-ups.
- [[index]] — knowledge-graph entrypoint.
