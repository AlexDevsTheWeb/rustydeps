---
title: "Implementation Report — v0.4.0 README and Landing Page Sync"
type: report
description: "Delivery record for the v0.4.0 README and GitHub Pages version, feature, and install-command sync."
status: done
version: 0.4.0
sources: [2026-11-27-v0.4.0]
license: CC-BY-4.0
created: 2026-11-27
updated: 2026-11-27
tags: [implementation-report, docs, readme, landing-page, v0.4.0]
---

# Implementation Report — v0.4.0 Docs Sync

## Summary

Applied `doc/raw/2026-11-27/2026-11-27-v0.4.0.md` on
`fix/v0.4.0-docs-and-readme-sync`, branched from `development` at `5a25cba`.
Both user-facing documents now describe v0.4.0, its release-note grounding, and
the published container image.

## Done

- **`README.md`** — v0.4.0 banner/badge/install/Docker/release references;
  explicit `docker pull …:0.4.0`; all five grounding sources; `GITHUB_TOKEN`/
  `GH_TOKEN`; `CACHE_FORMAT_VERSION = 2`; `cache clear`; accurate `--offline`
  scope; Docker tag note `0.4.0`/`0.4`.
- **`docs/index.html`** — v0.4.0 title/meta/header/hero/report mock/footer;
  `--tag v0.4.0` and `:0.4.0` copy commands; grounded-briefing feature card;
  release-note and token bullets; cache-format note; v0.4.0 release links.
- **Wiki** — this report and [[v0.4.0-docs-sync]]; index row and source entry;
  log entry; development report in `doc/raw/`.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --all-targets` — 162 unit + 5 integration passed.
- `html.parser` — 0 mismatched/unclosed tags; 155 `<div>` open/close pairs.
- Headless Chromium at 1365×900 and 390×844 — no horizontal overflow; updated
  title, version badges, grounding card, and `GITHUB_TOKEN` text visible.
- `data-copy` count — 15, with the four version-bearing commands on v0.4.0.
- Stale-version grep — no `0.3.1`/`v0.3.1`; only intentional `v0.3.x` cache
  compatibility text.
- External checks — release tag URL HTTP 200; GHCR tag list exposes `0.4.0`
  and `0.4`; Release and Docker workflows for the `v0.4.0` tag succeeded.
- OKF validator — recorded in the development report.

## Deviation

GHCR tags omit the git tag's `v` (`metadata-action` semver output), so the
container examples use `:0.4.0`, not the brief's `:v0.4.0`. Release page links
keep `v0.4.0`.

## Follow-up

`--offline` still lets an enabled AI provider serve cache misses. The README now
documents this and recommends `--no-ai` for hermetic runs. Making `--offline`
cache-only would be a separate behaviour change.

## Merge request

Draft PR opened to `development`; `main` picks it up through the normal
development → main sync.
