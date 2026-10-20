---
title: "Implementation Report — AI Provider Compatibility"
type: report
description: "File-by-file delivery report for provider-aware URL resolution, OpenRouter and custom-proxy support, HTTP diagnostics, and the README BYOK guide."
status: done
version: 0.3.0
sources: [2026-10-20-fix]
license: CC-BY-4.0
created: 2026-10-20
updated: 2026-10-20
tags: [implementation-report, ai, providers, openrouter, url, diagnostics, readme]
---

# Implementation Report — AI Provider Compatibility

## Summary

Implemented `doc/raw/2026-10-20/2026-10-20-fix.md` on
`fix/ai-provider-compat`. The AI client now resolves provider URLs through one
normalizer, supports OpenRouter and arbitrary custom OpenAI-compatible proxies,
sends OpenRouter attribution headers, and reports HTTP failures with the request
URL, status, and a compact body snippet. `README.md` gained a complete BYOK
provider guide.

The brief's inline code spans were stripped by its authoring tool, so the
specific provider labels, endpoint forms, and environment variable names were
reconstructed from the existing implementation and the task structure. No
behaviour was invented beyond that reconstruction.

## Done

- **`src/config/mod.rs`** — added `OpenRouter` and `Custom` to `AiProvider`;
  provider defaults and label aliases; optional keys for the custom provider;
  required model/endpoint semantics for custom; `OPENROUTER_API_KEY` fallback;
  expanded tests for defaults, aliases, and custom resolution.
- **`src/ai/client.rs`** — provider-aware `chat_endpoint` normalizer (trim,
  trailing slashes, bare origins, Gemini `/v1beta`, full endpoints,
  query/fragment preservation); OpenRouter `HTTP-Referer`/`X-Title` headers;
  custom-provider endpoint/model validation; URL-bearing `Transport` and
  `Status` errors; compact single-line body snippets; tests across every
  provider.
- **`README.md`** — rewrote the *AI changelog summaries (BYOK)* section:
  local-first model, config and cache paths, provider table, five copy-paste
  JSON configurations, environment table with native-key fallbacks, CLI flag
  table, and troubleshooting guidance for the new diagnostics.
- **Wiki** — this subfolder, [[ai-provider-compat]], plus updates to
  [[index]], [[architecture]], [[ai-changelog-summarizer]], and [[log]].

## Verification

- `cargo fmt --all --check` → clean.
- `cargo clippy --all-targets -- -D warnings` → clean.
- `cargo test` → 130 passed, 0 failed (122 before this change).
- `cargo build --release` → success.
- OKF validator → PASSED (0 errors, 0 warnings).
- CLI smoke test against a local mock provider: a `custom` endpoint configured as
  `http://127.0.0.1:18081/v1` was called at `/v1/chat/completions`; the 401
  warning named the URL, status, and body (`smoke: bad key`); the scan exited
  successfully with `ai_summary: null`.

No live provider calls were made: OpenAI, Gemini, OpenRouter, and a hosted custom
proxy all require credentials or accounts that are not part of the repository.
The resolver and error formatting are pure functions covered by unit tests, and
two socket-based tests exercise the full request path, error diagnostics, and
OpenRouter headers.

## Deviations

- The brief refers to provider names and variables through inline code spans that
  are empty in the source file. The implementation uses the labels already
  present in the project (`openai`, `gemini`, `ollama`), adds `openrouter` and
  `custom`, and keeps the existing `RUSTYDEPS_AI_*` namespace.
- The brief forbids duplicate reports under `doc/raw/`; developer notes therefore
  live only here in the wiki. The source brief itself is committed unchanged.
- No CLI flags for provider or endpoint were added. The brief names only
  `--ai`, `--no-ai`, and `--ai-model`; provider and endpoint remain file/env
  settings.

## Deferred

- No live integration test or committed mock server; the existing milestone's
  manual mock-server check remains the end-to-end evidence.
- OpenRouter's optional ranking headers (`X-Title`, `HTTP-Referer`) are fixed to
  the project URL and name; they are not configurable.

## MR

Draft PR: _pending — filled in after the draft is opened._

## Related

- [[ai-provider-compat]] — feature page.
- [[ai-changelog-summarizer]] — the underlying summarizer.
- [[architecture]] — current module design.
- [[index]] — knowledge-graph entrypoint.
