---
title: "Implementation Report — v0.3.1 Surgical Fixes"
type: report
description: "File-by-file delivery report for cache clear/no-cache, Ollama preflight, cached-model report badges, and wiremock integration tests."
status: done
version: 0.3.1
sources: [2026-10-21-0.3.1]
license: CC-BY-4.0
created: 2026-10-21
updated: 2026-10-21
tags: [implementation-report, ai, cache, ollama, report, testing]
---

# Implementation Report — v0.3.1 Surgical Fixes

## Summary

Implemented `doc/raw/2026-10-21/2026-10-21-0.3.1.md` on
`feat/v0.3.1-surgical-fixes`. The CLI can now clear the AI cache or bypass it
for one scan, an Ollama daemon is probed once before analysis, the report names
the model behind each cached badge, and `wiremock` integration tests cover the
200/429/404 provider paths plus the offline-daemon preflight.

## Done

- **`src/lib.rs`** — new library target exposing all modules; `src/main.rs` now
  consumes `rustydeps::*` instead of owning a duplicate module tree. Required for
  `tests/` integration coverage.
- **`Cargo.toml`** — version `0.3.1-dev`; `wiremock = "0.6"` dev-dependency.
- **`src/cli.rs`** — `cache clear` subcommand and `--no-cache` scan flag, with
  parsing tests.
- **`src/ai/cache.rs`** — `SummaryCache::clear()` removing every cache file and
  returning the count; missing directories yield zero; temp-root tests.
- **`src/config/mod.rs`** — `AiOverrides::no_cache` sets `force_refresh` while
  leaving `cache_enabled` intact, so fresh analyses still refresh the cache;
  resolution test.
- **`src/main.rs`** — dispatches `cache clear`, prints
  `✔ Cleared X cached AI changelog entries.`, and forwards `--no-cache`.
- **`src/ai/client.rs`** — `ollama_origin()`, `ollama_ready()` (1-second
  `/api/tags` probe), and `ollama_tags_url()` path normalization; tests.
- **`src/ai/summary.rs`** — preflight before task spawning; one actionable
  warning and breaker trip when the daemon is offline; cache hits still apply;
  records `ai_model` for fresh and cached summaries.
- **`src/classifier/mod.rs`**, **`src/report/mod.rs`**,
  **`templates/report.html`** — `ai_model` provenance; badge renders
  `⚡ Cached (<model>)`.
- **`tests/ai_integration_tests.rs`** — 200 parse, 429 breaker, 404 fatal, and
  offline Ollama tests.
- **`README.md`** — cache commands, `--no-cache`, model badge, Ollama preflight.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --all-targets` — 144 unit + 4 integration passed.
- `cargo build --release` — success.
- `XDG_CACHE_HOME=<temp> rustydeps cache clear` — removed two seeded entries and
  printed the documented summary.
- `rustydeps scan ~/CODE/CES/babette --no-cache` — regenerated 21 analyses with
  local Ollama and reported zero cache hits.
- A one-package temp project with `XDG_CACHE_HOME=<temp> --no-cache` produced one
  fresh analysis and wrote one cache entry tagged `llama3.2:3b`.
- An unreachable Ollama endpoint logged exactly one preflight warning and served
  21 analyses from cache without chat calls.
- OKF validator — PASSED.

## Technical debt

- The Ollama probe is hard-coded to the daemon root even for custom Ollama
  proxies; a proxy that does not expose `/api/tags` will be treated as offline.
- `cache clear` removes files only; nested directories are left untouched.
- The library target increases compile units; the binary and tests now share one
  crate instead of duplicating modules.

## Next actions

- Open the draft PR to `development`.
- Consider a `cache stats` command and a `--no-cache` scoped to one package.
