---
title: "RustyDeps Wiki — Log"
type: log
description: "Append-only history of wiki changes."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-15
tags: [log, history]
---

# Log

Append-only history. Newest entries at the bottom. Never edit or delete an
earlier entry. See [[index]] for the catalogue.

## 2026-10-14 — Wiki initialized

Created the initial OKF wiki during the scaffolding task
(`doc/raw/2026-10-14-scaffolding/2026-10-14-scaffolding.md`):

- [[index]] — catalogue and knowledge-graph entrypoint.
- [[architecture]] — module breakdown, `PackageParser` trait, and core DTOs
  (`Dependency`, `UpdateType`).
- [[roadmap]] — the six delivery phases.
- [[okf-compliance]] — open formats, licensing, and validation via the
  `.claude/skills/wiki-scribe/scripts/validate_okf.py` script.
- [[log]] — this page.

Also created `doc/raw/rustydeps-spec.md` (core specification) and rewrote
`AGENTS.md` / `CLAUDE.md` for the Rust stack, removing the legacy
React/TypeScript/Vite references.

## 2026-10-14 — Phase 1: Rust crate, CLI, and scanner

Implemented Phase 1 (`doc/raw/2026-10-14-phase-1/2026-10-14-phase-1.md`):

- Initialized the binary crate (`Cargo.toml`, edition 2021) with `clap`,
  `ignore`, `tokio`, `anyhow`, `thiserror`, `serde`/`serde_json`, `tracing`, and
  `tracing-subscriber`; `tempfile` as a dev-dependency.
- `src/cli.rs` — `Cli`, the `scan` subcommand (`path`, `--ai`, `--output`/`-o`,
  `--format html|json`, `--verbose`/`-v`), built with `clap` derive.
- `src/main.rs` — `#[tokio::main]` entrypoint; `tracing-subscriber` to stderr;
  dispatch to the scanner and print results.
- `src/scanner.rs` — `ManifestType`, `DiscoveredManifest`, and
  `scan_directory` over `ignore::WalkBuilder` (gitignore/ignore/hidden rules,
  `node_modules`/`target`/`.git` pruned); six unit tests.
- Added `LICENSE-MIT` and `LICENSE-APACHE`; removed the unrelated
  `.claude/skills/stitch-to-mui` skill.
- Updated [[architecture]] (implemented CLI/scanner modules) and [[roadmap]]
  (Phase 1 marked done). This entry.

Validation: `cargo fmt --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 6 passed; OKF validator PASSED (0 errors, 0
warnings).

## 2026-10-14 — HTML report UI specification added

Documented the finalized HTML report design produced by the Stitch UI iterations
(`doc/raw/stitch/`):

- Added [[html-report-spec]] — industrial dark mode, Zinc palette, project-centric
  component hierarchy, minimalist 6px risk dots, discrete grey `AI` tag, inline
  dependency drawer, filter/search bar, empty state, and "Update All".
- Linked it from [[index]] (knowledge map) and from the report-generator section
  of [[architecture]].
- This entry.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Phase 2: parsers, registry clients, and CLI wiring

Implemented the Phase 2 brief
(`doc/raw/2026-10-15-phase-2/2026-10-15-phase2.md`):

- Added the feature subfolder `features/phase-2-parsers-registry/` with
  [[phase-2-parsers-registry]] (feature page) and [[implementation-report]].
- `src/parser/` — the `PackageParser` trait, `CargoParser`, `NpmParser`, the
  `ParsedDependency` DTO, and the failure-tolerant `parse_manifests` driver.
- `src/registry/` — the `RegistryClient` trait, `CratesIoClient`,
  `NpmRegistryClient`, and the bounded `fetch_latest_versions` helper.
- `src/model.rs` — `Ecosystem`. `src/main.rs` — `scan` now parses and resolves,
  printing a table or JSON summary. `Cargo.toml` — added `toml`, `reqwest`,
  and `futures`.
- Updated [[architecture]] (parser and registry modules marked implemented),
  [[roadmap]] (Phase 2 ✅, Phase 3 next), and [[index]] (feature row and Phase 2
  source document).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 28 passed; OKF validator PASSED (0 errors,
0 warnings).

## 2026-10-15 — Phase 2 draft PR opened

Opened draft PR #4 (`feat/phase-2` → `development`) and recorded the link in
[[implementation-report]]'s MR section.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Raw documents reorganized; wiki references repointed

Consolidated the dated raw inputs under `doc/raw/2026-10-15/` and repointed
the wiki paths that named the old locations:

- `doc/raw/2026-10-15-phase-2/2026-10-15-phase2.md` and
  `doc/raw/stitch/opencode-prompt-wiki-html-spec.md` moved into
  `doc/raw/2026-10-15/`; the superseded Stitch iteration prompts were removed.
  A new Phase 3 brief (`doc/raw/2026-10-15/2026-10-15-phase3.md`) was added.
- Updated [[html-report-spec]] and [[index]] to the new prompt/Phase 2 paths,
  and [[roadmap]] and [[implementation-report]] to the new Phase 2 path.
- Earlier log entries that name the old paths are left unchanged: this log is
  append-only, so those remain a faithful record of where the files lived then.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Phase 3: SemVer classifier, AI summaries, banner, offline mode

Implemented the Phase 3 brief
(`doc/raw/2026-10-15/2026-10-15-phase3.md`):

- Added the feature subfolder `features/phase-3-semver-classifier/` with
  [[phase-3-semver-classifier]] (feature page) and its implementation report.
- `src/model.rs` — `UpdateType` (`Safe`, `AtRisk`, `Breaking`, `UpToDate`,
  `Unknown`). `src/classifier/` — `classify_update`, `classify`,
  `ClassifiedDependency`, `latest_stable`, and `tests.rs`.
- `src/registry/crates_io.rs` — the upgrade target is now the latest stable
  release (`max_stable_version` → highest stable in `versions` → `max_version`).
- `src/ai/` — provider detection (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`,
  `RUSTYDEPS_AI_ENDPOINT`) and templated summaries; live provider calls are
  deferred to Phase 5.
- `src/cli.rs`, `src/main.rs` — the ASCII banner (suppressed for JSON) and the
  `--offline`/`--no-fetch` flag. `Cargo.toml` — added `semver` and `colored`.
- Updated [[architecture]] (classifier and AI sections), [[roadmap]] (Phase 3
  ✅, Phase 4 next), and [[index]] (feature row and Phase 3 source document).
- The active feature pages and [[index]] now link their implementation reports
  by relative path instead of `[[implementation-report]]`, which became
  ambiguous once a second subfolder used that filename.

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 47 passed; OKF validator PASSED (0 errors,
0 warnings).

## 2026-10-15 — Phase 3 draft PR opened

Opened draft PR #6 (`feat/phase-3` → `development`) and recorded the link in
the Phase 3 implementation report's MR section.

Validation: OKF validator PASSED (0 errors, 0 warnings).


