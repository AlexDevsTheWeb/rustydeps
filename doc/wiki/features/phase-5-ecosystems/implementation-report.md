---
title: "Implementation Report — Phase 5 Expanded Ecosystems"
type: report
description: "File-by-file report of the Phase 5 PyPI, Go, and Composer parsers and registry clients."
status: done
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [implementation-report, phase-5]
---

# Implementation Report — Phase 5

## Summary

The raw brief `doc/raw/2026-10-15/2026-10-15-phase5.md` asked for three new
ecosystems behind the existing `PackageParser` and `RegistryClient` traits:
Python (PyPI), Go, and PHP (Composer). All shipped on branch
`feat/phase-5-ecosystems`; a scan now parses and resolves five ecosystems, and
the Stitch report renders them with no template change.

## Done

- `src/model.rs` — `Ecosystem` gains `Pypi`, `Go`, `Composer` with lowercase
  labels.
- `src/parser/pypi.rs` (new) — `pyproject.toml` (PEP 621 + Poetry) and
  `requirements.txt`.
- `src/parser/go.rs` (new) — `go.mod` requires (direct only).
- `src/parser/composer.rs` (new) — `composer.json` `require`/`require-dev`.
- `src/parser/mod.rs` — modules registered in `default_parsers()`; a test
  asserts every `ManifestType` has a parser.
- `src/registry/pypi.rs`, `go.rs`, `composer.rs` (new) — clients with pure
  `parse_*` functions and unit tests.
- `src/registry/mod.rs` — modules registered.
- `src/classifier/mod.rs` — `parse_version` accepts a leading `v`;
  `declared_version` strips `!`; `latest_stable` returns the winning entry
  verbatim.
- `src/main.rs` — `resolve_lookups` refactored to a generic `resolve_ecosystem`
  helper; all five ecosystems resolve concurrently.
- `src/report/mod.rs` — ecosystem labels and update commands for the new
  ecosystems.
- Wiki: this subfolder plus updates to `architecture.md`, `roadmap.md`,
  `index.md`, and `log.md`.

## Not done / deferred

- `[project.optional-dependencies]` (extras) and Poetry dependency groups are
  not parsed; only `dependencies`/`dev-dependencies`.
- Go `replace` resolution and `// indirect` dependencies are ignored (the brief
  asked for direct requires).
- Composer platform filtering is name-prefix based; exotic pseudo-packages are
  not enumerated.
- The AI changelog summarizer is now the next planned phase (was the roadmap's
  "Phase 5"; per the brief it moves after the ecosystems).

## Blocked

None.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 83 passed, 0 failed.
- Live `scan --format json` over a six-manifest fixture (Cargo, npm,
  pyproject.toml, requirements.txt, go.mod, composer.json) resolved all five
  ecosystems: e.g. `flask =2.3.0 → 3.1.3` breaking,
  `requests >=2.28.0 → 2.34.2` safe, `gin v1.9.1 → v1.12.0` at-risk,
  `monolog/monolog ^3.0 → 3.12.1` safe, `express ^4.18.0 → 5.3.0` breaking.
  Indirect/testify, `php`/`ext-json`, and `-r` lines were skipped.
- The HTML report rendered PyPI/Go/Composer pills and the correct commands, and
  remained 100% self-contained.

## MR

Draft PR into `development` from `feat/phase-5-ecosystems` (see the PR
description for the link).

## Related

- [[phase-5-ecosystems]] — the feature page for this work.
- [[architecture]] — the parser and registry traits.
- [[phase-2-parsers-registry]] — the Cargo/npm equivalent.
- [[roadmap]] — Phase 5 of six.
