---
title: "Per-Library AI Changelog Summarizer & BYOK Config"
type: feature
description: "Milestone 0.3.0: a granular per-package AI briefing for every outdated dependency, powered by a BYOK JSON config, an OpenAI-compatible client, and a per-version disk cache."
status: active
version: 0.3.0
license: CC-BY-4.0
created: 2026-10-19
updated: 2026-10-20
tags: [feature, ai, changelog, byok, config, cache, report]
---

# Per-library AI changelog summarizer & BYOK config

## Motivation

Earlier milestones attached a single templated line to at-risk and breaking
updates. That is project-generic: it does not tell a developer what changed in
*this* library between *these* two versions. Milestone 0.3.0 makes per-library
analysis the product's differentiator. Every outdated, registry-resolved
package gets its own Markdown briefing inside its report drawer, covering
breaking changes, security context, performance/architectural impact, and a
concrete migration step.

Privacy and cost stay with the user: RustyDeps ships no shared key. A local JSON
file supplies the provider credentials, and generated analyses are cached per
package version so repeat scans across projects are free. The brief is
`doc/raw/2026-10-19/2026-10-19-v0.3.0.md`.

## Configuration (BYOK)

The effective configuration is layered, most specific first:

1. CLI flags: `--ai` (force on), `--no-ai` (force off; wins), `--ai-model`.
2. `RUSTYDEPS_AI_*` environment variables.
3. `~/.config/rustydeps/config.json` (on Windows,
   `%APPDATA%\rustydeps\config.json`).
4. A project-local `.rustydeps.json` in the current working directory, used as
   a fallback when the user configuration does not exist.
5. Built-in defaults.

```json
{
  "ai": {
    "enabled": true,
    "provider": "openai",
    "api_key": "sk-...",
    "model": "gpt-4o-mini",
    "endpoint": "https://api.openai.com/v1",
    "max_tokens": 400,
    "cache_enabled": true
  }
}
```

- `provider` is `openai`, `gemini`, `openrouter`, `ollama`, or `custom`
  (`google` aliases `gemini`; `openai-compatible`/`proxy` alias `custom`).
- Defaults: OpenAI `gpt-4o-mini` at `https://api.openai.com/v1`; Gemini
  `gemini-2.5-flash` at Google's OpenAI-compatible
  `https://generativelanguage.googleapis.com/v1beta/openai`; OpenRouter
  `google/gemini-2.5-flash` at `https://openrouter.ai/api/v1`; Ollama `llama3.1`
  at `http://localhost:11434/v1` (no API key). A `custom` proxy has no defaults
  and requires both `model` and `endpoint`; its key is optional.
- `max_tokens` is clamped to `1..=8192`; `cache_enabled` defaults to `true`.
- `.rustydeps.json` is git-ignored because it may contain a key.

Environment overrides:

| Variable | Value |
|----------|-------|
| `RUSTYDEPS_AI_ENABLED` | `true`/`false`, `1`/`0`, `yes`/`no`, `on`/`off` |
| `RUSTYDEPS_AI_PROVIDER` | `openai`, `gemini`, `openrouter`, `ollama`, `custom` |
| `RUSTYDEPS_AI_API_KEY` | Key; also falls back to `OPENAI_API_KEY`, `GEMINI_API_KEY`/`GOOGLE_API_KEY`, or `OPENROUTER_API_KEY` |
| `RUSTYDEPS_AI_MODEL` | Model name |
| `RUSTYDEPS_AI_ENDPOINT` | OpenAI-compatible base URL |

A malformed file or invalid environment value is logged and disables AI for the
run; it never aborts the scan.

## Per-library request

`src/ai/prompt.rs` builds a system prompt that fixes four level-3 headings the
report styles:

- 🚨 Breaking Changes & Deprecations
- 🛡️ Security & Vulnerability Context
- ⚡ Performance & Architectural Impact
- 💡 Actionable Migration Advice

The user prompt names the ecosystem, package, current requirement, and target
version, and asks for a concise (<220 word) package-specific answer. The prompt
explicitly forbids inventing CVEs or changelog entries.

`src/ai/client.rs` is one OpenAI-compatible `reqwest` client for every
provider. It posts to `{endpoint}/chat/completions`, adds a bearer token when a
key exists, sends non-streaming `system`/`user` messages, and applies a 10-second
per-request timeout. Provider-aware URL normalization, OpenRouter attribution
headers, and URL-rich HTTP diagnostics are documented in
[[ai-provider-compat]]. Only the first choice's text is used.

## Cache

`src/ai/cache.rs` stores one JSON file per upgrade at:

```text
~/.cache/rustydeps/changelogs/{ecosystem}_{package}_{target_version}.json
```

The key is sanitized to lowercase `[a-z0-9._-]`, so scoped npm names and Go
module paths cannot escape the cache directory. Each entry records the
ecosystem, package, target version, model, Markdown, and a Unix timestamp. A
cache entry is used only when all identity fields match and the summary is
non-empty; corrupt or mismatched entries are ignored. `cache_enabled: false`
skips both reading and writing.

## Graceful degradation

`Summarizer` skips `UpToDate` and `Unknown` rows, analyzes at most four packages
concurrently, and prefers a cache hit before creating a request. Disabled AI,
a missing key, an invalid config, an HTTP error, a timeout, an empty completion,
or an unwritable cache all leave `ai_summary` as `None` and produce a warning.
The report then renders the standard package changelog link and a short
"AI analysis unavailable" note. There is no templated AI fallback.

## Report integration

`templates/report.html` renders a **🤖 AI Library Release Analysis** card inside
the expanded drawer:

- the Markdown analysis, converted to HTML by `src/report/markdown.rs`;
- the existing major-bump badge;
- a green **⚡ Cached analysis** badge when the entry came from disk;
- a **View package changelog** link to crates.io, npm, PyPI, pkg.go.dev, or
  Packagist.

Markdown is rendered with `pulldown-cmark`; raw HTML events and images are
dropped, and `javascript:`/`data:`/`vbscript:` link targets are replaced with
`#` before rendering, so provider output cannot inject markup or an executable
link. Askama then inserts the safe result with `|safe`. The row's one-line
insight is condensed from the first body line of the analysis.

## Related

- [[architecture]] — module boundaries and DTOs.
- [[ai-provider-compat]] — provider compatibility follow-up (URL resolution,
  OpenRouter, custom proxies, diagnostics).
- [[html-report-spec]] — the drawer and AI-tag contract.
- [[implementation-report]] — file-by-file delivery report.
- [[roadmap]] — Phase 6 status.
- [[index]] — knowledge-graph entrypoint.
