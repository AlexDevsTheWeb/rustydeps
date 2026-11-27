# Development report — 2026-11-27T07:21:00Z

## Task

Worktree 2 of `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md`: investigate
the "only the last outdated package gets an AI summary" report and verify
custom OpenAI-compatible provider support.

## Branch

`fix/ai-multi-package-bug-and-custom-provider` (worktree
`../rustydeps-fix-ai-bug`), based on `development` at `76890f4`.

## Findings

The multi-package failure does not reproduce. `summarize_all` maps every
`is_update()` dependency to its own `analyze(&mut dependency)` future; each
future writes a distinct slice element. `ReportData` derives
`has_ai_analysis` per row, and drawer ids are positional
(`drawer-{project}-{dep}`), so there is no shared buffer, flag, or slug
collision.

Custom providers already exist: `AiProvider::Custom` requires endpoint and
model, keeps an optional key, and posts to
`{endpoint}/chat/completions` with `Authorization: Bearer`. `provider:
"openai"` with an explicit endpoint also uses a custom gateway. No production
change was needed.

## Changes

- `tests/ai_integration_tests.rs` — five outdated packages against one wiremock
  provider: five summaries, five requests, five `has_ai_analysis` rows, five
  `ai-analysis` cards, five `AI Library Release Analysis` headings.
- `src/config/mod.rs` — custom-provider JSON file test (key/endpoint/model) and
  OpenAI-with-gateway test.
- `src/ai/client.rs` — custom endpoint bearer-auth test.
- `doc/wiki/features/ai-multi-package-hardening/` — feature page and
  implementation report.
- `doc/wiki/index.md`, `doc/wiki/log.md` — catalogue and history entries.

## Tests executed

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --lib` — 147 passed.
- `cargo test --test ai_integration_tests` — 5 passed.
- OKF validator (staged shim) — PASSED, 0 errors / 0 warnings.

## Technical debt

None introduced. The regression tests lock the current per-row contract; a
future change that shares summaries across rows will fail them.

## Next recommended actions

Open the draft PR to `development`, record its link in the wiki log, and merge
after review.
