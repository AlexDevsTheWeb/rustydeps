# Development Report — 2026-10-15T130130Z

> RustyDeps — Phase 4 implemented: the interactive, self-contained HTML report
> engine. Source: `doc/raw/2026-10-15/2026-10-15-phase4.md`. Raw documents are
> immutable; a follow-up supersedes this with a new dated file.

## Summary

Feature work on branch `feat/phase-4`. Added the report engine (Askama template
with embedded CSS/JS) and wired it into `scan`: `--format html` now writes a
single self-contained `rustydeps-report.html`, grouping dependencies by project
with risk dots, an inline drawer, filters, and copyable update commands. The
layout implements the shipped `doc/wiki/html-report-spec.md` contract.

Commits on the branch:

- `docs(raw): add Phase 4 task document`
- `feat(report): render a self-contained interactive HTML report`
- `docs(wiki): document Phase 4 HTML report engine`
- `docs(raw): add Phase 4 development report` (this file)

## Modifications

- **`src/report/mod.rs` (new)** — `ReportData`, `ProjectReport`,
  `DependencyReport`, `ReportMeta`, `from_classified`, `render_html`.
- **`src/report/tests.rs` (new)** — self-containment, escaping, grouping,
  commands, risk indicators/filters/empty state, AI tag, empty scan, metadata.
- **`templates/report.html` (new)** — the Askama template (embedded CSS + JS).
- **`src/cli.rs`** — `--output` default `rustydeps-report.html`.
- **`src/main.rs`** — writes the report, prints the path and a `file://` URL,
  measures scan duration.
- **`.gitignore`** — allow-list `templates/**/*.html`.
- **`Cargo.toml`** — added `askama`; tidied dependency order.
- **Wiki** — new `features/phase-4-html-report/`; `architecture.md`,
  `roadmap.md`, `index.md`, `log.md`, `html-report-spec.md`.

Design decisions (rationale in the wiki feature page):

- Askama (compile-time, auto-escaping) over Tera/maud.
- All display logic in `ReportData`, not the template, so rows are unit-tested
  without HTML structure parsing.
- Fully self-contained output, enforced by a test that rejects any
  `http(s)://` / external `<script src>` / `<link>`.
- `.gitignore` allow-lists the source template so the blanket `*.html` rule
  does not hide it.

## Verification

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 56 passed, 0 failed |
| live `scan -o tmp/phase-4/report.html` | ~23 KB self-contained report, 2 project cards, correct commands |
| `grep` external refs | 0 `http(s)://`, 0 external `<script src>`/`<link>` |
| `scan` without `-o` | produced `rustydeps-report.html` |
| `python3 validate_okf.py` (staging shim) | **PASSED** — 0 errors, 0 warnings |

## Technical debt

- Live AI provider summaries are Phase 5; the report renders Phase 3's templated
  summaries.
- `AtRisk` still means "outside the declared range"; deprecation signals need
  changelog data (Phase 5).
- No PDF/CSV export and no diffing between scans; not in the brief.
- Cargo `[build-dependencies]` and npm optional/peer dependencies remain
  unextracted (carried from Phase 2).

## Next recommended actions

1. **Phase 5:** replace the templated summaries with real provider calls and
   changelog fetching; broaden `AtRisk` using deprecation data.
2. **Phase 6:** add PyPI/Go/Composer parsers and clients behind the existing
   `PackageParser` trait; the report should pick them up with no changes beyond
   an ecosystem badge.
3. Consider snapshot-testing the rendered HTML (e.g. an insta-style golden
   file) to catch unintended template drift.
