---
title: "Implementation Report — AI Multi-Package and Custom Provider Coverage"
type: report
description: "Delivery record for the multi-package AI regression test and custom-endpoint verification."
status: done
version: 0.3.1
sources: [2026-11-27-v0.4.0-and-fixes]
license: CC-BY-4.0
created: 2026-11-27
updated: 2026-11-27
tags: [implementation-report, ai, regression, providers]
---

# Implementation Report — AI Multi-Package and Custom Provider Coverage

## Summary

Applied Worktree 2 of `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md` on
`fix/ai-multi-package-bug-and-custom-provider`. The reported "only the last
package gets an AI card" defect does not reproduce; the existing per-slot
assignment, per-row `has_ai_analysis`, and positional drawer ids are correct.
The change adds regression coverage for five simultaneous outdated packages and
for custom OpenAI-compatible endpoints. No production code changed.

## Done

- `tests/ai_integration_tests.rs` — new test: five breaking packages, one
  wiremock provider, all five summaries set, five requests sent, all five report
  rows flagged `has_ai_analysis`, five rendered `ai-analysis` cards and five
  `AI Library Release Analysis` headings.
- `src/config/mod.rs` — new tests for the documented `provider: "custom"` JSON
  file (key, model, endpoint preserved) and for `provider: "openai"` with a
  custom gateway endpoint/key/model.
- `src/ai/client.rs` — new socket-level test asserting a custom endpoint
  receives `Authorization: Bearer opencode-key-xxx` on
  `/v1/chat/completions`.
- New wiki page [[ai-multi-package-hardening]] and this report.

## Verification

- Focused integration test `every_outdated_package_gets_a_summary_and_its_own_report_card`
  — PASS.
- `cargo test --lib` — result recorded in the development report.
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
  OKF validation — results recorded in the development report.

## Root-cause note

v0.3.1 already routed only `Safe`/`AtRisk`/`Breaking` rows into per-slot futures
and rendered cards from each row's own Markdown. The regression test is the
durable fix: it fails if a refactor starts sharing one summary buffer, one
`has_ai_analysis` flag, or slug-based drawer ids across rows.

## Next actions

Open the draft PR to `development` and record its link in the log.
