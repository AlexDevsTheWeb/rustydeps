---
title: "Implementation Report — Report UX & CLI Progress Refinements"
type: report
description: "File-by-file report of the MultiProgress/suspend CLI fix and the report card refinements (collapsed default, clean header, risk sort)."
status: done
version: 0.2.0
sources: [issue21]
license: CC-BY-4.0
created: 2026-10-17
updated: 2026-10-17
tags: [implementation-report, report, ui, cli, progress]
---

# Implementation Report — Report UX & CLI Progress Refinements

## Summary

`doc/raw/2026-10-17/2026-10-17-issue21-fix.md` is the issue #21 follow-up: fix
the CLI progress-bar artifacts and refine five report-card behaviours. Shipped on
the existing `feat/ui-branding-fonts` branch (folded into PR #22). The CLI now
writes status lines and logs with the progress bars suspended, and the report
renders collapsed, clean cards with risk-sorted rows.

## Done

- **`src/progress.rs`** — `ScanProgress` now owns a shared
  `indicatif::MultiProgress`; `spinner`/`bar` add to it and
  `ScanProgress::println` writes with the bars suspended. New
  `ProgressWriter` implements `tracing_subscriber::fmt::MakeWriter`, buffering
  each record and flushing it under `MultiProgress::suspend` on drop. Disabled
  runs still use `ProgressDrawTarget::hidden()`.
- **`src/main.rs`** — the reporter is created before `init_tracing` and passed to
  `run_scan`; `init_tracing` takes the writer; `report_repositories` prints via
  `ScanProgress::println`.
- **`templates/report.html`** — `project-card collapsed` default; the collapsed
  header drops the path and the `Update All` button; new `.project-deps-header`
  holds the path (`title` = full path, ellipsised) and the button; the
  `.accordion-toggle` chevron is 14px and brighter; `updateCollapseBtnText()`
  runs on load.
- **`src/report/mod.rs`** — `severity_rank` and a stable
  `sort_by(severity then name)` over each project's rows.
- **`src/report/tests.rs`** — combined-command expectation updated for the new
  order; added `sorts_dependencies_by_severity_then_name`,
  `renders_project_cards_collapsed_by_default`,
  `keeps_collapsed_header_clean_and_moves_path_and_action_into_the_body`;
  `progress.rs` gained `hidden_progress_bars_never_draw`.
- **Wiki** — this subfolder, [[report-ux-refinements]], and updates to
  [[index]], [[log]], [[architecture]].

## Not done / deferred

- **No visual-regression harness** for the card layout, hover affordance or
  chevron; verified structurally and by generating a report.
- **Verbose/log interleaving is improved, not eliminated.** Logs now suspend the
  bars, but a log line still appears inline between frames; that is intended.
- **`Unknown` rows sort last** though the brief lists only four groups — a
  deliberate choice, recorded on the feature page.

## Blocked

None.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 94 passed, 0 failed (90 + 4).
- pty run over a 3-project fixture: the counter reaches `3/3` and no
  `Scanning repositories` frame remains after the summary; log lines appear
  above the bar.
- Non-interactive run: 0 escape sequences on stdout; `--format json` parses as a
  single JSON array.
- Generated report: cards collapsed, one `.project-deps-header` per card, dots
  ordered breaking → risk → safe.
- OKF validator PASSED (0 errors, 0 warnings).

## MR

Folded into draft PR #22 (`feat/ui-branding-fonts` → `development`):
<https://github.com/AlexDevsTheWeb/rustydeps/pull/22>.

## Related

- [[report-ux-refinements]] — the feature page for this work.
- [[cli-scanning-ux]] — the scan progress this refines.
- [[ui-font-branding]] — the report styling this refines.
