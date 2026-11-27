# Development report — 2026-11-27T14:55:38Z

## Task

Apply `doc/raw/2026-11-27/2026-11-27-v0.4.0.md`: sync `README.md` and
`docs/index.html` to v0.4.0.

## Branch

`fix/v0.4.0-docs-and-readme-sync` (worktree
`../rustydeps-fix-v0.4.0-docs`), based on `development` at `5a25cba`.

## Changes

- `README.md` — banner, badge, Cargo `--tag v0.4.0`, Docker run/Compose/CI
  examples on `0.4.0`, explicit `docker pull …:0.4.0`, tag note `0.4.0`/`0.4`,
  release examples `v0.4.0`; AI feature names all five repository sources;
  grounding section documents `CACHE_FORMAT_VERSION = 2`, one-time v0.3.x
  regeneration, and `rustydeps cache clear`; `--offline` scope corrected.
- `docs/index.html` — v0.4.0 title, meta description, header badge, hero pill,
  report mock header, and footer; v0.4.0 Cargo/Docker copy commands; feature
  card 3 rewritten as "Grounded Per-Library AI Briefings"; AI section gains
  release-note and `GITHUB_TOKEN`/`GH_TOKEN` bullets; cache card mentions
  `CACHE_FORMAT_VERSION = 2`; release links use `v0.4.0`.
- `doc/wiki/features/v0.4.0-docs-sync/` — feature page and implementation
  report; `doc/wiki/index.md` and `doc/wiki/log.md` updated.
- This development report.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --all-targets` — 162 unit + 5 integration passed.
- `html.parser` stack check — 0 errors; 155 `<div>` open/close pairs.
- Headless Chromium 1365×900 and 390×844 — `scrollWidth == innerWidth`
  (no horizontal overflow); title, v0.4.0 badges, grounding card, and
  `GITHUB_TOKEN` text render.
- `data-copy` count — 15; the four version-bearing commands use v0.4.0.
- Stale-version grep — no `0.3.1`/`v0.3.1`; only intentional `v0.3.x` cache
  compatibility text.
- Release URL — `releases/tag/v0.4.0` returns HTTP 200.
- GHCR tags — live list contains `0.4.0` and `0.4` (no `v`); Release and Docker
  workflows for `v0.4.0` succeeded.
- OKF validator — PASSED, 0 errors / 0 warnings.

## Technical debt / notes

- GHCR omits the git `v`; container examples use `:0.4.0` while release links
  use `v0.4.0`.
- `--offline` still lets an enabled AI provider serve cache misses; the docs now
  state this and recommend `--no-ai` for a hermetic run.

## Next recommended actions

Open the draft PR to `development`, record its link in the wiki log, then sync
`development` → `main` through the normal release flow.
