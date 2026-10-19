---
title: "RustyDeps Wiki — Log"
type: log
description: "Append-only history of wiki changes."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-19
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

## 2026-10-17 — CLI scanning UX & v0.2.0 (issue #19)

Applied `doc/raw/2026-10-17/2026-10-17-issue19.md` (issue #19, "Improvement:
review CLI interface when scanning"), preparing release v0.2.0:

- Added the feature subfolder `features/cli-scanning-ux/` with
  [[cli-scanning-ux]] (feature page) and its implementation report.
- `src/progress.rs` (new) — `ScanProgress`, a thin wrapper over `indicatif`:
  spinners for discovery and registry resolution, a `pos/len` counter for
  per-repository reporting, hidden when stderr is not a TTY. Pure
  `project_label` and `repository_summary` helpers.
- `src/main.rs` — spinners around discovery and resolution; a new
  `report_repositories` prints `✔ Scanned '<relative path>': N dependencies
  (<ecosystem>)` per manifest and advances the `Scanning repositories [bar]
  X/N` counter. `--format json` suppresses the banner and all status lines so
  stdout stays one JSON document (bars already draw to stderr).
- `src/scanner.rs` — `ManifestType::ecosystem_label()` for the report-matching
  pill labels.
- `Cargo.toml`/`Cargo.lock` — `version = "0.2.0"` and `indicatif = "0.18"`.
  `README.md` — banner, badge and Docker/CI examples bumped to `0.2.0`.
- Updated [[architecture]] (CLI progress paragraph), [[index]] (feature row and
  source document), and this log.

The brief's step 5 names a wiki path under `doc/raw/`; `doc/raw/` is immutable
input in this repository, so the documentation lives under
`doc/wiki/features/cli-scanning-ux/` (deviation recorded on the feature page and
the implementation report).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 89 passed (85 + 3 progress + 1 scanner);
`cargo build --release` succeeds and `--version` reports `rustydeps 0.2.0`;
non-TTY and `--format json` runs emit no animation and a clean JSON stream; a
pty run shows the spinners and the `0/2 → 2/2` counter; OKF validator PASSED
(0 errors, 0 warnings).

## 2026-10-17 — CLI scanning UX draft PR opened

Opened draft PR #20 (`feat/cli-scanning-ux` → `development`) and recorded the
link in the CLI-scanning-UX implementation report's MR section.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-17 — Repository hygiene and the issue #15 source brief

Two small follow-ups folded into PR #20:

- `.gitignore` — ignore the generated `/rustydeps-report.html` at the repository
  root, so a default `rustydeps scan .` no longer leaves an untracked report
  next to the manifest files.
- Committed the previously untracked issue #15 brief
  (`doc/raw/2026-10-15/opencode-prompt-08-issue15-docker.md`) and added it to
  the [[index]] source-document list as the prompt [[docker-image]] implements.

Validation: `git check-ignore -v rustydeps-report.html` matches the new rule;
OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-17 — UI font scaling & branding (issue #21)

Applied `doc/raw/2026-10-17/2026-10-17-issue21.md` (issue #21, "UI Font Scaling
& Branding (Favicon + Logo)"):

- Added the feature subfolder `features/ui-font-branding/` with
  [[ui-font-branding]] (feature page) and its implementation report.
- `templates/report.html` — all 47 `font-size` declarations lifted to a
  12/13/14/16px scale (10→12, 11/11.5→13, 12/12.5/13→14, 14→16); an embedded
  data-URI SVG favicon in `<head>` and an inline SVG `.logo-icon` in the header
  replacing the plain `.rust-dot`.
- `src/report/tests.rs` — the self-containment assertion now allows the single
  `data:` favicon `<link>` (the old test banned every `<link>`); added
  `renders_brand_logo_and_favicon`.
- Updated [[architecture]] (report generator branding paragraph), [[index]]
  (feature row and source document), and this log.

The brief targets a Tailwind build (`text-xs`/`text-sm`) and a `<h1>` header
that this Stitch-export template does not have; the intent was translated to the
file's own px scale and header structure, keeping the report self-contained
(the favicon namespace is percent-encoded and the logo omits `xmlns`, so no
literal URL scheme remains). The brief's `doc/raw/` wiki path is again mapped to
`doc/wiki/` per repository rule.

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 90 passed (89 + 1 branding); a generated report is
self-contained (0 URL-scheme refs, exactly 1 `<link>`, 1 logo, no font below
12px); OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-17 — UI font & branding draft PR opened

Opened draft PR #22 (`feat/ui-branding-fonts` → `development`) and recorded the
link in the UI-font-branding implementation report's MR section.

Validation: OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-17 — CLI progress & report refinements (issue #21 follow-up)

Applied `doc/raw/2026-10-17/2026-10-17-issue21-fix.md` on the existing
`feat/ui-branding-fonts` branch (folded into PR #22):

- Added the feature subfolder `features/report-ux-refinements/` with
  [[report-ux-refinements]] (feature page) and its implementation report.
- `src/progress.rs` — all bars share one `indicatif::MultiProgress`;
  `ScanProgress::println` writes with the bars suspended, and `ProgressWriter`
  (a `tracing_subscriber::fmt::MakeWriter`) flushes log records under the same
  suspend, so no orphan `Scanning repositories` frames remain.
- `src/main.rs` — the reporter is built before `init_tracing` and threaded into
  `run_scan`; `report_repositories` prints through `ScanProgress::println`.
- `templates/report.html` — cards render collapsed by default; the collapsed
  header keeps chevron/pill/name/counts only; the manifest path and `Update All`
  move to a new `.project-deps-header` in the expanded body; the chevron is
  enlarged; `updateCollapseBtnText()` runs on load.
- `src/report/mod.rs` — rows sorted by risk severity then name;
  `src/report/tests.rs` — updated the combined-command expectation and added
  sort/collapsed/clean-header tests.
- Updated [[architecture]] (progress and report-generator paragraphs),
  [[index]] (feature row and source document), [[cli-scanning-ux]] /
  [[ui-font-branding]] (follow-up links), and this log.

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 94 passed (90 + 4); a pty run reaches `3/3` with no
leftover bar frame; non-TTY stdout has 0 escape sequences and JSON stays pure;
OKF validator PASSED (0 errors, 0 warnings).

## 2026-10-19 — Milestone 0.3.0: per-library AI changelog summarizer

Applied `doc/raw/2026-10-19/2026-10-19-v0.3.0.md` on
`feat/ai-changelog-summarizer` (version `0.3.0-dev`):

- Added the feature subfolder `features/ai-changelog-summarizer/` with
  [[ai-changelog-summarizer]] and its implementation report.
- `src/config/mod.rs` — layered BYOK configuration (user
  `~/.config/rustydeps/config.json`, project `.rustydeps.json` fallback,
  `RUSTYDEPS_AI_*`, CLI flags), provider
  defaults and native key fallbacks, cache path, and validation errors.
- `src/ai/client.rs`, `src/ai/prompt.rs`, `src/ai/cache.rs` — one
  OpenAI-compatible client (OpenAI/Gemini/Ollama, 10s timeout), the fixed
  four-section prompt, and per-version JSON caching under
  `~/.cache/rustydeps/changelogs/`.
- `src/ai/summary.rs` — cache-first, four-way concurrent `Summarizer` for every
  outdated resolved package; the Phase 3 templated summaries are removed.
- `src/cli.rs`, `src/main.rs` — `--no-ai` and `--ai-model`, layered settings
  loading, an AI spinner, and a compact analysis/cache tally.
- `src/classifier/mod.rs` — `ClassifiedDependency.ai_cached`.
- `src/report/markdown.rs`, `src/report/mod.rs`, `templates/report.html` — safe
  Markdown rendering, the 🤖 AI card, `⚡ Cached analysis` badge, fallback note,
  and per-ecosystem changelog links.
- `README.md`, `.gitignore`, `Cargo.toml`/`Cargo.lock` — BYOK docs, ignored
  local config, `pulldown-cmark`, and the `0.3.0-dev` version.
- Updated [[architecture]] (CLI, report generator, AI module, DTOs),
  [[roadmap]] (Phase 6 complete), [[html-report-spec]] (AI card and drawer), and
  [[index]] (feature row and source document).

Validation: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
warnings` clean; `cargo test` 122 passed; `cargo build --release` succeeded; an
offline JSON scan and an HTML scan both completed with `ai_summary: null`.
A local OpenAI-compatible mock served 19 analyses; a second scan made no new
requests and rendered 19 `⚡ Cached analysis` badges. OKF validator PASSED
(0 errors, 0 warnings). Opened draft PR
[#24](https://github.com/AlexDevsTheWeb/rustydeps/pull/24).






