---
title: "Implementation Report — v0.4.0 Real Registry Changelog Fetcher"
type: report
description: "File-by-file delivery record for release-note grounding, prompt injection, and cache versioning."
status: done
version: 0.4.0
sources: [2026-11-27-v0.4.0-and-fixes]
license: CC-BY-4.0
created: 2026-11-27
updated: 2026-11-27
tags: [implementation-report, ai, changelog, release-notes, v0.4.0]
---

# Implementation Report — v0.4.0 Changelog Fetcher

## Summary

Applied Worktree 3 of `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md` on
`feat/v0.4.0-real-changelog-fetcher`. Outdated packages now get official GitHub
release notes injected into their AI prompt when the registry metadata exposes a
GitHub repository. Every failure path falls back to the previous prompt.

## Done

- **`src/registry/changelog.rs`** (new) — `ChangelogFetcher` resolves a package
  repository from crates.io, npm, PyPI, Packagist, or a GitHub Go module path,
  fetches the matching GitHub release with exact/`v` tag candidates, falls back
  to a bounded release list for workspace-prefixed tags, sanitizes
  and truncates to 1,500 characters, and supports `GITHUB_TOKEN`/`GH_TOKEN`.
  The whole lookup is bounded to 8 seconds; a disabled fetcher makes no requests.
- **`src/registry/mod.rs`** — exports the module.
- **`src/ai/prompt.rs`** — `UpgradeContext::changelog` plus `with_changelog`;
  the user prompt injects a bounded `Official Release Notes / Changelog Excerpt`
  block and marks it as the primary source; the system prompt tells the model to
  prefer supplied facts and never invent CVEs.
- **`src/ai/summary.rs`** — `Summarizer` owns a `ChangelogFetcher`, fetches after
  a cache miss and before the chat request, and exposes `without_changelog` for
  `--offline` and tests.
- **`src/ai/cache.rs`** — `CACHE_FORMAT_VERSION = 2`; `CachedSummary::prompt_version`
  rejects v0.3.x entries as misses.
- **`src/main.rs`** — `--offline` disables release-note fetching.
- **`Cargo.toml`** — version `0.4.0-dev`.
- **`README.md`** — feature bullet and an "Official release-note grounding"
  section covering the pipeline, budgets, token, and cache invalidation.
- New [[v0.4.0-changelog-fetcher]] page and this report; [[architecture]]
  and [[roadmap]] updated.

## Verification

Results recorded in `doc/raw/development-report-20261127T072110Z.md`:

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --lib`
- `cargo test --test ai_integration_tests`
- OKF validator

## Technical debt

- Workspace tags are matched by suffix against the 100 most recent releases, so
  a package whose target is older than that window still resolves to `None`.
- Release notes are not cached separately, so a cache clear refetches them.
- Anonymous GitHub access is limited to 60 requests per hour; large scans need
  `GITHUB_TOKEN` for full coverage.

## Next actions

Open the draft PR to `development` and record its link in the log.
