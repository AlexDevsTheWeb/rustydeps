---
title: "Implementation Report — Free Local AI"
type: report
description: "File-by-file delivery report for quota-free Ollama switching, upgrade-only AI requests, the fatal-error breaker, and honest report AI states."
status: done
version: 0.3.0
sources: []
license: CC-BY-4.0
created: 2026-10-21
updated: 2026-10-21
tags: [implementation-report, ai, ollama, quota, report, cli]
---

# Implementation Report — Free Local AI

## Summary

Addressed the reported Gemini quota warnings and the misleading report copy.
AI work is now limited to packages that actually have an upgrade, a fatal
provider failure stops the remaining requests after one warning, and the report
no longer prints "AI analysis unavailable" for up-to-date or unknown rows.
Ollama is the documented free provider, with `--ai-provider`/`--ai-endpoint`
flags for a one-scan switch.

## Done

- **`src/model.rs`** — added `UpdateType::is_update()` as the shared
  "has an upgrade" predicate.
- **`src/ai/summary.rs`** — request gating reuses `is_update()`; a shared
  `Arc<AtomicBool>` circuit breaker stops network calls after a fatal error
  (cache hits still apply); one warning replaces the per-package quota spam.
- **`src/ai/client.rs`** — added `AiClientError::is_fatal()` covering
  authentication, quota, model-not-found, connection, and invalid-response
  failures; unit test for fatal vs transient statuses.
- **`src/config/mod.rs`** — `--ai-provider` and `--ai-endpoint` overrides;
  provider switches reset model/endpoint to the new provider's defaults unless
  supplied explicitly; `AiProvider` derives `clap::ValueEnum`; tests for reset
  behavior.
- **`src/cli.rs`**, **`src/main.rs`** — wired the new flags and passed
  `ai_enabled` into report metadata.
- **`src/report/mod.rs`**, **`src/report/tests.rs`**, **`templates/report.html`**
  — added `ai_applicable`/`ai_enabled` row state; up-to-date and unknown rows
  render a **📦 Dependency Status** card with no unavailable notice; disabled AI
  renders no notice; existing-analysis cards still show the AI heading.
- **`README.md`** — free Ollama guidance, new flags, upgrade-only scope, and the
  breaker behavior.
- **Wiki** — [[free-local-ai]] feature page; [[architecture]] updated (flags,
  report states, 45-second timeout, breaker).

## Local environment

- Installed Ollama 0.40.7 via Homebrew and started `sh.brew.ollama`.
- Pulled `llama3.2:3b`.
- Replaced `~/.config/rustydeps/config.json` with the Ollama provider and
  `llama3.2:3b`; the previous Gemini config is backed up beside it as
  `config.json.bak-20261021T...`.

## Tests and live verification

- `cargo fmt --all`
- `cargo test --all-targets`: 137 passed.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo build --release`: clean.
- Live scan `target/release/rustydeps scan ~/CODE/CES/babette --no-open`:
  37 dependencies (16 up-to-date, 18 safe, 1 at-risk, 2 breaking); 21 AI
  analyses (14 cached, 7 fresh from Ollama); `ai_provider="ollama"`; no quota or
  provider warnings; 39-second run.
- Generated HTML assertions: 21 AI cards, 16 status cards, 0 "AI analysis
  unavailable".

## Technical debt

- The breaker is scan-local; a later scan retries the provider (intentional, so
  a restored quota works without restarting).
- The local model is smaller than hosted options and may produce shorter
  summaries; `max_tokens` remains 400.
- Cached summaries are keyed by package + target version, not by model, so a
  cached hosted analysis is reused after switching to Ollama.

## Next actions

- Open the draft PR from `fix/ai-provider-compat` to `development`.
- Consider a one-time `ollama serve` reachability check so a stopped local
  server yields an actionable setup hint rather than a transport warning.
