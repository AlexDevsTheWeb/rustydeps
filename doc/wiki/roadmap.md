---
title: "RustyDeps — Roadmap"
type: roadmap
description: "Six delivery phases from Rust setup to multi-ecosystem expansion."
status: active
version: 0.4.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-11-27
tags: [roadmap, phases, planning]
---

# Roadmap

RustyDeps ships in six incremental phases. Each phase leaves the tool in a
usable state and maps to modules described in [[architecture]]. All six phases
are complete; Milestone 0.3.0 delivered the per-library AI summarizer and
Milestone 0.4.0 grounds it in official release notes.

## Phase status

| # | Phase | Status |
|---|-------|--------|
| 1 | Rust Setup & CLI (`clap` + `ignore`) | ✅ Done |
| 2 | Parsers (Cargo & npm) & Registry HTTP Clients | ✅ Done |
| 3 | Unified DTO & SemVer Logic | ✅ Done |
| 4 | Interactive HTML Report Engine | ✅ Done |
| 5 | Multi-ecosystem Expansion (PyPI, Go, Composer) | ✅ Done |
| 6 | AI Changelog Summarizer Module | ✅ Done |

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

## Phase 5 — Multi-ecosystem Expansion (PyPI, Go, Composer) ✅

Added `PypiParser`, `GoParser`, and `ComposerParser` plus the PyPI, Go module
proxy, and Packagist clients behind the existing `PackageParser`/`RegistryClient`
traits, and extended `Ecosystem` with `Pypi`/`Go`/`Composer`. Exit criteria met:
the new ecosystems required no changes to the scanner or the report template.
See [[phase-5-ecosystems]] and `doc/raw/2026-10-15/2026-10-15-phase5.md`.

## Phase 6 — AI Changelog Summarizer ✅

Implemented Milestone 0.3.0 (`doc/raw/2026-10-19/2026-10-19-v0.3.0.md`). Every
outdated, registry-resolved package now receives its own Markdown briefing:
breaking changes, security context, performance/architectural impact, and
migration advice. A layered BYOK configuration (user
`~/.config/rustydeps/config.json`, project `.rustydeps.json` fallback,
`RUSTYDEPS_AI_*`, CLI flags) selects an
OpenAI-compatible provider (OpenAI, Gemini, Ollama). Analyses are cached per
`{ecosystem}_{package}_{target_version}` and reused across projects. The report
drawer renders the analysis as safe Markdown, with a cache badge and a standard
changelog link. Exit criteria met: AI is optional, bounded to four concurrent
requests with a 10-second timeout, and every failure leaves the scan and report
intact. See [[ai-changelog-summarizer]].

## Milestone 0.4.0 — Real Registry Changelogs ✅

Implemented `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md` (Worktree 3).
Before each AI request that misses the cache, `ChangelogFetcher` resolves the
package repository from crates.io, npm, PyPI, Packagist, or a GitHub Go module
path, fetches the matching GitHub release (exact and `v`-toggled tags), and
injects a sanitized 1,500-character excerpt as the prompt's primary source.
Missing metadata, unknown tags, rate limits, and timeouts fall back to the
previous prompt. `GITHUB_TOKEN`/`GH_TOKEN` raises the API limit; `--offline`
skips lookups; cache format 2 regenerates v0.3.x entries once. See
[[v0.4.0-changelog-fetcher]].

## Related

- [[architecture]] — the modules each phase builds.
- [[ai-changelog-summarizer]] — the Phase 6 per-library AI summarizer.
- [[index]] — knowledge-graph entrypoint.
- `doc/raw/rustydeps-spec.md` — the requirements these phases satisfy.
