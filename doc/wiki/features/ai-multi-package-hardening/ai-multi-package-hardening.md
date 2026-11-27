---
title: "AI Multi-Package Coverage and Custom Provider Verification"
type: feature
description: "Regression coverage proving every outdated package gets its own AI summary and report card, plus custom OpenAI-compatible endpoint and bearer-key tests."
status: active
version: 0.3.1
license: CC-BY-4.0
created: 2026-11-27
updated: 2026-11-27
tags: [feature, ai, regression, providers, custom-endpoint, report]
---

# AI multi-package coverage and custom provider verification

## Motivation

A report claimed that only the last outdated library showed an AI briefing while
earlier rows lost theirs, and asked for first-class custom
OpenAI-compatible endpoints. The brief is
`doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md` (Worktree 2).

## Findings

The reported multi-package failure does not reproduce on `development`:

- `Summarizer::summarize_all` maps every dependency that satisfies
  `latest_version.is_some() && update_type.is_update()` to its own
  `analyze(&mut dependency)` future. Each future owns a distinct mutable slot;
  `buffer_unordered(4)` cannot overwrite another row.
- `SummaryCache` lookups and fresh responses write `ai_summary`, `ai_model`, and
  `ai_cached` on that same slot.
- `ReportData::from_classified` derives `has_ai_analysis` per row from that
  row's rendered Markdown, so the template's `{% if dep.has_ai_analysis %}`
  branch is independent for every dependency.
- Drawer ids are positional (`drawer-{project_index}-{dep_index}`), not slugged
  package names, so duplicate or special-character names cannot collide.

Custom providers were already implemented in v0.3.0 and hardened in
[[ai-provider-compat]]: `AiProvider::Custom` requires an explicit endpoint and
model, accepts an optional key, and sends `Authorization: Bearer <key>` to
`{endpoint}/chat/completions`. `provider: "openai"` with an overridden
`endpoint` and `api_key` also resolves to the custom gateway.

## Added coverage

- `tests/ai_integration_tests.rs` — five outdated packages against a wiremock
  provider: all five summaries are present, exactly five requests are made, and
  the rendered report contains five `ai-analysis` cards and five
  `AI Library Release Analysis` headings.
- `src/config/mod.rs` — the documented `provider: "custom"` JSON file keeps its
  key, model, and endpoint after resolution; `provider: "openai"` also honors a
  custom endpoint, key, and model.
- `src/ai/client.rs` — a custom endpoint receives a real
  `Authorization: Bearer opencode-key-xxx` header at
  `/v1/chat/completions`.

No production code changed: the tests pin the already-correct behaviour so a
future refactor of the summarizer, cache, or template cannot reintroduce the
reported symptom.

## Related

- [[ai-changelog-summarizer]] — per-package summarizer, cache, and report card.
- [[ai-provider-compat]] — provider model, URL resolution, and diagnostics.
- [[implementation-report]] — delivery record.
- [[index]] — knowledge-graph entrypoint.
