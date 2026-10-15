# Development Report — 2026-10-15T143317Z

> RustyDeps — auto-open the HTML report in the default browser. Source:
> `doc/raw/2026-10-15/opencode-prompt-auto-open.md`. Raw documents are
> immutable; a follow-up supersedes this with a new dated file.

## Summary

Small CLI change on branch `feat/auto-open` (off `development`). After
`scan --format html` writes the report, the default browser now opens it
automatically; `--no-open` suppresses it, and `--format json` never opens.
Opening is non-fatal: a failure is logged as a warning and the scan still
succeeds.

Commits on the branch:

- `docs(raw): add auto-open report task document`
- `feat(cli): auto-open the HTML report in the default browser`
- `docs(wiki): note the report auto-open flag`
- `docs(raw): add auto-open development report` (this file)

## Modifications

- **`src/cli.rs`** — added the `--no-open` flag to `ScanArgs`, plus two parsing
  tests (default opens; the flag is parsed).
- **`src/main.rs`** — in the `--format html` arm, after printing the path and
  `file://` URL, calls `open::that(&absolute)` when auto-open is enabled;
  info on success, `warn!` on failure; the scan is unaffected.
- **`Cargo.toml`** — added the `open` crate (5.x).
- **`doc/wiki/architecture.md`**, **`doc/wiki/log.md`** — CLI flags/behaviour
  and the log entry.

Design decision: auto-open is the default for HTML output (matching the brief),
with an explicit opt-out (`--no-open`) for headless/CI use; the launch is
best-effort so it never blocks a successful scan.

## Verification

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 59 passed, 0 failed |
| `scan --offline -o a.html --no-open` | printed path, no open attempt |
| `scan --offline -o b.html` (default) | opened the report; logged success |
| `scan --offline --format json` | no banner, no open |
| default path exit code | 0 |
| `python3 validate_okf.py` (staging shim) | **PASSED** — 0 errors, 0 warnings |

## Technical debt

- The failure branch of `open::that` is not exercised by an automated test: on
  macOS a successful launch cannot be forced to fail deterministically. The
  branch is trivial (log a warning) and covered by inspection.
- Auto-open runs by default even in non-interactive environments; the CLI still
  prints the `file://` URL as a manual fallback.

## Next recommended actions

1. If CI ever runs `scan` with HTML output, pass `--no-open` (or set it via the
   default) to avoid launching a browser.
2. Phase 6 (AI Changelog Summarizer) remains the next roadmap phase.
