---
title: "Implementation Report — CLI Scanning UX & v0.2.0"
type: report
description: "File-by-file report of the interactive scan progress, per-repository metrics, non-TTY/JSON fallback, and the v0.2.0 version bump."
status: done
version: 0.2.0
sources: [issue19]
license: CC-BY-4.0
created: 2026-10-17
updated: 2026-10-17
tags: [implementation-report, cli, ux, progress, release]
---

# Implementation Report — CLI Scanning UX

## Summary

`doc/raw/2026-10-17/2026-10-17-issue19.md` implements issue #19 ("Improvement:
review CLI interface when scanning") and prepares release v0.2.0. Shipped on
branch `feat/cli-scanning-ux`: a new `src/progress.rs` wrapping `indicatif`,
interactive spinners and a per-repository counter wired into `scan`, a
per-manifest dependency line, a non-TTY/JSON fallback, and the 0.2.0 version
bump across `Cargo.toml` and the README.

## Done

- **`src/progress.rs` (new)** — `ScanProgress` with `spinner(message)` and
  `bar(length, message)`; a disabled reporter returns `ProgressBar::hidden()`.
  Interactivity needs the caller's flag *and* `std::io::stderr().is_terminal()`.
  Pure helpers `project_label` and `repository_summary` (both unit tested).
- **`src/main.rs`** — `run_scan` computes `machine_readable = format == Json`,
  builds a `ScanProgress`, brackets discovery and resolution with spinners, and
  calls `report_repositories` (new) to print one `✔ Scanned ...` line per
  manifest while advancing the counter. For JSON, the banner and the
  per-repository lines are skipped; the bars draw to stderr regardless.
- **`src/scanner.rs`** — `ManifestType::ecosystem_label()` returning
  `Cargo`/`npm`/`PyPI`/`Go`/`Composer`, matching the report's project pill,
  plus a unit test.
- **`Cargo.toml` / `Cargo.lock`** — `version = "0.2.0"` and
  `indicatif = "0.18"`.
- **`README.md`** — banner, version badge, Docker image tags, CI snippets and
  the Releasing tag example updated to `0.2.0`.
- **Wiki** — this subfolder, [[cli-scanning-ux]], and updates to [[index]],
  [[log]], and [[architecture]].
- **`doc/raw/2026-10-17/2026-10-17-issue19.md`** — the immutable source brief
  committed verbatim.

## Not done / deferred

- **No `--quiet`/`--no-progress` flag.** Interactivity is inferred from the TTY
  and the format; the brief asked for neither flag, so none was added. A user
  who wants a barless interactive run can redirect stderr.
- **The counter does not measure directory traversal.** `ignore::WalkBuilder`
  cannot report a total before the walk ends, so `X/N` counts repositories
  reported, not directories visited. Instrumenting the walk itself would change
  `scanner.rs`'s public API and was not needed to fix the "frozen" symptom.
- **Verbose log/spinner interleaving.** With `-v`, `tracing` lines share stderr
  with the spinner and can momentarily overwrite a frame. Cosmetic only.
- **No snapshot/integration test of the rendered bars.** Progress rendering is
  terminal-dependent and not asserted; the pure formatting helpers are.

## Blocked

None.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 89 passed, 0 failed (85 before; +3 progress, +1 scanner).
- `cargo build --release` — succeeds; `./target/release/rustydeps --version`
  prints `rustydeps 0.2.0`.
- `scan . --offline --no-open` on the repository itself — exit 0, prints
  `✔ Scanned '.': 18 dependencies (Cargo)`, writes a report; no animation
  escape codes on a non-TTY stdout.
- Multi-project fixture (`mono/backend/Cargo.toml`, `mono/web/package.json`) —
  prints `✔ Scanned 'mono/backend': 2 dependencies (Cargo)` and
  `✔ Scanned 'mono/web': 1 dependency (npm)`.
- `--format json` — stdout parses as a single JSON array (3 entries for the
  fixture) with no banner or status lines.
- pty run under `script -q /dev/null` — shows the "Discovering dependency
  manifests…" and "Parsing manifests…" spinners and the
  `Scanning repositories [bar] 0/2 → 2/2` counter.
- OKF validator PASSED (0 errors, 0 warnings).

## MR

Draft PR #20 into `development` from `feat/cli-scanning-ux`:
<https://github.com/AlexDevsTheWeb/rustydeps/pull/20>.

## Related

- [[cli-scanning-ux]] — the feature page for this work.
- [[architecture]] — the CLI pipeline this change instruments.
- [[release-automation]] — the pipeline that will publish v0.2.0.
