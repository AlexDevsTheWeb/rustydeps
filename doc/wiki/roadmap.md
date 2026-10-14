---
title: "RustyDeps — Roadmap"
type: roadmap
description: "Six delivery phases from Rust setup to multi-ecosystem expansion."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-14
tags: [roadmap, phases, planning]
---

# Roadmap

RustyDeps ships in six incremental phases. Each phase leaves the tool in a
usable state and maps to modules described in [[architecture]]. Phase 1 is
complete; Phase 2 is next.

## Phase status

| # | Phase | Status |
|---|-------|--------|
| 1 | Rust Setup & CLI (`clap` + `ignore`) | ✅ Done |
| 2 | Parsers (Cargo & npm) & Registry HTTP Clients | Next |
| 3 | Unified DTO & SemVer Logic | Planned |
| 4 | Interactive HTML Report Engine | Planned |
| 5 | AI Changelog Summarizer Module | Planned |
| 6 | Multi-ecosystem Expansion (PyPI, Go, Composer) | Planned |

## Phase 1 — Rust Setup & CLI ✅

Crate initialized (edition 2021) with `clap` (derive) and the `ignore`-based
scanner. The `scan` subcommand walks a root, honours ignore rules, and lists
every supported manifest as text or JSON. Dual licensing (MIT/Apache-2.0) added.
Exit criteria met: the CLI runs, accepts roots, and lists discovered manifest
paths. See [[architecture]] and `doc/raw/development-report-*`.


## Phase 2 — Parsers & Registry Clients

Implement the `PackageParser` trait with `CargoParser` and `NpmParser`, plus the
`reqwest`/`tokio` registry clients for crates.io and the npm registry. Exit
criteria: a scan resolves the latest version for Cargo and npm dependencies.

## Phase 3 — Unified DTO & SemVer Logic

Lock down `Dependency` and `UpdateType`, and implement the pure classifier that
buckets upgrades into Safe / At-Risk / Breaking. Exit criteria: a fully tested
classifier driven by the DTOs.

## Phase 4 — Interactive HTML Report Engine

Render a self-contained, filterable/sortable HTML report from the result set.
Exit criteria: a report opens offline and reflects a real scan.

## Phase 5 — AI Changelog Summarizer

Add the optional, flag-gated AI module that summarizes changelogs and flags
migration notes. Exit criteria: `--ai` adds summaries without becoming a hard
dependency of a successful scan.

## Phase 6 — Multi-ecosystem Expansion

Add PyPI, Go (`go.mod`), and Composer parsers and clients behind the existing
trait. Exit criteria: new ecosystems require no changes to the scanner.

## Related

- [[architecture]] — the modules each phase builds.
- [[index]] — knowledge-graph entrypoint.
- `doc/raw/rustydeps-spec.md` — the requirements these phases satisfy.
