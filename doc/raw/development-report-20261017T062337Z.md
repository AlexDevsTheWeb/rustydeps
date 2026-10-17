# Development Report — 2026-10-17T062337Z

> RustyDeps — CLI scanning UX (issue #19) and release preparation for v0.2.0.
> Source: `doc/raw/2026-10-17/2026-10-17-issue19.md`. Raw documents are
> immutable; a follow-up supersedes this with a new dated file.

## Summary

Worked on branch `feat/cli-scanning-ux` (off `development`). Added interactive
progress reporting to `scan` so a large tree no longer looks frozen: spinners
for manifest discovery and registry resolution, a per-repository `pos/len`
counter, and a `✔ Scanned '<path>': N dependencies (<ecosystem>)` line per
manifest. Animations are suppressed when stderr is not a terminal, and
`--format json` additionally suppresses the banner and status lines so stdout
stays a single JSON document. Bumped the crate to 0.2.0 and added `indicatif`.

Commits on the branch:

- `chore(release): prepare v0.2.0 and add indicatif`
- `feat(cli): add interactive progress indicators to scan (#19)`
- `docs(raw): add the issue 19 CLI scanning UX brief`

## Changes made

- **`src/progress.rs` (new)** — `ScanProgress` over `indicatif` 0.18:
  `spinner(message)` and `bar(length, message)`; a disabled reporter returns
  `ProgressBar::hidden()`. Interactivity requires the caller's flag *and*
  `std::io::stderr().is_terminal()` (`IsTerminal` is in std since 1.70, so no
  `atty`/`is-terminal` crate). Pure, unit-tested helpers `project_label` and
  `repository_summary`.
- **`src/main.rs`** — `run_scan` computes `machine_readable`, brackets discovery
  and resolution with spinners, and calls the new `report_repositories`, which
  groups parsed dependencies by manifest, prints a line per manifest and
  advances the counter.
- **`src/scanner.rs`** — `ManifestType::ecosystem_label()` plus a unit test.
- **`Cargo.toml`/`Cargo.lock`** — `version = "0.2.0"`, `indicatif = "0.18"`.
- **`README.md`** — banner, badge, Docker/CI examples and the Releasing tag
  example bumped to `0.2.0`.
- **Wiki** — `doc/wiki/features/cli-scanning-ux/` (feature page +
  implementation report), plus updates to `index.md`, `log.md` and
  `architecture.md`.

## Tests executed

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — **89 passed**, 0 failed (85 before; +3 progress, +1 scanner).
- `cargo build --release` — succeeds; `--version` → `rustydeps 0.2.0`.
- Non-TTY `scan . --offline --no-open` — exit 0, prints
  `✔ Scanned '.': 18 dependencies (Cargo)`, no animation escape codes.
- Multi-project fixture — two distinct per-repository lines.
- `--format json` — stdout parsed as a single JSON array; no banner/status.
- pty run under `script` — spinners and the `0/2 → 2/2` counter observed.

## Technical debt

- **Counter granularity.** `ignore::WalkBuilder` cannot report a total before the
  walk ends, so the `X/N` counter is over repositories reported, not directories
  visited. Subdividing the walk would change `scanner.rs`'s API.
- **No progress snapshot test.** Bar rendering is terminal-dependent; only the
  pure formatting helpers are asserted.
- **Verbose log/spinner interleaving** on stderr is cosmetic and unaddressed.
- **No `--quiet`/`--no-progress` flag.** Interactivity is inferred from the TTY
  and the format; the brief requested no such flag.

## Deviations from the brief

- **Wiki location.** The brief's step 5 places the page under
  `doc/raw/YYYY-MM-DD/cli-scanning-ux.md`. This repository treats `doc/raw/` as
  immutable input, so the page was created under
  `doc/wiki/features/cli-scanning-ux/` per `AGENTS.md` / `raw-ingest-ship-loop`.

## Next recommended actions

1. Review and merge draft PR #20 into `development`:
   <https://github.com/AlexDevsTheWeb/rustydeps/pull/20>.
2. After merge, publish the `v0.2.0` tag (`git tag v0.2.0 && git push origin
   v0.2.0`) so the release workflow produces binaries and the Docker workflow
   publishes the `0.2.0`/`0.2`/`latest` image tags.
3. Consider a `--no-progress`/`--quiet` flag and a walk-level progress hook if
   users ask for finer-grained progress.
