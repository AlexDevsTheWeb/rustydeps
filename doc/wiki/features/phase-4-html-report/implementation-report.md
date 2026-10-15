---
title: "Implementation Report — Phase 4 HTML Report Engine"
type: report
description: "File-by-file report of the Phase 4 self-contained HTML report engine, template, and CLI output wiring."
status: done
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [implementation-report, phase-4]
---

# Implementation Report — Phase 4

## Summary

The raw brief `doc/raw/2026-10-15/2026-10-15-phase4.md` asked for an interactive
HTML report engine (Askama/Tera/maud) that renders the classified dependency set
as one self-contained file following the Stitch v4 layout in
[[html-report-spec]]. All shipped on branch `feat/phase-4`: `scan --format html`
now writes `rustydeps-report.html` (or `--output`), and the report groups
dependencies by project with risk dots, an inline drawer, filters, and update
commands.

## Done

- `src/report/mod.rs` (new) — `ReportData`, `ProjectReport`, `DependencyReport`,
  `ReportMeta`, `from_classified` grouping/tallies, and `render_html`.
- `src/report/tests.rs` (new) — self-containment, escaping, grouping, commands,
  risk indicators/filters/empty state, AI tag, empty scan, and metadata.
- `templates/report.html` (new) — the Askama template with embedded CSS/JS
  implementing [[html-report-spec]].
- `src/cli.rs` — `--output` defaults to `rustydeps-report.html`.
- `src/main.rs` — writes the report for `--format html`, prints the path and a
  `file://` URL, measures scan duration, and passes scan metadata.
- `.gitignore` — allow-lists `templates/**/*.html`.
- `Cargo.toml` — added `askama`; tidied the dependency list.
- Wiki: this subfolder plus updates to `architecture.md`, `roadmap.md`,
  `index.md`, `log.md`, and `html-report-spec.md`.

## Not done / deferred

- Live AI provider summaries are Phase 5; the report renders the templated
  summaries produced in Phase 3.
- `AtRisk` still means "a bump outside the declared range"; deprecation signals
  need changelog data (Phase 5).
- The report has no export to PDF/CSV and no per-repository diffing yet; not in
  the Phase 4 brief.
- Cargo `[build-dependencies]` and npm optional/peer dependencies remain
  unextracted (carried from Phase 2).

## Blocked

None. The brief's "Draft PR #5" was stale — PRs #5 and #6 already exist, so this
work opens the next draft PR number.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 57 passed, 0 failed.
- Live `scan -o tmp/phase-4/report-stitch.html` over a Cargo + npm fixture wrote
  a ~55 KB self-contained report using the Stitch design: two project cards
  (`Cargo` backend-api, `npm` web-dashboard), risk dots, drawers, AI prefixes,
  the batch-update modal, the empty state, and correct commands
  (`cargo update -p tokio --precise 1.53.3`, `npm install express@5.3.0`).
- `grep` over the report found zero `http://`/`https://` and zero external
  `<script src>`/`<link>` references; all 7 drawer toggles resolve to their ids
  and every referenced JS handler is defined.
- `scan` without `-o` produced `rustydeps-report.html`; an empty scan renders
  "No dependencies found".

## Follow-up — Stitch template integration

`doc/raw/2026-10-15/opencode-prompt-stitch-integration.md` supplied the raw
Google Stitch "UI v4" export at `src/templates/report_stitch.html`. The report
template was swapped to it:

- `src/templates/report_stitch.html` — the raw design export, tracked as a
  reference input (`.gitignore` allow-lists `src/templates/**/*.html`).
- `templates/report.html` — the export's CSS/JS kept verbatim, the mock body
  replaced by Askama loops, the Google Fonts links removed, plus a supplemental
  stylesheet (empty state, modal, `semver-badge`, `migration-step`,
  `terminal-prompt-prefix`) and the missing JS handlers
  (`openBatchModal`/`closeBatchModal`/`copyBatchModalCommand`/
  `handleBackdropClick`/`resetAllFilters`, and the empty-state toggle).
- `src/report/mod.rs` — the DTOs reshaped to the design's fields (row
  `status`/`dot_class`/`risk_label`/`is_breaking`/`insight`/`has_ai_summary`,
  drawer id and bump badge; project `ecosystem_label`/`run_in`).
- `src/report/tests.rs` — updated for the new fields.
- `src/main.rs` — `ReportMeta` no longer carries `scanned_root` (the Stitch
  header has no slot for it).

Deferred as before: the drawer's extended changelog bullets, migration steps,
and release links need Phase 5 changelog data and are not fabricated.

## MR

Draft PR #7 into `development` from `feat/phase-4`:
<https://github.com/AlexDevsTheWeb/rustydeps/pull/7>.

## Related

- [[phase-4-html-report]] — the feature page for this work.
- [[html-report-spec]] — the design contract.
- [[architecture]] — module design.
- [[roadmap]] — Phase 4 of six.
