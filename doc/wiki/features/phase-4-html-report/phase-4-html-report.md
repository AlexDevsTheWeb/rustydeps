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
- **Askama template (`templates/report.html`)** — the whole report: embedded
  CSS, embedded vanilla JavaScript, and the Stitch v4 layout.
- **CLI wiring (`src/cli.rs`, `src/main.rs`)** — `scan --format html` writes the
  report to `--output` (default `rustydeps-report.html`) and prints a `file://`
  URL to open it.

## Components

### Report model (`src/report/mod.rs`)

`from_classified` groups dependencies by manifest path (each manifest becomes a
project card), derives the project name from the manifest's directory, and
computes a risk tally per project and overall. Each row is flattened into
display-ready fields so the template holds no logic: `risk_class`
(`breaking`/`at-risk`/`safe`/`up-to-date`/`unknown`), `risk_label`
(`major`/`minor`/`patch`/…), the latest-version display, the copyable update
command, and the AI summary.

Update commands are per ecosystem — `cargo update -p <name> --precise <latest>`
and `npm install <name>@<latest>` — emitted only for `Safe`/`AtRisk`/`Breaking`
updates. Each project also exposes a combined **Update All** command joined with
` && `.

### Template (`templates/report.html`)

Implements [[html-report-spec]]: the Zinc industrial dark palette
(`#09090b`/`#18181b`/`#27272a`), monospace package names and version jumps
(`1.35.1 → 2.0.0`), solid 6px risk dots (red/amber/emerald) with no filled
pills, and a discrete grey `AI` tag beside the one-line summary. Each project
card carries ecosystem badges, the manifest path, a risk breakdown, and the
**Update All** action. Each dependency row is a `<details>` drawer holding the
summary and a copy-to-clipboard update command. A sticky controls bar provides a
search input (`/` shortcut) and risk filter pills (`All`, `Breaking`, `At-Risk`,
`Safe`), with an empty state that appears when a filter matches nothing.

Everything is inline: no CDN, no external stylesheet, no external script, no
framework — the file opens straight from disk.

### CLI output (`src/cli.rs`, `src/main.rs`)

`--format html` (the default) writes the rendered report to `--output`
(default `rustydeps-report.html`, parent directories created as needed) and
prints the path plus a `file://` URL. The human-readable table still prints to
stdout. `--format json` is unchanged and, as before, prints no banner.

## Design decisions

- **Askama** (compile-time, auto-escaping) over Tera/maud: type-safe templates
  with escaping handled for untrusted package names and summaries.
- All display logic lives in `ReportData`, not the template, so the report is
  unit-testable without parsing HTML for structure.
- The report is fully self-contained, verified by a test that asserts no
  `http(s)://` reference and no external `<script src>`/`<link>`.
- `templates/**/*.html` is allow-listed in `.gitignore` so the source template
  is tracked despite the blanket `*.html` ignore of generated output.

## Verification

`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` (56 tests) all pass. A live scan produced a ~23 KB self-contained
report with two project cards, correct commands, and no external references.
See the [implementation report](implementation-report.md).

## Related

- [[html-report-spec]] — the design contract this page implements.
- [[architecture]] — the report module's place in the pipeline.
- [[roadmap]] — Phase 4 of six.
- [implementation report](implementation-report.md) — file-by-file detail.
- [[index]] — knowledge-graph entrypoint.
