---
title: "Report UX & CLI Progress Refinements"
type: feature
description: "Removes orphan progress-bar frames from the scan output and refines the report cards: collapsed by default, clean headers, path + Update All inside the body, and rows sorted by risk."
status: active
version: 0.2.0
license: CC-BY-4.0
created: 2026-10-17
updated: 2026-10-17
tags: [feature, report, ui, cli, progress, ux]
---

# Report UX & CLI progress refinements

## Motivation

A review of real scans (issue #21 follow-up,
`doc/raw/2026-10-17/2026-10-17-issue21-fix.md`) surfaced two rough edges before
v0.2.0:

1. **CLI.** The `Scanning repositories [bar] X/N` counter left duplicate frames
   in the terminal scrollback: the per-repository lines were printed with
   `println!` while the bar was live, and tracing records drawn to stderr during
   a bar did the same.
2. **Report.** Every project card opened expanded and showed its long filesystem
   path and an `Update All` button in the header row, which read as clutter; and
   rows were unsorted, so a card did not lead with its riskiest updates.

## What shipped

### CLI — clean, in-place progress

- All progress bars now share a single `indicatif::MultiProgress`.
- `ScanProgress::println` writes a line with the bars **suspended** — cleared,
  written, then redrawn — so the counter advances in place and the summary is
  never buried under orphan frames.
- A `ProgressWriter` (`tracing_subscriber::fmt::MakeWriter`) buffers each log
  record and flushes it under the same suspend on drop, so log lines no longer
  leave a stray bar frame either.
- A disabled reporter (`--format json` or stderr not a TTY) still returns
  `ProgressBar::hidden()` and writes straight through, so CI/JSON output is
  unchanged.

### Report — compact, focused cards

- Cards render **collapsed by default**; the global toggle is synced to
  "Expand All" on load.
- The **collapsed header** keeps only the chevron, ecosystem pill, project name,
  and risk counts.
- The **filesystem path and `Update All`** move into a `.project-deps-header`
  inside the expanded body.
- The accordion chevron is enlarged and brightened, with the header's existing
  hover affordance kept.
- Rows are **sorted by risk severity** (Breaking → At-Risk → Safe → Up-To-Date →
  Unknown) and then alphabetically by name.

## Design decisions

- **One `MultiProgress` for the whole scan.** It is the simplest way to make
  *any* terminal write (status line or log) suspend the bars; a per-phase bar
  cannot clear a sibling bar that another phase owns.
- **Log output goes through a writer, not per-call suspends.** Routing every
  `info!`/`warn!` through `mp.suspend` by hand is error-prone; the `MakeWriter`
  does it for the whole subscriber. It buffers per record so one log line is one
  suspend.
- **The TTY check stays on stderr.** The bars draw to stderr, so that is the
  stream that must be interactive; stdout being piped while stderr is a terminal
  still gets clean, suspended status lines on stdout.
- **`Unknown` sorts last.** The brief names four severity groups; `Unknown` sits
  after `Up-To-Date` so unresolved lookups never outrank a real update.

## Known limitations

- **Visual verification is manual.** Card layout and hover states are checked by
  opening a generated report; automated coverage is structural (collapsed
  default, clean header, sort order).
- **The counter is still per-manifest.** `ignore::WalkBuilder` cannot report a
  total before the walk ends, so discovery stays a spinner and the `X/N` counter
  runs over discovered manifests (see [[cli-scanning-ux]]).

## Related

- [[index]] — knowledge-graph entrypoint.
- [[cli-scanning-ux]] — the scan progress this refines.
- [[ui-font-branding]] — the report styling this refines.
- [[html-report-spec]] — the report's visual contract.
- [[architecture]] — the report generator and CLI pipeline.
- [implementation report](implementation-report.md) — file-by-file detail and
  verification.
