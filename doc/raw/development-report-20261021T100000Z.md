# Development Report — Free Local AI and Honest Report States

- **Date:** 2026-10-21
- **Branch:** `fix/ai-provider-compat`
- **Scope:** quota-free AI provider, upgrade-only AI requests, fatal-error
  breaker, report AI-state cleanup.

## Changes

### Free local provider

- Ollama 0.40.7 installed via Homebrew and started as `sh.brew.ollama`.
- `llama3.2:3b` pulled locally.
- `~/.config/rustydeps/config.json` switched to `provider: "ollama"`,
  `model: "llama3.2:3b"`, `endpoint: "http://localhost:11434/v1"`. The previous
  Gemini config was backed up beside it.
- Added `--ai-provider <PROVIDER>` and `--ai-endpoint <URL>` (clap `ValueEnum`
  for providers). A provider switch resets model and endpoint to that provider's
  defaults unless explicitly overridden, so `--ai-provider ollama` cannot inherit
  a Gemini URL/model.

### Request gating and breaker

- `UpdateType::is_update()` is the shared predicate for `Safe`/`AtRisk`/
  `Breaking`.
- `Summarizer` filters on a resolved target plus `is_update()`, so up-to-date and
  unknown packages never call the provider.
- `AiClientError::is_fatal()` marks 401/402/403/404/429, quota bodies, connect
  failures, and unusable responses as fatal. A scan-local `Arc<AtomicBool>`
  stops further network calls after the first fatal error; cached analyses still
  apply. One warning replaces per-package spam.

### Report

- New row state: `ai_applicable` (has an upgrade) and `ai_enabled` (copied from
  scan metadata).
- Up-to-date and unknown rows render **📦 Dependency Status** and never print
  "AI analysis unavailable".
- Disabled AI renders no unavailable notice; eligible rows without a summary
  show a package-scoped fallback and the changelog link.
- Existing analyses still use the **🤖 AI Library Release Analysis** heading,
  including cached ones served while AI is disabled.

### Docs

- `README.md`: free Ollama section, new flags, upgrade-only scope, breaker
  behavior.
- Wiki: new [[free-local-ai]] page and implementation report; [[architecture]]
  updated; index and log updated.

## Tests executed

- `cargo fmt --all`
- `cargo test --all-targets` — 137 passed.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo build --release` — clean.
- OKF validator — PASSED (0 errors, 0 warnings).
- Live scan:
  `target/release/rustydeps scan ~/CODE/CES/babette --no-open -o /tmp/rustydeps-free.html`
  - 37 dependencies: 16 up-to-date, 18 safe, 1 at-risk, 2 breaking.
  - 21 AI analyses: 14 cache hits, 7 fresh local Ollama calls.
  - `ai_provider="ollama"`, 38.5 s, no quota/provider warnings.
  - HTML: 21 AI cards, 16 status cards, 0 "AI analysis unavailable".

## Technical debt

- The breaker is scan-local by design; the next scan retries a provider whose
  quota or key was fixed.
- Cache keys ignore the producing model; switching providers reuses existing
  summaries for the same package + target version.
- A stopped Ollama server produces one transport warning per scan, not a
  dedicated setup hint yet.
- Local 3B summaries are shorter than hosted-model output.

## Next recommended actions

- Open the draft PR from `fix/ai-provider-compat` to `development`.
- Consider an explicit Ollama reachability probe with a one-line
  `brew services start ollama` hint.
- Optionally record the model name in the report's cached-analysis badge.
