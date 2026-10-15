---
title: "Implementation Report — Phase 2 Parsers & Registry Clients"
type: report
description: "File-by-file report of the Phase 2 parsers and registry clients: what shipped, what was deferred, and how it was verified."
status: done
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [implementation-report, phase-2]
---

# Implementation Report — Phase 2

## Summary

The raw brief
`doc/raw/2026-10-15-phase-2/2026-10-15-phase2.md` asked for a parser subsystem
(`PackageParser` with `CargoParser`/`NpmParser`) and asynchronous registry
clients (crates.io and npm) wired into `scan`. All of that shipped on branch
`feat/phase-2`: the CLI now parses Cargo and npm dependencies and resolves each
package's latest published version, degrading to `unknown` on lookup failure.

## Done

- `src/model.rs` (new) — `Ecosystem` (`Cargo`, `Npm`) with a lowercase label
  for console and JSON output.
- `src/parser/mod.rs` (new) — `PackageParser` trait, `ParsedDependency` DTO,
  `ParserError`, `default_parsers()`, and `parse_manifests()` with a
  failure-tolerant `ParseOutcome`.
- `src/parser/cargo.rs` (new) — `[dependencies]`, `[dev-dependencies]`, and
  `[workspace.dependencies]`, including `workspace = true` inheritance.
- `src/parser/npm.rs` (new) — `dependencies` and `devDependencies`, with
  non-registry specs skipped.
- `src/registry/mod.rs` (new) — `RegistryClient` trait, `RegistryError`,
  `DEFAULT_CONCURRENCY = 10`, and the bounded `fetch_latest_versions` helper.
- `src/registry/crates_io.rs` (new) — `CratesIoClient` (`crate.max_version`,
  required `User-Agent`) plus pure `parse_crates_io_response`.
- `src/registry/npm.rs` (new) — `NpmRegistryClient` (`dist-tags.latest`,
  scoped-name encoding) plus pure `parse_npm_response`.
- `src/main.rs` (changed) — `scan` parses then resolves per ecosystem and
  prints a table or JSON summary.
- `Cargo.toml` (changed) — added `toml`, `reqwest` (`json`), and `futures`.
- `doc/wiki/architecture.md`, `roadmap.md`, `log.md`, `index.md` — updated for
  Phase 2.

## Not done / deferred

- SemVer classification (`UpdateType`, Safe/At-Risk/Breaking) and the HTML
  report engine remain in Phases 3 and 4; `scan` still prints text/JSON.
- `[build-dependencies]` and npm `optionalDependencies`/`peerDependencies` are
  not extracted yet (the brief scoped Phase 2 to `dependencies` plus
  `dev-dependencies`).
- Cargo prerelease-only crates report `crate.max_version` verbatim; the
  "latest non-prerelease" refinement is deferred to the SemVer work.

## Blocked

None. The brief's "Draft PR #3" was inaccurate — PR #3 already exists (the
merged HTML report UI spec), so this work was opened as draft PR #4 instead.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 28 passed, 0 failed.
- `cargo run -- scan tmp/phase-2/fixture --format json` — resolved real
  versions (serde 1.0→1.0.229, anyhow 1.0→1.0.104, tempfile 3→3.27.0, express
  ^4.18.0→5.3.0, typescript ~5.4.0→7.0.2) and recorded a crates.io 404 as an
  error without aborting.
- OKF validator — see [[log]] for the result recorded at the documentation
  commit.

## MR

Draft PR #4 into `development` from `feat/phase-2`:
<https://github.com/AlexDevsTheWeb/rustydeps/pull/4>.

## Related

- [[phase-2-parsers-registry]] — the feature page for this work.
- [[architecture]] — module design.
- [[roadmap]] — Phase 2 of six.
