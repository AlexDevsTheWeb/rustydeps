---
title: "AI Provider Compatibility — OpenRouter, Custom Proxies, and URL Resolution"
type: feature
description: "Provider-aware chat-completions URL resolution, OpenRouter and custom-proxy support, and URL-rich HTTP diagnostics for the BYOK AI summarizer."
status: active
version: 0.3.0
license: CC-BY-4.0
created: 2026-10-20
updated: 2026-10-20
tags: [feature, ai, providers, openrouter, gemini, ollama, url, diagnostics]
---

# AI provider compatibility

## Motivation

Milestone 0.3.0 shipped one OpenAI-compatible client for OpenAI, Gemini, and
Ollama. That was correct in principle but fragile in practice: the client always
appended `/chat/completions` to the configured endpoint, so a full endpoint, a
trailing slash, a query string, or a bare provider origin could produce a
malformed URL and an opaque HTTP failure. The provider surface also stopped
short of OpenRouter and user-run OpenAI-compatible proxies.

The brief is `doc/raw/2026-10-20/2026-10-20-fix.md`. This page documents the
resulting provider model, URL resolver, diagnostics, and README guide.

## Provider model

`AiProvider` in `src/config/mod.rs` now covers five variants. All speak the same
chat-completions protocol; the variant selects defaults and header behaviour.

| Provider | Label | Default endpoint | Default model | API key |
|----------|-------|------------------|---------------|---------|
| OpenAI | `openai` | `https://api.openai.com/v1` | `gpt-4o-mini` | Required |
| Gemini | `gemini` | `https://generativelanguage.googleapis.com/v1beta/openai` | `gemini-3.5-flash` | Required |
| OpenRouter | `openrouter` | `https://openrouter.ai/api/v1` | `google/gemini-2.5-flash` | Required |
| Ollama | `ollama` | `http://localhost:11434/v1` | `llama3.1` | Not required |
| Custom proxy | `custom` | none (user-supplied) | none (user-supplied) | Optional |

Label aliases keep existing configurations working: `google` and
`google-gemini` select Gemini; `open-router`/`open_router` select OpenRouter;
`openai-compatible`/`open_ai_compatible`/`proxy` select the custom provider.

Provider-native key fallbacks are resolved in this order:

1. `RUSTYDEPS_AI_API_KEY`
2. `OPENAI_API_KEY`, `GEMINI_API_KEY`/`GOOGLE_API_KEY`, or
   `OPENROUTER_API_KEY`, depending on the provider.

A custom proxy has no implicit key fallback and does not require a key, because
keyless local gateways are common. It does require an explicit `model` and
`endpoint`; `AiClient::new` rejects either when missing and the summarizer logs
a warning instead of issuing a doomed request.

## URL resolution

`chat_endpoint` in `src/ai/client.rs` is the single place that turns a configured
base into a request URL. It is provider-aware and deliberately forgiving:

1. Trim whitespace and split an optional `?query` or `#fragment`.
2. Strip trailing slashes from the path part.
3. If the path already ends in `/chat/completions`, keep it as-is.
4. If the URL is a bare origin, append that provider's documented root:
   - OpenAI and Ollama: `/v1/chat/completions`
   - Gemini: `/v1beta/openai/chat/completions`
   - OpenRouter: `/api/v1/chat/completions`
   - Custom: `/chat/completions`
5. If Gemini is given the bare `/v1beta` root, insert `/openai` before the
   completion path.
6. Otherwise append `/chat/completions`.
7. Re-attach the original query or fragment at the very end.

Recognized forms therefore include `https://api.openai.com`,
`https://api.openai.com/v1`, `.../v1/`, `.../v1/chat/completions/`,
`https://generativelanguage.googleapis.com/v1beta`, `.../v1beta/openai`, and a
custom URL with an existing query string. No path is appended twice.

OpenRouter requests additionally send `HTTP-Referer`
(`OPENROUTER_APP_URL`) and `X-Title` (`OPENROUTER_APP_TITLE`) so usage is
attributed in its dashboard. Bearer authentication is applied whenever a key is
present.

Gemini thinking models (`gemini-3*`, excluding `-lite`) receive
`reasoning_effort: "none"`: Google charges hidden reasoning against
`max_tokens`, so the 400-token summary budget could otherwise be spent before
any visible text. Lite and older variants reject that field; when a provider
answers `400` with `invalid argument`, the client retries once without it. The
request timeout is 45 seconds, reflecting real reasoning-model latency.

## Diagnostics

Non-success responses now produce an error that names the resolved URL, the HTTP
status, and a compact body excerpt:

```text
request to https://api.openai.com/v1/chat/completions returned HTTP 401: {"error": ...}
```

Transport failures carry the same URL. The excerpt collapses all whitespace into
a single line, keeps at most 300 characters (`ERROR_BODY_LIMIT`), is
character-safe for multi-byte text, and renders `<empty response body>` for an
empty body. A user can usually identify a wrong endpoint, key, or model without
`--verbose`. These errors remain non-fatal: the summary is skipped and the report
renders its normal changelog link.

## README guide

`README.md` now documents the BYOK model end to end:

- local-first architecture: no RustyDeps proxy, no telemetry, direct
  machine-to-provider calls;
- primary config path `~/.config/rustydeps/config.json` and the
  `.rustydeps.json` fallback;
- per-version cache path (`~/.cache/rustydeps/changelogs/`);
- copy-paste JSON for Gemini, OpenAI, OpenRouter, and Ollama, plus a custom-proxy
  example;
- `RUSTYDEPS_AI_ENABLED`, `RUSTYDEPS_AI_PROVIDER`, `RUSTYDEPS_AI_API_KEY`,
  `RUSTYDEPS_AI_MODEL`, and `RUSTYDEPS_AI_ENDPOINT`;
- the `--ai`, `--no-ai`, and `--ai-model` flags;
- a short troubleshooting note explaining the new URL/status/body errors.

## Verification

Covered by unit tests in `src/config/mod.rs` and `src/ai/client.rs`:

- every provider resolves from its base URL and from a bare origin;
- Gemini `/v1beta` and full endpoints, trailing slashes, and custom full
  endpoints all resolve correctly;
- query strings and fragments survive resolution unchanged;
- a custom provider without an endpoint or model is rejected;
- error excerpts are single-line, bounded, and character-safe;
- socket-based tests exercise real requests: error diagnostics carry the
  resolved URL/status/body, OpenRouter sends its attribution headers, and a `400
  invalid argument` retries once without `reasoning_effort`.

The repository gates (`cargo fmt`, clippy with `-D warnings`, tests, release
build) are recorded in the [implementation report](implementation-report.md).

## Related

- [[ai-changelog-summarizer]] — the per-library summarizer and BYOK config.
- [[architecture]] — module-level design.
- [[implementation-report]] — file-by-file delivery report.
- [[index]] — knowledge-graph entrypoint.
