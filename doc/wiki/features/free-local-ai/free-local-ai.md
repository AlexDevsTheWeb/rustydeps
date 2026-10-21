---
title: "Free Local AI — Ollama, Request Gating, and Honest Report States"
type: feature
description: "Quota-free local Ollama summaries, CLI provider switching, fatal-error circuit breaking, and report drawers that never advertise AI on packages with nothing to update."
status: active
version: 0.3.0
license: CC-BY-4.0
created: 2026-10-21
updated: 2026-10-21
tags: [feature, ai, ollama, free, quota, report, cli]
---

# Free local AI

## Motivation

A hosted Gemini key hit its quota mid-scan. The console then repeated an
HTTP/quota warning for every package, and the HTML report showed "AI analysis
unavailable — configure a provider" even on dependencies that were already at
their latest version — rows for which no analysis had ever been requested.

Three changes follow from that report:

1. Make the no-key, no-quota path first-class: local Ollama.
2. Spend no request on a package that has no upgrade, and stop the whole run
   after a fatal provider error instead of warning once per package.
3. Render AI state honestly: no "unavailable" notice on up-to-date or unknown
   rows, and no provider lecture when AI is simply disabled.

## Request gating

`UpdateType::is_update()` (`src/model.rs`) is the single predicate for "there is
an upgrade": `Safe`, `AtRisk`, or `Breaking`. Both the summarizer and the report
use it.

- `Summarizer::summarize_all` filters on `latest_version.is_some() &&
  update_type.is_update()`. Up-to-date rows cost no request; unknown rows have
  no target to describe.
- The report sets `ai_applicable` from the same predicate. Cached analyses still
  render if they exist, but an inapplicable row uses the **📦 Dependency Status**
  heading and never prints the unavailable notice.

## Fatal-error circuit breaker

`AiClientError::is_fatal()` (`src/ai/client.rs`) classifies errors that cannot
succeed on retry:

| Error | Fatal |
|-------|-------|
| HTTP 401/402/403/404/429 | yes |
| Any status whose body contains `quota` | yes |
| Transport connect failure (server down) | yes |
| Invalid/unusable response | yes |
| Other HTTP 5xx or per-package timeout | no |

`Summarizer` holds a shared `Arc<AtomicBool>`. The first fatal error trips it and
logs one warning; already-running requests finish, later packages return
immediately (cache hits still apply). A quota-exhausted key therefore produces
one message, not one per dependency.

## Free provider

Ollama speaks the same OpenAI-compatible protocol and runs on the user's
machine, so there is no key, account, network call, or quota. The existing
`ollama` provider is now documented as the free path, and two CLI flags make the
switch a one-liner:

```bash
rustydeps scan . --ai --ai-provider ollama
```

`--ai-provider` also resets the model and endpoint to that provider's defaults
unless `--ai-model` or `--ai-endpoint` is supplied. Without that rule, switching
from Gemini to Ollama would inherit the Gemini URL and model and fail.

The recommended local model is `llama3.2:3b`:

```json
{
  "ai": {
    "enabled": true,
    "provider": "ollama",
    "model": "llama3.2:3b",
    "endpoint": "http://localhost:11434/v1"
  }
}
```

`brew services start ollama` keeps the server available across logins;
`ollama pull llama3.2:3b` downloads the model once.

## Verification

Live scan of `~/CODE/CES/babette` with the Ollama config:

- 37 dependencies: 16 up-to-date, 21 with an upgrade.
- 21 AI analyses produced (14 cache hits, 7 fresh local calls); no provider
  warnings and no quota messages.
- Generated report: 21 AI cards, 16 status cards, **0** occurrences of "AI
  analysis unavailable".

Unit tests cover the update predicate, the per-row applicability flags, the
up-to-date/unknown rendering, disabled-AI rendering, provider override reset,
and fatal-error classification.

See [[ai-provider-compat]] for URL resolution and [[ai-changelog-summarizer]]
for the cache and prompt contract.
