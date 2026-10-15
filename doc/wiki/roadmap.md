---
title: "RustyDeps — Roadmap"
type: roadmap
description: "Six delivery phases from Rust setup to multi-ecosystem expansion."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-15
tags: [roadmap, phases, planning]
---

# Roadmap

RustyDeps ships in six incremental phases. Each phase leaves the tool in a
usable state and maps to modules described in [[architecture]]. Phases 1
through 4 are complete; Phase 5 is next.

## Phase status

| # | Phase | Status |
|---|-------|--------|
| 1 | Rust Setup & CLI (`clap` + `ignore`) | ✅ Done |
| 2 | Parsers (Cargo & npm) & Registry HTTP Clients | ✅ Done |
| 3 | Unified DTO & SemVer Logic | ✅ Done |
| 4 | Interactive HTML Report Engine | ✅ Done |
| 5 | AI Changelog Summarizer Module | Next |
| 6 | Multi-ecosystem Expansion (PyPI, Go, Composer) | Planned |

## Phase 1 — Rust Setup & CLI ✅

Crate initialized (edition 2021) with `clap` (derive) and the `ignore`-based
scanner. The `scan` subcommand walks a root, honours ignore rules, and lists
every supported manifest as text or JSON. Dual licensing (MIT/Apache-2.0) added.
Exit criteria met: the CLI runs, accepts roots, and lists discovered manifest
paths. See [[architecture]] and `doc/raw/development-report-*`.


## Phase 2 — Parsers & Registry Clients ✅

Implemented the `PackageParser` trait with `CargoParser` and `NpmParser`
(`src/parser/`), plus the `reqwest`/`tokio` registry clients for crates.io and
the npm registry (`src/registry/`), wired into `scan`. A malformed manifest or
a failed registry lookup degrades gracefully instead of aborting the run. Exit
criteria met: a scan parses Cargo and npm dependencies and resolves each one's
latest published version. See [[phase-2-parsers-registry]] and
`doc/raw/2026-10-15/2026-10-15-phase2.md`.

## Phase 3 — Unified DTO & SemVer Logic ✅

Locked down `UpdateType` and the `ClassifiedDependency` result DTO, added the
pure classifier (`Safe` / `AtRisk` / `Breaking` / `UpToDate` / `Unknown`), the
latest-stable target, the CLI banner, the templated AI summary scaffold, and
`--offline`/`--no-fetch`. Exit criteria met: a fully tested classifier driven by
the DTOs (`src/classifier/tests.rs`). See [[phase-3-semver-classifier]] and
`doc/raw/2026-10-15/2026-10-15-phase3.md`.

## Phase 4 — Interactive HTML Report Engine ✅

Built the report engine (`src/report/`) and the Askama template
(`templates/report.html`) per [[html-report-spec]]: project-centric cards, Zinc
dark mode, 6px risk dots, a discrete `AI` tag, an inline drawer with copyable
update commands, search and risk filters, an empty state, and per-project
"Update All". `scan --format html` writes a single self-contained
`rustydeps-report.html`. Exit criteria met: the report opens offline and
reflects a real scan. See [[phase-4-html-report]] and
`doc/raw/2026-10-15/2026-10-15-phase4.md`.

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
