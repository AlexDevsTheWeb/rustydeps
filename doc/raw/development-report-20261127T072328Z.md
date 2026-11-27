# Development report — 2026-11-27T07:23:28Z

## Task

Worktree 3 of `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md`: Milestone
0.4.0, real registry changelog fetching and prompt grounding.

## Branch

`feat/v0.4.0-real-changelog-fetcher` (worktree `../rustydeps-feat-v0.4.0`),
based on `development` at `76890f4`.

## Changes

- `src/registry/changelog.rs` (new) — `ChangelogFetcher`:
  - repository discovery: crates.io `crate.repository`; npm `repository`
    string/object; PyPI `project_urls`/`home_page`/`project_url`; Packagist
    `source.url` for the matching version; `github.com/...` Go module paths.
  - GitHub release lookup by exact tag and `v` variant, then a bounded
    `releases?per_page=100` fallback for workspace tags (`tokio-1.54.1`,
    `serde-v1.0.200`, `crates/pkg/1.2.3`).
  - `GITHUB_TOKEN`/`GH_TOKEN` bearer auth; 8-second lookup cap; 5-second HTTP
    cap; sanitized and 1,500-character truncated bodies; all failures `None`.
- `src/ai/prompt.rs` — `UpgradeContext::changelog`/`with_changelog`; injects
  the bounded `Official Release Notes / Changelog Excerpt` block as the primary
  source; knowledge-based fallback unchanged.
- `src/ai/summary.rs` — owns a fetcher; resolves notes after a cache miss and
  before the chat call; `without_changelog()` for `--offline`/tests.
- `src/ai/cache.rs` — `CACHE_FORMAT_VERSION = 2`; `prompt_version` rejects
  legacy entries once, forcing a grounded regeneration.
- `src/main.rs` — `--offline` disables release-note fetching.
- `Cargo.toml` — `0.4.0-dev`.
- `README.md` — AI feature bullet and "Official release-note grounding"
  section (pipeline, budgets, token, cache invalidation, offline).
- `tests/ai_integration_tests.rs` — summarizer tests use `without_changelog()`.
- Wiki: new feature page/report, architecture, roadmap, index, log.

## Tests executed

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --lib` — 159 passed (16 new: changelog 11, prompt 3, cache 1,
  plus updated fixtures).
- `cargo test --test ai_integration_tests` — 4 passed.
- OKF validator (staged shim) — PASSED, 0 errors / 0 warnings.

## Real-world verification

Live responses matched the parsers:
- crates.io `/api/v1/crates/tokio` → `crate.repository =
  https://github.com/tokio-rs/tokio`.
- GitHub tokio releases are workspace-tagged (`tokio-1.54.1` with a body); the
  list fallback matches them.
- npm `/express/latest` → `repository: { url: git+https://github.com/expressjs/express.git }`.
- PyPI `/requests/json` → `project_urls.Source = https://github.com/psf/requests`.
- Packagist `/p2/monolog/monolog.json` → `source.url =
  https://github.com/Seldaek/monolog.git`.
- GitHub exact-tag checks: express `5.3.0` → 404, `v5.3.0` → 200, confirming
  the `v` toggle is required.

## Technical debt

- Workspace tags are matched against the 100 most recent releases; older
  targets beyond that window resolve to `None`.
- Release notes are not cached separately, so `cache clear` refetches them.
- Anonymous GitHub access is limited to 60 requests/hour; large scans need
  `GITHUB_TOKEN` for full coverage.

## Next recommended actions

Open the draft PR to `development`, record its link in the wiki log, and merge
after review.
