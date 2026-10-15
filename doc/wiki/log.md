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

## 2026-10-15 — Docker image & CI publishing

Applied `doc/raw/2026-10-15/opencode-prompt-08-issue15-docker.md` (issue #15,
"publish an official Docker image for use in CI pipelines"):

- `Dockerfile` (new) — `rust:1-alpine` builder (`musl-dev`, `openssl-dev`,
  `pkgconf`; dependency-graph layer compiled against a stub `main.rs`, then
  `src/` and `templates/`) and an `alpine:3` runtime
  (`ca-certificates`, `libssl3`, `libgcc`) with
  `ENTRYPOINT ["rustydeps"]`, `CMD ["scan", "."]`, `WORKDIR /workspace` and OCI
  labels.
- `.dockerignore` (new) — keeps the build context to `Cargo.toml`,
  `Cargo.lock`, `src/` and `templates/`.
- `.github/workflows/docker.yml` (new) — builds on `main` pushes, `v*` tags and
  PRs to `main` and `development`; QEMU + Buildx; GHCR login via `GITHUB_TOKEN`; publishes
  `linux/amd64` + `linux/arm64` tagged `latest`/branch/semver, and on PRs loads
  a single-platform image and smoke-tests it instead of pushing.
- `README.md` — Docker workflow badge, a `## Docker` section (run commands,
  GitLab CI and GitHub Actions snippets, image notes), and a *Releasing* note
  that a `v*` tag publishes the image too.
- Added `features/docker-image/` with [[docker-image]] (feature page) and its
  implementation report; updated [[index]].

Two brief specifics changed and are documented in [[docker-image]]: the runtime
base is the rolling `alpine:3` rather than `alpine:3.19` (past its release
window, and it keeps the builder and runtime on the same Alpine release), and
distroless was rejected because the issue requires a shell in the job image.
The runtime OpenSSL libraries were kept because `reqwest` uses `native-tls` on
Linux.

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 85 passed; `cargo build --release --locked`
succeeds; the smoke-test commands run green against the local release binary;
`cargo tree --target x86_64-unknown-linux-gnu -i openssl-sys` confirms the
OpenSSL dependency; `docker.yml` parses as YAML and both `run:` blocks pass
`bash -n`; OKF validator PASSED (0 errors, 0 warnings). No container runtime is
installed on the development machine, so the image build itself is first
exercised by the new workflow.

## 2026-10-15 — Docker image draft PR opened

Opened draft PR #16 (`feat/docker-support` → `development`) and recorded the
link in the docker-image implementation report's MR section. The PR body lists
the unverified gates, the two brief deviations (rolling `alpine:3`, distroless
rejected), and the deferred work.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Docker build test on PRs into `development`

Corrected the trigger for the new workflow: the pull-request filter was
`main`-only, but feature branches integrate into `development` in this
repository, so the image build test would not have run on the PR that adds it.
`.github/workflows/docker.yml` now lists `branches: [main, development]` for
`pull_request`, and its header comment no longer claims pull requests build both
architectures (they build `linux/amd64` and load it for the smoke test). Push
triggers are unchanged: `main` plus `v*` tags, so `latest` still comes from the
default branch.

Also corrected in the same step: [[docker-image]] (trigger prose and the event
table), the docker-image implementation report, the `log.md` entry above, and
`doc/raw/2026-10-15/2026-10-15-issue15-docker.md` (deviation 5 plus a correction
note; the raw correction is flagged inline because raw documents are otherwise
immutable).

Validation: workflow YAML parses and both `run:` blocks still pass `bash -n`;
OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-15 — Docker image verified in CI; two defects found and fixed

The PR's Docker workflow was exercised end to end, which caught two defects that
would otherwise have shipped a non-functional image:

1. **Run 38800896598 (failed) — the stub binary shipped as `rustydeps`.** The
   strengthened smoke test reported an empty `--version`. The build log showed
   `[builder 8/8] RUN cargo build --release --locked` finishing in 0.14 s with no
   recompile: cargo's mtime freshness check treats the in-image stub
   `src/main.rs` as newer than the sources `COPY` preserves, so the real crate
   was never built and `fn main() {}` was copied into the runtime stage.
2. **Run 38801611324 (failed) — `ld: cannot find -lssl`.** With the crate
   finally compiling, the link failed because `x86_64-unknown-linux-musl` links
   statically by default (`crt-static`) and therefore needs `libssl.a`. The stub
   had hidden this: unused crates are not linked.

Fixes: `touch src/main.rs` plus a build-time
`./target/release/rustydeps --version | grep -q '^rustydeps '` assertion;
`openssl-libs-static` added to the builder (Alpine builds OpenSSL with `no-zlib`,
per aports `main/openssl/APKBUILD`, so no `-lz`); the runtime stage reduced to
`ca-certificates` because the static binary carries libc, libgcc and OpenSSL
internally. The smoke test was also rewritten to assert on container output
rather than trust a green step.

The mtime-skip mechanism was reproduced locally in a scratch crate. Files
changed: `Dockerfile`, `.github/workflows/docker.yml`, `.dockerignore`,
[[docker-image]] (components, decisions, verification), its implementation
report, and `doc/raw/2026-10-15/2026-10-15-issue15-docker.md`.

**Run 38801998432 (PR #16) is green**: the build compiles the crate and passes
its assertion, and the smoke test reports `container reports: rustydeps 0.1.0`
and `report written through the mount: 65482 bytes` (report asserted non-empty
HTML). Unverified: the `linux/arm64` leg and a real GHCR push, which only run on
`main`/tag events.

Validation: workflow YAML parses and all `run:` blocks pass `bash -n`;
`cargo fmt`/`clippy`/`test` (85) unaffected; OKF validator PASSED (0 errors,
0 warnings).


