---
title: "Development Report — Gemini AI Failure Fix"
type: report
description: "Retired Gemini model ID, hidden reasoning budget, and a too-short request timeout causing missing AI summaries."
status: done
version: 0.3.0
sources: [2026-10-20-fix]
license: CC-BY-4.0
created: 2026-10-20
updated: 2026-10-20
tags: [development-report, ai, gemini, config, timeout]
---

# Development Report — Gemini AI failure fix

Timestamp: `20261020T161116Z`
Branch: `fix/ai-provider-compat` (off `development`)
Version: `0.3.0-dev`

## Diagnosis

The reported console warnings and missing AI cards were reproduced with the
user configuration (`~/.config/rustydeps/config.json`, provider `gemini`,
OpenAI-compatible endpoint, valid `AQ.` API key):

```text
request ... returned HTTP 404: models/gemini-1.5-flash-latest is not found
```

The key authenticated correctly; the configured model had been retired. Live
probes also showed that the code's default Gemini model (`gemini-2.5-flash`) is
no longer served to new accounts, and that a 10-second request timeout is too
short for reasoning models (observed completions took 12–30 seconds).

## Changes made

- **Configuration** (`src/config/mod.rs`): Gemini default model is now
  `gemini-3.5-flash`; the provider-defaults test was updated.
- **AI client** (`src/ai/client.rs`):
  - `REQUEST_TIMEOUT` raised from 10 to 45 seconds.
  - Gemini thinking models (`gemini-3*`, excluding `-lite`) send
    `reasoning_effort: "none"`. Gemini charges hidden reasoning against
    `max_tokens`, which previously consumed the 400-token budget and returned
    truncated or empty summaries.
  - A `400` whose body contains `invalid argument` is retried once without
    `reasoning_effort`, keeping lite/older Gemini variants usable.
  - The request sends were factored into `AiClient::post`; the test server now
    serves request sequences.
- **User configuration** (`~/.config/rustydeps/config.json`): model changed to
  `gemini-3.5-flash`; a timestamped backup was written beside the file.
- **Docs**: README provider table/example, retired-model troubleshooting note,
  45-second timeout, and reasoning rule; wiki `architecture`,
  `ai-provider-compat`, `ai-changelog-summarizer`, and `log` updated.

## Tests and verification

| Command / scenario | Result |
|--------------------|--------|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 132 passed, 0 failed |
| Live scan, 3 outdated npm packages | 3/3 AI analyses, 0 warnings, ~20 s |
| Repeat scan | 3/3 served from the local cache in ~0.3 s |
| Rendered report | 3 AI cards, 3 row AI tags, 0 "AI analysis unavailable" |
| OKF validator | PASSED (0 errors, 0 warnings) |

New coverage: the reasoning gate (`gemini-3.5-flash` vs `-lite`, OpenAI,
OpenRouter) and a socket-level test proving the retry drops
`reasoning_effort` after a `400 invalid argument`.

## Technical debt

- Model availability remains provider-managed. A retired model ID still
  surfaces as a per-package HTTP 404 warning; the user must change `ai.model`.
- `reasoning_effort` support is inferred from the model name (`-lite`); a future
  variant could reject it and incur one extra request before the retry succeeds.
- No automated live-provider integration test (requires a real key).
- The cache key still omits the model, so cached summaries survive a model
  change until the cache is cleared.

## Next recommended actions

1. Parse a provider 404 body and log the provider's suggested replacement model
   explicitly.
2. Add an optional `ai.reasoning_effort` setting for users who want to trade
   latency for deeper analysis.
3. Include the model in the cache key or add a cache format version.
4. Add mocked-provider integration tests for 429, timeout, and truncated
   completions.
