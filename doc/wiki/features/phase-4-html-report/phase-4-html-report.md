---
title: "Phase 4 — Interactive HTML Report Engine"
type: feature
description: "Renders the classified dependency set as a single self-contained, interactive HTML report and writes it from `scan`."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [feature, phase-4, report, html, ui]
---

# Phase 4 — Interactive HTML report engine

## Motivation

Phases 1–3 produced a classified dependency set but only printed it as a table
or JSON. The product's user-facing output is an interactive HTML report; Phase 4
builds that engine. It consumes `Vec<ClassifiedDependency>`, groups the data by
project, and renders a single self-contained file that opens offline and
matches the design contract in [[html-report-spec]].

## What shipped

- **Report engine (`src/report/`)** — `ReportData`/`ProjectReport`/
  `DependencyReport`, built from the classified set by
  `ReportData::from_classified`, plus `render_html`.
- **Askama template (`templates/report.html`)** — the Google Stitch "UI v4"
  design export, with its static mock rows replaced by Askama loops over the
  data; embedded CSS and vanilla JavaScript.
- **CLI wiring (`src/cli.rs`, `src/main.rs`)** — `scan --format html` writes the
  report to `--output` (default `rustydeps-report.html`) and prints a `file://`
  URL to open it.

## Components

### Report model (`src/report/mod.rs`)

`from_classified` groups dependencies by manifest path (each manifest becomes a
project card), derives the project name from the manifest's directory, and
computes a risk tally per project and overall. Each row is flattened into
display-ready fields so the template holds no logic: `status`
(`breaking`/`risk`/`safe`/`up-to-date`/`unknown`, consumed by the filter JS),
`dot_class` and `risk_label` (`major`/`minor`/`patch`/…), the latest-version
display, the copyable update command, a unique drawer id, the semver bump badge,
and the insight (the AI summary, or a type-specific default).

Update commands are per ecosystem — `cargo update -p <name> --precise <latest>`
and `npm install <name>@<latest>` — emitted only for `Safe`/`AtRisk`/`Breaking`
updates. Each project also exposes a combined **Update All** command joined with
` && `.

### Template (`templates/report.html`)

The template is the Google Stitch "UI v4" export. Its CSS and JavaScript are
kept verbatim (they define the design) and its static mock rows are replaced by
Askama loops. It implements [[html-report-spec]]: the Zinc industrial dark
palette (`#09090b`/`#18181b`/`#222226`), monospace package names and version
jumps (`1.35.1 → 2.0.0`), solid 6px risk dots (red/amber/emerald) with no
filled pills, and a discrete grey `AI` prefix on the one-line insight. Each
project card is an accordion with an ecosystem pill, the manifest path, a risk
breakdown, and an **Update All** button that opens a batch-command modal. Each
dependency row expands an inline drawer holding the summary and a
copy-to-clipboard update command. A sticky controls strip provides a search
input (`/` shortcut, `Esc` to clear), risk filter tabs (`All`, `Breaking`,
`At-Risk`, `Safe`), and a Collapse All action.

Everything is inline — no CDN, no external stylesheet, no external script, no
framework — and the exported Google Fonts links are removed (the font stacks
already fall back to system fonts), so the file opens straight from disk.

### CLI output (`src/cli.rs`, `src/main.rs`)

`--format html` (the default) writes the rendered report to `--output`
(default `rustydeps-report.html`, parent directories created as needed) and
prints the path plus a `file://` URL. The human-readable table still prints to
stdout. `--format json` is unchanged and, as before, prints no banner.

## Stitch template integration

The brief `doc/raw/2026-10-15/opencode-prompt-stitch-integration.md` supplied
the raw export at `src/templates/report_stitch.html` (kept as the design
source). Adopting it meant wiring the mock markup to the data model and filling
the gaps the export left:

- **Dynamic loops.** Static rows (`tokio`, `backend-api`, …) are replaced by
  loops over `projects` and their `dependencies`; drawer ids
  (`drawer-<project>-<row>`) are generated per row.
- **Missing CSS.** The export referenced `#emptyState`/`.empty-state-*`, the
  batch modal (`.modal-*`, `.btn-primary`/`.btn-secondary`), `.semver-badge`,
  `.migration-step`, and `.terminal-prompt-prefix` but shipped no rules for
  them; a supplemental stylesheet defines them in the same visual language.
- **Missing JavaScript.** `openBatchModal`/`closeBatchModal`/
  `copyBatchModalCommand`/`handleBackdropClick` and `resetAllFilters` are
  implemented, and the empty state is toggled inside the existing
  `applyFilters`. Buttons carry their payload in `data-*` attributes rather than
  inline string arguments.
- **Self-containment.** The Google Fonts `<link>` tags are removed.

Not fabricated: the export's extended breaking-change bullets, migration steps,
and GitHub release/diff links need changelog data (Phase 5); the drawer shows
the available summary and the update command. The export's PyPI and Go cards are
mock data — RustyDeps supports Cargo and npm today (Phase 6 adds more).

> Milestone 0.3.0 added the per-library AI card and a standard package-changelog
> link to this drawer; see [[ai-changelog-summarizer]].

## Design decisions

- **Askama** (compile-time, auto-escaping) over Tera/maud: type-safe templates
  with escaping handled for untrusted package names and summaries.
- The Stitch export is the design source of truth: the template keeps its CSS
  and JavaScript verbatim and only replaces the mock body with loops.
- Export gaps are completed (styled and wired) rather than shipped broken; data
  the export mocked but the scanner does not produce is not fabricated.
- All display logic lives in `ReportData`, not the template, so the report is
  unit-testable without parsing HTML for structure.
- The report is fully self-contained, verified by a test that asserts no
  `http(s)://` reference and no external `<script src>`/`<link>`.
- `templates/**/*.html` and `src/templates/**/*.html` are allow-listed in
  `.gitignore` so the source template and design export are tracked despite the
  blanket `*.html` ignore of generated output.

## Verification

`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` (57 tests) all pass. A live scan produced a ~55 KB self-contained
report (the Stitch design with real data), with no external references, all
drawer ids resolving, and every JS handler defined. See the
[implementation report](implementation-report.md).

## Related

- [[html-report-spec]] — the design contract this page implements.
- [[architecture]] — the report module's place in the pipeline.
- [[roadmap]] — Phase 4 of six.
- [implementation report](implementation-report.md) — file-by-file detail.
- [[index]] — knowledge-graph entrypoint.
