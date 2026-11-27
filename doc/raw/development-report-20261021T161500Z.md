# Development Report — v0.3.1 Surgical Fixes

- **Date:** 2026-10-21
- **Branch:** `feat/v0.3.1-surgical-fixes`
- **Scope:** AI cache management, Ollama preflight, cached-model report badges,
  integration tests.

## Changes

### Cache management

- `rustydeps cache clear` deletes every file in
  `~/.cache/rustydeps/changelogs/` through `SummaryCache::clear()` and prints
  `✔ Cleared X cached AI changelog entries.` A missing directory reports zero.
- `rustydeps scan --no-cache` sets `AiOverrides::no_cache`, which resolves
  `force_refresh = true` for the run. Cache reads are skipped; fresh analyses
  are still written back when caching is enabled.

### Ollama preflight

- Before spawning package tasks, an Ollama run probes `<origin>/api/tags` with a
  1-second timeout. The URL is rebuilt from either a `/v1` base or a full
  chat-completions URL.
- On failure, one warning names the daemon origin and suggests
  `ollama serve` / `brew services start ollama`; the circuit breaker is tripped
  and no chat requests are attempted. Cached analyses still apply.
- `AiClientError` and the existing breaker remain unchanged; the preflight is a
  new producer of the same stop flag.

### Report badge

- `ClassifiedDependency` gained `ai_model`. Cache hits copy the model from
  `CachedSummary`; fresh analyses record the configured model.
- The report renders `⚡ Cached (<model>)`, falling back to `unknown` for old
  entries without provenance.

### Test infrastructure

- Added `src/lib.rs` and converted `src/main.rs` to consume `rustydeps::*`. The
  binary no longer duplicates the module tree, and integration tests can use the
  public API.
- Added `wiremock = "0.6"` and `tests/ai_integration_tests.rs` with 200 OK,
  429 breaker, 404 fatal, and offline-Ollama cases.
- Unit tests cover cache clearing, `--no-cache` resolution, CLI parsing, and the
  Ollama probe URL.
- Version bumped to `0.3.1-dev`.

## Tests executed

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --all-targets` — 144 unit + 4 integration passed.
- `cargo build --release` — success.
- `XDG_CACHE_HOME=<temp> rustydeps cache clear` — removed 2 seeded entries and
  printed `✔ Cleared 2 cached AI changelog entries.`
- `rustydeps scan ~/CODE/CES/babette --no-cache` — 37 dependencies; 21 AI
  analyses, 0 served from cache, 216.8 s with local Ollama; report written.
- `rustydeps scan <temp project> --no-cache` with `XDG_CACHE_HOME=<temp>` — 1
  analysis, 0 cached, and 1 fresh JSON entry containing `llama3.2:3b`, proving
  the refresh still writes to the cache.
- `rustydeps scan ~/CODE/CES/babette --ai-endpoint http://127.0.0.1:9/v1` — one
  `Ollama daemon is offline at http://127.0.0.1:9` warning; 21 analyses served
  from cache with zero chat requests.
- Report inspection — `⚡ Cached (llama3.2:3b)` and
  `⚡ Cached (gemini-3.5-flash)` badges reflect the producing model; 0
  "AI analysis unavailable" strings.
- OKF validator — PASSED.

## Technical debt

- The Ollama probe always targets `/api/tags`; a custom Ollama-compatible proxy
  without that endpoint is treated as offline.
- `cache clear` removes files but leaves nested directories untouched.
- The library target adds a second compile surface; the binary remains thin.
- No-cache scans regenerate every package and are intentionally slower (216 s
  for 21 local analyses).

## Next recommended actions

- Open the draft PR to `development`.
- Consider `cache stats` and a per-package cache bust.
- Add a provider-aware preflight hook for other local runtimes (llama.cpp,
  LM Studio) instead of hard-coding Ollama.
