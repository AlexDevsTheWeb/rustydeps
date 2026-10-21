---
title: "Development Report — Milestone 0.3.0 Per-Library AI Summarizer"
type: report
description: "Changes, verification, technical debt, and next actions for the granular AI changelog summarizer with BYOK config and caching."
status: done
version: 0.3.0
sources: [2026-10-19-v0.3.0]
license: CC-BY-4.0
created: 2026-10-19
updated: 2026-10-19
tags: [development-report, ai, changelog, byok, cache, report]
---

# Development Report — Milestone 0.3.0

Timestamp: `20261019T160629Z`
Branch: `feat/ai-changelog-summarizer` (off `development`)
Version: `0.3.0-dev`

## Changes made

- **Configuration** (`src/config/mod.rs`): layered BYOK config from CLI,
  `RUSTYDEPS_AI_*`, project `.rustydeps.json`, user
  `~/.config/rustydeps/config.json`, then defaults. Supports OpenAI, Gemini,
  and Ollama, provider-native key fallbacks, `max_tokens` clamping, and cache
  toggling. Invalid input disables AI without failing the scan.
- **AI pipeline** (`src/ai/`): `client.rs` (OpenAI-compatible chat completions,
  10-second timeout), `prompt.rs` (four-section per-library prompt),
  `cache.rs` (per-version JSON entries under
  `~/.cache/rustydeps/changelogs/`), and `summary.rs` (cache-first, four-way
  concurrent analysis for every outdated resolved package). The Phase 3
  templated summaries were removed.
- **CLI** (`src/cli.rs`, `src/main.rs`): added `--no-ai` and `--ai-model`; AI is
  also enabled by config. The scan loads settings once, runs the AI phase behind
  a spinner, and reports an analysis/cache tally.
- **Model** (`src/classifier/mod.rs`): `ClassifiedDependency` gained
  `ai_cached`.
- **Report** (`src/report/`, `templates/report.html`): AI Markdown is rendered by
  `pulldown-cmark` with raw HTML dropped, then inserted with `|safe`. The drawer
  has a 🤖 AI Library Release Analysis card, a ⚡ cache badge, a fallback note,
  and a per-ecosystem changelog link.
- **Docs and metadata**: version `0.3.0-dev`, `pulldown-cmark` dependency,
  `.rustydeps.json` ignored, README BYOK section, wiki feature page and
  implementation report, index/architecture/roadmap/report-spec/log updates.

## Tests and verification

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 122 passed, 0 failed |
| `cargo build --release` | success |
| `cargo run -- scan . --offline --no-open --format json` | exit 0, `ai_summary: null`, `ai_cached: false` |
| `cargo run -- scan . --offline --no-open -o /tmp/...` | fallback AI drawer and changelog links render |
| Local OpenAI-compatible mock, two scans | 19 requests + 19 cache files on the first run; 0 new requests and 19 cache badges on the second |
| OKF validator | PASSED (0 errors, 0 warnings) |

New unit coverage: config resolution/precedence and file parsing; provider
mapping and booleans; cache keys, round trips, mismatches, corruption, and
disabled mode; chat response parsing and endpoint resolution; prompt headings and
fields; Markdown rendering and HTML filtering; AI card, cache badge, fallback
link, and row-insight condensation.

## Technical debt

- No live provider integration test and no committed mocked HTTP server; a local
  OpenAI-compatible mock verified the full request/cache/report path, and unit
  tests cover request encoding and response parsing in isolation.
- The model answers from its own knowledge; release notes are not fetched or
  injected into the prompt.
- Cache entries never expire and there is no pruning command.
- Cache keys sanitize punctuation, so pathological version strings could collide;
  identity validation rejects the mismatched entry and rewrites it.
- JSON output gained `ai_cached` without a schema version field.

## Next recommended actions

1. Add a mocked-provider integration test (for example `wiremock`) covering
   success, 429, timeout, and malformed response paths.
2. Feed release notes/changelog excerpts into the user prompt for higher
   fidelity, with a bounded context budget.
3. Add `rustydeps cache clear` / `--no-cache` for explicit cache control.
4. Consider a cache format version field before changing the entry schema.
