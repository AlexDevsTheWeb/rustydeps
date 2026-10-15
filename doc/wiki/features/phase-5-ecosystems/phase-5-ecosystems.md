---
title: "Phase 5 — Expanded Ecosystems (PyPI, Go, Composer)"
type: feature
description: "Adds PyPI, Go and Composer manifest parsers and registry clients behind the existing traits, with no report template change."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [feature, phase-5, pypi, go, composer, parser, registry]
---

# Phase 5 — Expanded ecosystems (PyPI, Go, Composer)

## Motivation

After Phase 4, RustyDeps scanned and classified Cargo and npm dependencies and
rendered them in the Stitch report. The architecture was built around two seams
— the `PackageParser` trait and the `RegistryClient` trait — precisely so more
ecosystems could be added without touching the scanner or the report. Phase 5
proves that out by adding Python (PyPI), Go, and PHP (Composer): three parsers,
three clients, an `Ecosystem` extension, and zero report-template changes.

## What shipped

- **Parsers (`src/parser/`)** — `PypiParser` (`pyproject.toml`,
  `requirements.txt`), `GoParser` (`go.mod`), `ComposerParser`
  (`composer.json`), all registered in `default_parsers()`.
- **Registry clients (`src/registry/`)** — `PypiClient`, `GoRegistryClient`,
  `ComposerClient`.
- **`Ecosystem`** — gained `Pypi`, `Go`, `Composer`.
- **CLI/report** — `resolve_lookups` now runs all five ecosystems concurrently;
  the report maps the new ecosystem labels and update commands.

## Components

### Parsers

| Parser | Manifest(s) | Notes |
|--------|-------------|-------|
| `PypiParser` | `pyproject.toml`, `requirements.txt` | PEP 621 `[project.dependencies]` and Poetry `[tool.poetry.dependencies]`/`dev-dependencies`; PEP 508 strings reduced to `(name, specifier)`. Skips the Poetry `python` entry and path/git-only deps; skips `-r`/`-e`/comments in `requirements.txt`. |
| `GoParser` | `go.mod` | `require` blocks and single-line requires; skips `// indirect`, `replace`, and `exclude`. Versions keep their `v` prefix (exact pins). |
| `ComposerParser` | `composer.json` | `require`/`require-dev`; skips platform pseudo-packages (`php`, `php-*`, `ext-*`, `lib-*`, `composer-*`, `hhvm`). |

PEP 508 specifiers are normalized (`~=`→`~`, `==`→`=`, whitespace removed,
markers and extras stripped); a requirement with no pin becomes `*` so the
dependency still surfaces and classifies as `Unknown`.

### Registry clients

| Client | Endpoint | Latest-stable logic |
|--------|----------|---------------------|
| `PypiClient` | `pypi.org/pypi/{name}/json` | Highest non-prerelease in `releases` with a non-yanked file; falls back to `info.version`. |
| `GoRegistryClient` | `proxy.golang.org/{module}/@v/list` | Highest stable tag; uppercase module-path letters encoded as `!x`; `v` prefix kept. |
| `ComposerClient` | `repo.packagist.org/p2/{name}.json` | Highest stable tag, skipping `dev-*` and prereleases. |

Each client keeps its response parsing in a pure `parse_*` function so it is
unit-tested without network access, exactly like the Phase 2 clients.

### Classifier and report integration

- `parse_version` accepts a leading `v` (Go/Packagist tags) and
  `declared_version` strips `!` (PyPI `!=`); `latest_stable` returns the winning
  entry verbatim so `v1.2.3` is preserved.
- The report maps ecosystem labels (`Cargo`, `npm`, `PyPI`, `Go`, `Composer`)
  and per-ecosystem update commands:
  `cargo update -p <name> --precise <ver>`, `npm install <name>@<ver>`,
  `pip install <name>==<ver>`, `go get <name>@<ver>`,
  `composer require <name>:<ver>`.

## Design decisions

- One `PypiParser` claims both `pyproject.toml` and `requirements.txt`: same
  ecosystem, one trait implementation.
- Go keeps the `v` prefix so a `require` (an exact pin) that moves to a newer
  version classifies as `AtRisk` on a minor bump — this matches the report's
  Stitch mock for gin.
- The Go proxy `@v/list` is used instead of `@latest` so the highest *stable*
  tag is chosen rather than the proxy's "latest" (which can be a prerelease).
- Packagist's `p2` metadata endpoint is used rather than the legacy
  `packages/{name}.json`.
- No report template change: the Stitch `.ecosystem-pill` is generic and
  uppercased by CSS; only the label text differs.

## Verification

`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` (83 tests) all pass. A live scan over a six-manifest fixture
resolved all five ecosystems and rendered the PyPI/Go/Composer pills without any
template change. See the [implementation report](implementation-report.md).

## Related

- [[architecture]] — the parser and registry traits this extends.
- [[roadmap]] — Phase 5 of six.
- [[phase-2-parsers-registry]] — the Cargo/npm parsers this mirrors.
- [implementation report](implementation-report.md) — file-by-file detail.
- [[index]] — knowledge-graph entrypoint.
