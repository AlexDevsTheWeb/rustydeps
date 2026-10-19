---
title: "Implementation Report — Per-Library AI Changelog Summarizer"
type: report
description: "File-by-file delivery report for Milestone 0.3.0: layered BYOK config, OpenAI-compatible client, per-version cache, prompt design, graceful fallback, and the report AI card."
status: done
version: 0.3.0
sources: [2026-10-19-v0.3.0]
license: CC-BY-4.0
created: 2026-10-19
updated: 2026-10-19
tags: [implementation-report, ai, changelog, config, cache, report]
---

# Implementation Report — Per-Library AI Changelog Summarizer

## Summary

Implemented `doc/raw/2026-10-19/2026-10-19-v0.3.0.md` on
`feat/ai-changelog-summarizer` (version `0.3.0-dev`). Each outdated,
registry-resolved package now gets its own cached Markdown briefing from a
BYOK-configured OpenAI-compatible provider. AI remains optional: every failure
path degrades to a normal report row with a package changelog link.

## Done

- **`Cargo.toml`** — version `0.3.0-dev`; added `pulldown-cmark = "0.13"`.
- **`src/config/mod.rs`** (new) — `Config`/`AiConfig` schema, `AiProvider`
  defaults (OpenAI/Gemini/Ollama), `AiOverrides`, `AiSettings`, layered
  `load()`, project-local and XDG config paths, cache path, boolean parsing,
  file/env error types, provider-native key fallback, and unit tests.
- **`src/ai/client.rs`** (new) — one `reqwest` OpenAI-compatible client,
  `chat/completions` URL resolution, bearer auth, 10-second timeout, response
  parsing, character-safe error excerpts, and tests.
- **`src/ai/prompt.rs`** (new) — fixed four-section system prompt and a
  per-upgrade user prompt; tests assert headings and package/version fields.
- **`src/ai/cache.rs`** (new) — `CachedSummary`, sanitized
  `{ecosystem}_{package}_{target_version}` keys, read/write with identity
  validation, disabled-cache no-op, and tests.
- **`src/ai/summary.rs`** — replaced the templated Phase 3 summarizer with
  `Summarizer::from_overrides`/`from_settings`, bounded concurrent
  `summarize_all`, cache-first `analyze`, non-fatal warnings, and tests.
- **`src/ai/mod.rs`** — exports the client, cache, prompt, and summarizer
  modules.
- **`src/cli.rs`** — added `--no-ai` and `--ai-model <MODEL>` while keeping
  `--ai`; added parse tests.
- **`src/main.rs`** — loads layered AI settings, classifies first, runs the AI
  phase behind a spinner, and reports an analysis/cache tally instead of dumping
  raw Markdown.
- **`src/classifier/mod.rs`**, **`src/classifier/tests.rs`** — added
  `ai_cached: bool` to `ClassifiedDependency`; classification still starts with
  no summary.
- **`src/report/markdown.rs`** (new) — `pulldown-cmark` rendering with raw HTML
  events and images filtered, plus `javascript:`/`data:`/`vbscript:` href
  neutralisation; tests cover headings/code, HTML injection, and unsafe links.
- **`src/report/mod.rs`**, **`src/report/tests.rs`** — AI HTML, cached flag,
  one-line insight condensation, per-ecosystem changelog URLs, updated
  self-contained-document assertion (anchors allowed, resources not), plus
  cached/fallback/insight tests.
- **`templates/report.html`** — AI card, cache badge, Markdown typography,
  fallback note, changelog link, and `has_ai_analysis` wiring.
- **`README.md`** — feature bullet, CLI options, BYOK config section, env table,
  cache location, and v0.3.0 tag examples.
- **`.gitignore`** — ignores `.rustydeps.json`.
- **Wiki** — this subfolder, [[ai-changelog-summarizer]], plus updates to
  [[index]], [[architecture]], [[roadmap]], [[html-report-spec]], and [[log]].

## Verification

- `cargo fmt --all --check` → clean.
- `cargo clippy --all-targets -- -D warnings` → clean.
- `cargo test` → 122 passed, 0 failed.
- `cargo build --release` → success.
- `cargo run -- scan . --offline --no-open --format json` → succeeds with
  `ai_summary: null`, `ai_cached: false`.
- An HTML scan renders the fallback drawer with the standard changelog link.
- **End-to-end AI check** against a local OpenAI-compatible mock: the first scan
  made 19 requests and wrote 19 entries under
  `~/.cache/rustydeps/changelogs/`; the second scan made zero new requests,
  reported `19 served from the local cache`, and rendered 19 `⚡ Cached analysis`
  badges plus the Markdown card.

## Technical debt / deferred

- **No live integration test** against OpenAI/Gemini/Ollama and no committed
  mocked HTTP server. Request shape, response parsing, and cache reuse were
  verified manually against a local OpenAI-compatible mock; unit tests cover the
  isolated pieces.
- **No changelog fetching** — the model answers from its own knowledge. A later
  milestone can feed release notes into the user prompt.
- **No cache pruning or TTL** — entries are intentionally permanent until the
  user removes `~/.cache/rustydeps/changelogs/`.
- **Cache key collisions are theoretically possible** for versions that differ
  only in punctuation; identity validation rejects a mismatched entry, so this
  causes a rewrite rather than a wrong summary.
- **The scan output is unchanged in JSON shape except for `ai_cached`**; no
  schema version field exists yet.

## Blocked

- The brief asks to create `doc/raw/YYYY-MM-DD/ai-changelog-summarizer.md` and
  update `doc/raw/index.md`/`doc/raw/log.md`. `doc/raw/` is immutable input in
  this repository and those index/log files do not exist, so the feature
  documentation lives in the wiki per `AGENTS.md`.

## MR

Draft PR: pending (branch `feat/ai-changelog-summarizer` → `development`).

## Related

- [[ai-changelog-summarizer]] — feature page.
- [[architecture]] — current module design.
- [[roadmap]] — Phase 6 completion.
