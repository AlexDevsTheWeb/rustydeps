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

## 2026-10-15 — Phase 4: interactive HTML report engine

Implemented the Phase 4 brief
(`doc/raw/2026-10-15/2026-10-15-phase4.md`):

- Added the feature subfolder `features/phase-4-html-report/` with
  [[phase-4-html-report]] (feature page) and its implementation report.
- `src/report/` — `ReportData`/`ProjectReport`/`DependencyReport`,
  `from_classified` grouping, `render_html`, and tests.
- `templates/report.html` — the Askama template implementing
  [[html-report-spec]] with embedded CSS and vanilla JavaScript.
- `src/cli.rs`, `src/main.rs` — `--output` defaults to
  `rustydeps-report.html`; `--format html` writes the report and prints a
  `file://` URL.
- `Cargo.toml` — added `askama`. `.gitignore` — allow-listed
  `templates/**/*.html` so the source template is tracked.
- Updated [[architecture]] (report module), [[roadmap]] (Phase 4 ✅, Phase 5
  next), [[index]] (feature row and Phase 4 source document), and
  [[html-report-spec]] (now implemented).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 56 passed; OKF validator PASSED (0 errors,
0 warnings).

## 2026-10-15 — Phase 4 draft PR opened

Opened draft PR #7 (`feat/phase-4` → `development`) and recorded the link in
the Phase 4 implementation report's MR section.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Stitch UI v4 template integrated into the report engine

Adopted the Google Stitch HTML export
(`doc/raw/2026-10-15/opencode-prompt-stitch-integration.md`) as the report
template:

- `src/templates/report_stitch.html` — the raw design export, tracked as a
  reference input (`.gitignore` allow-lists `src/templates/**/*.html`).
- `templates/report.html` — the export's CSS/JS kept verbatim, the mock body
  replaced by Askama loops over the data, the Google Fonts links removed for
  self-containment, plus a supplemental stylesheet (empty state, batch modal,
  `semver-badge`, `migration-step`, `terminal-prompt-prefix`) and the missing
  JS handlers (`openBatchModal`, `closeBatchModal`, `copyBatchModalCommand`,
  `handleBackdropClick`, `resetAllFilters`, and the empty-state toggle).
- `src/report/mod.rs`, `src/report/tests.rs`, `src/main.rs` — model reshaped to
  the design's fields; `ReportMeta::scanned_root` dropped.
- Updated [[phase-4-html-report]] (template + Stitch integration sections),
  its implementation report (follow-up section), [[architecture]] (report
  generator), and [[html-report-spec]] (related brief).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 57 passed; a live scan produced a ~55 KB
self-contained report with all drawer ids resolving and every JS handler
defined; OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Phase 5: PyPI, Go, and Composer ecosystems

Implemented the Phase 5 brief
(`doc/raw/2026-10-15/2026-10-15-phase5.md`):

- Added the feature subfolder `features/phase-5-ecosystems/` with
  [[phase-5-ecosystems]] (feature page) and its implementation report.
- `src/parser/` — `PypiParser` (`pyproject.toml`, `requirements.txt`),
  `GoParser` (`go.mod`), `ComposerParser` (`composer.json`), registered in
  `default_parsers()`.
- `src/registry/` — `PypiClient`, `GoRegistryClient`, `ComposerClient` with
  pure `parse_*` functions.
- `src/model.rs` — `Ecosystem` gains `Pypi`, `Go`, `Composer`.
- `src/classifier/` — `v`-prefixed tags and `!=` specifiers accepted;
  `latest_stable` returns the winning entry verbatim.
- `src/main.rs` — a generic `resolve_ecosystem` helper resolves all five
  ecosystems concurrently. `src/report/mod.rs` — ecosystem labels and update
  commands for the new ecosystems (no template change).
- Updated [[architecture]] (parsers + clients), [[roadmap]] (Phase 5 =
  Multi-ecosystem ✅, Phase 6 = AI summarizer next), and [[index]] (feature row
  and Phase 5 source document).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 83 passed; a live scan resolved all five
ecosystems and the report rendered PyPI/Go/Composer pills while staying
self-contained; OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Phase 5 draft PR opened

Opened draft PR #8 (`feat/phase-5-ecosystems` → `development`) and recorded the
link in the Phase 5 implementation report's MR section.
## 2026-10-15 — Auto-open the HTML report in the default browser

Applied `doc/raw/2026-10-15/opencode-prompt-auto-open.md`:

- `src/cli.rs` — a `--no-open` flag on `scan`; the report opens by default on
  `--format html`.
- `src/main.rs` — after writing the report and printing the path/`file://` URL,
  `open::that` launches it; success logs at info, failure logs a warning and
  never fails the scan. `--format json` and `--no-open` skip the call.
- `Cargo.toml` — added the `open` crate (5.x).
- Updated [[architecture]] (CLI flags and behaviour).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 59 passed; live checks confirmed `--no-open`
suppresses the launch, the default opens and logs success, `--format json` does
not open, and the scan still exits 0; OKF validator PASSED (0 errors,
0 warnings).

## 2026-10-15 — Auto-open draft PR opened

Opened draft PR #9 (`feat/auto-open` → `development`).

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Release automation and README

Applied `doc/raw/2026-10-15/2026-10-15-phase7-release.md` (titled "Phase 6";
treated as release tooling, not a roadmap phase):

- `README.md` — full rewrite: ASCII banner, v0.1.0/license/Rust/Release badges,
  the live website link, a supported-ecosystems table, a risk legend,
  quickstart/usage, the feature overview, and releasing/development/license
  sections.
- `.github/workflows/release.yml` (new) — tag-triggered (`v*`) release: a
  three-target build matrix (`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`,
  `aarch64-apple-darwin`), tar.gz + sha256 packaging, and a GitHub Release with
  generated changelog notes and attached assets.
- Added `features/release-automation/` with [[release-automation]] (feature
  page) and its implementation report; updated [[index]].

The `v0.1.0` tag is not pushed (the developer will push it after this lands on
the default branch). Blank specifics in the brief were resolved by developer
confirmation: `v*` trigger, gnu Linux + both macOS arches.

Validation: workflow YAML parses and both `run:` blocks pass `bash -n`;
`cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings`
clean; `cargo test` 85 passed; OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Release automation draft PR opened

Opened draft PR #13 (`feat/release-automation` → `development`) and recorded the
link in the release-automation implementation report's MR section.

Validation: OKF validator PASSED (0 errors, 0 warnings).


