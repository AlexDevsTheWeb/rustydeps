# Development report — 2026-11-27T07:14:41Z

## Task

Worktree 1 of `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md`: align
`README.md` with the shipped v0.3.1 release.

## Branch

`fix/readme-badges-v0.3.1` (worktree `../rustydeps-fix-readme`), based on
`development` at `76890f4`.

## Changes

- `README.md` — banner `v0.2.0` → `v0.3.1`; version badge `0.2.0` → `0.3.1`;
  `cargo install --git … --tag v0.3.1`; Docker run/Compose/CI examples use
  `ghcr.io/alexdevstheweb/rustydeps:0.3.1`; tag note lists `0.3.1`, `0.3`,
  `latest`; release example and commands use `v0.3.1`.
- `doc/wiki/features/readme-version-alignment/` — feature page and
  implementation report.
- `doc/wiki/index.md` — feature row and source-document entry.
- `doc/wiki/log.md` — dated entry.

## Tests executed

- `cargo fmt --all --check` — clean (Markdown-only change).
- OKF validator (staged shim) — PASSED, 0 errors / 0 warnings.
- `grep -nE '0\.[0-9]+\.[0-9]+|v0\.[0-9]' README.md` — only v0.3.1 references
  remain.

No Rust source changed, so clippy and the test suite were not rerun.

## Technical debt

`README.md` hard-codes `v0.3.1` in the install and Docker examples; each future
release must update it.

## Next recommended actions

Open the draft PR to `development`, record its link in the wiki log, and merge
after review.
