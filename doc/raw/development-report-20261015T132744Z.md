# Development Report — 2026-10-15T132744Z

> RustyDeps — Stitch UI v4 HTML template integrated into the report engine.
> Source: `doc/raw/2026-10-15/opencode-prompt-stitch-integration.md` and the
> design export `src/templates/report_stitch.html`. Raw documents are immutable;
> a follow-up supersedes this with a new dated file.

## Summary

The Phase 4 report template was replaced with the supplied Google Stitch "UI v4"
export. The export's CSS and JavaScript are kept verbatim; its static mock rows
were replaced with Askama loops over the scanned data, the Google Fonts links
were removed to keep the file self-contained, and the parts the export omitted
were completed (a stylesheet for the empty state, batch modal, `semver-badge`,
`migration-step`, and `terminal-prompt-prefix`, plus the missing JS handlers and
the empty-state toggle). Landed on the existing `feat/phase-4` branch and pushed
to draft PR #7, per the developer's choice.

Commits on the branch:

- `docs(raw): add Stitch template integration prompt`
- `feat(report): adopt the Stitch UI v4 HTML template`
- `docs(wiki): document the Stitch UI v4 template integration`
- `docs(raw): add Stitch integration development report` (this file)

## Modifications

- **`src/templates/report_stitch.html` (new)** — the raw Stitch export, tracked
  as the design source; `.gitignore` allow-lists `src/templates/**/*.html`.
- **`templates/report.html` (replaced)** — the export's CSS/JS verbatim + an
  Askama body (loops over projects/dependencies) + a supplemental stylesheet and
  the missing JS handlers.
- **`src/report/mod.rs`** — DTOs reshaped to the design: row
  `status`/`dot_class`/`risk_label`/`is_breaking`/`has_insight`/
  `has_ai_summary`/`insight`/`drawer_id`/`has_bump_badge`/`bump_label`/
  `drawer_title`/`drawer_desc`; project `ecosystem_label`/`run_in`/counts;
  `ReportMeta::scanned_root` dropped.
- **`src/report/tests.rs`** — updated to the new fields; added drawer-id and
  bump-badge coverage.
- **`src/main.rs`** — report metadata no longer passes `scanned_root`.
- **Wiki** — Phase 4 feature page + implementation report updated; `log.md`,
  `architecture.md`, `html-report-spec.md` touched.

Decisions (rationale in the wiki feature page):

- The Stitch export is the design source of truth; CSS/JS stay verbatim.
- Export gaps are completed rather than shipped broken.
- Data the export mocked but the scanner does not produce (changelog bullets,
  migration steps, release links) is not fabricated.

## Verification

| Check | Result |
|-------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 57 passed, 0 failed |
| live `scan -o tmp/phase-4/report-stitch.html` | ~55 KB self-contained Stitch report |
| external references | 0 `http(s)://`, 0 external `<script src>`/`<link>` |
| wiring | 7/7 drawer toggles resolve; all JS handlers defined |
| empty scan | renders "No dependencies found" |
| `python3 validate_okf.py` (staging shim) | **PASSED** — 0 errors, 0 warnings |

## Technical debt

- `templates/report.html` is now the source of truth for the report; the
  one-off assembler used to splice the Stitch export into it ran from
  `tmp/phase-4/` (git-ignored) and is not committed. Regenerating from a future
  Stitch export would mean re-applying the Askama body and the supplemental
  CSS/JS.
- The drawer shows the one-line summary and command only; extended changelog
  content is Phase 5.
- The export's PyPI and Go cards are mock data; RustyDeps supports Cargo and npm
  only (Phase 6).
- No HTML snapshot test yet; current tests assert markers rather than exact
  markup.

## Next recommended actions

1. **Phase 5:** real AI provider calls + changelog fetching to fill the drawer's
   breaking-change bullets and migration steps.
2. Keep `templates/report.html` as the report source of truth; only revisit the
   Stitch export if the design changes.
3. **Phase 6:** PyPI/Go/Composer parsers so the export's other ecosystem cards
   become real.
