# Development Report — 2026-10-17T070740Z

> RustyDeps — CLI progress rendering fix and HTML report UX refinements
> (issue #21 follow-up). Source: `doc/raw/2026-10-17/2026-10-17-issue21-fix.md`.
> Raw documents are immutable; a follow-up supersedes this with a new dated
> file.

## Summary

Work on the existing `feat/ui-branding-fonts` branch (folded into draft PR #22,
per the maintainer's request to update that PR). Two areas: the CLI progress bar
now writes status lines and logs with the bars suspended (no more orphan frames),
and the report cards are refined — collapsed by default, clean headers, path and
`Update All` moved into the body, and rows sorted by risk.

Commits on the branch:

- `fix(cli): stop the progress bar from leaving orphan frames`
- `feat(report): collapse cards by default and sort rows by risk`
- `docs(raw): add the issue 21 refinements brief`
- `docs(wiki)` / development report (this file)

## Changes made

- **`src/progress.rs`** — one shared `indicatif::MultiProgress`;
  `ScanProgress::println` suspends the bars around a stdout line;
  `ProgressWriter` (`MakeWriter`) buffers each tracing record and flushes it
  under the same suspend on drop; `hidden_progress_bars_never_draw` test added.
- **`src/main.rs`** — reporter created before `init_tracing`, passed into
  `run_scan`; `report_repositories` uses `ScanProgress::println`.
- **`templates/report.html`** — default `collapsed` card; clean collapsed header;
  `.project-deps-header` with the path + `Update All`; larger chevron;
  `updateCollapseBtnText()` on load.
- **`src/report/mod.rs`** — `severity_rank` + per-group sort (severity, then
  name).
- **`src/report/tests.rs`** — updated the `update_all_command` order expectation;
  added severity-sort, collapsed-default and clean-header tests.
- **Wiki** — `doc/wiki/features/report-ux-refinements/`, plus `index.md`,
  `log.md`, `architecture.md`, and follow-up links in `cli-scanning-ux` /
  `ui-font-branding`.

## Tests executed

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — **94 passed**, 0 failed (90 + 4).
- `cargo build --release`; pty run over a 3-project fixture — counter reaches
  `3/3`, no leftover `Scanning repositories` frame after the summary, log lines
  appear above the bar.
- Non-TTY run — 0 escape sequences on stdout; `--format json` parses as one
  JSON array.
- Generated report — cards collapsed, one `.project-deps-header` per card, dots
  ordered breaking → risk → safe.

## Technical debt

- **No visual-regression harness** for card layout/hover; structural tests only.
- **Log lines still interleave between bar frames** (intended) — improved, not
  eliminated.
- **Discovery is still an indeterminate spinner**: an `ignore` walk cannot report
  a total up front, so the `X/N` counter runs over discovered manifests.

## Deviations from the brief

- **Branch/PR.** The brief asks for `feat/cli-and-report-refinements` + a new
  draft PR; at the maintainer's explicit direction the work was folded into the
  existing `feat/ui-branding-fonts` branch / PR #22.
- **Alpine.js.** The brief's item 1 mentions an Alpine.js `open: false`; this
  template uses vanilla JS and a `collapsed` class, so the default was set
  server-side on the card element and the toggle label synced on load.
- **TTY check.** The brief says `!stdout().is_terminal()`; bars draw to stderr,
  so the check is on stderr. stdout being piped while stderr is a terminal still
  yields clean, suspended status lines.
- **`Unknown` severity.** The brief lists four groups; `Unknown` sorts after
  `Up-To-Date`.

## Next recommended actions

1. Review the updated draft PR #22:
   <https://github.com/AlexDevsTheWeb/rustydeps/pull/22>.
2. Open the generated report in a browser to confirm the collapsed cards, the
   body header and the risk ordering visually.
3. Merge PR #22, then publish the `v0.2.0` tag.
