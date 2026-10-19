---
title: "Phase 3 — SemVer Classifier, AI Summaries & CLI Polish"
type: feature
description: "Classifies each dependency update as Safe/At-Risk/Breaking/UpToDate/Unknown, adds the CLI banner, templated AI summaries, and an offline mode."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [feature, phase-3, classifier, semver, ai, cli]
---

# Phase 3 — SemVer classifier, AI summaries & CLI polish

## Motivation

After Phase 2, a scan knew each dependency's declared version and the latest
published version, but it could not say whether that difference was safe to
take. Phase 3 turns the version pair into an actionable classification, brands
the CLI, attaches a human-readable summary to risky updates, and adds an
offline mode so a scan can run without network access. This is the data the
Phase 4 report renders.

## What shipped

- **Classifier (`src/classifier/`)** — a pure `classify_update` function, the
  `ClassifiedDependency` DTO, and a `latest_stable` helper.
- **`UpdateType`** in `src/model.rs` — `Safe`, `AtRisk`, `Breaking`,
  `UpToDate`, `Unknown`.
- **Latest-stable target** in `src/registry/crates_io.rs` — prereleases are
  filtered so the upgrade target is the newest stable release.
- **AI summary module (`src/ai/`)** — provider detection plus templated
  summaries (live provider calls arrive in Phase 5).
- **CLI banner and `--offline`/`--no-fetch`** — `src/cli.rs`, `src/main.rs`.

## Components

### Classification (`classify_update`)

`classify_update(current, latest)` parses the declared value leniently
(`^1.2.3`, `~1.2`, `=1.0.0`, `1.2`, `1`) and the registry version, then buckets
the jump:

| Situation | `UpdateType` |
|-----------|--------------|
| Major version differs | `Breaking` |
| Minor differs and the declared requirement allows it | `Safe` |
| Minor differs and the requirement does not allow it (e.g. past a `~` pin) | `AtRisk` |
| Patch-only difference (same major/minor) | `Safe` |
| Versions equal | `UpToDate` |
| Unparseable value, a downgrade, or a stable version facing a prerelease | `Unknown` |

`AtRisk` is currently driven by requirement mismatches: without changelog data
there is no "deprecation note" signal yet, so it means "a bump your declared
range does not cover". The function is pure and is the most heavily tested part
of the crate (`src/classifier/tests.rs`).

### Prerelease filtering and the stable target

`latest_stable(iter)` returns the highest non-prerelease version. The crates.io
client uses it so the target defaults to the latest stable release: it prefers
`crate.max_stable_version`, otherwise the highest non-prerelease, non-yanked
entry in `versions`, and only falls back to `max_version` for a prerelease-only
crate. npm's `dist-tags.latest` is already stable. When a dependency's declared
version is itself a prerelease, `classify_update` still compares it against the
stable target (for example `1.0.0-rc.1` → `1.0.0` is `Safe`); a stable current
version is never pointed at a prerelease (`Unknown`).

### AI summaries (`src/ai/`)

`Summarizer::new(--ai)` detects a provider from `ANTHROPIC_API_KEY`,
`OPENAI_API_KEY`, or `RUSTYDEPS_AI_ENDPOINT` and logs which one applies. Every
`AtRisk`/`Breaking` dependency receives a templated one-line summary
(`Safe`/`UpToDate`/`Unknown` get none). No outbound LLM calls are made yet —
live, changelog-aware summaries are Phase 5; the fallback is always present, so
the report has text to show regardless.

> Superseded: Milestone 0.3.0 replaced this scaffold with live per-library
> analyses and removed the templated fallback. See
> [[ai-changelog-summarizer]].

### CLI banner and offline mode

`scan` prints the "Full Block Solid" ASCII banner (Rust-orange logo, cyan
version badge, light-grey tagline) on text runs; it is suppressed for
`--format json` and colours drop out automatically when stdout is not a
terminal. `--offline` (alias `--no-fetch`) skips registry lookups entirely and
classifies from local constraints only, which makes every dependency `Unknown`
by design.

## Design decisions

- `classify_update` is a pure function of two strings so it is trivially
  testable and free of I/O.
- `ClassifiedDependency::latest_version` is an `Option`, not a bare `String`, so
  the `Unknown` state (no usable registry data) is representable rather than
  smuggled through an empty string.
- Prerelease filtering lives in `latest_stable` (target selection) and in
  `classify_update` (comparison), keeping both rules unit-testable.
- The AI module is scaffold-only for now: pattern detection and templated
  fallback, with live providers deferred to Phase 5 as agreed.

## Verification

`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` (47 tests) all pass. A live scan classified real upgrades across
Cargo and npm, `--offline`/`--no-fetch` ran with no network, and `--format
json` emitted no banner. See the [implementation report](implementation-report.md)
for detail.

## Related

- [[architecture]] — the module this feature implements.
- [[roadmap]] — Phase 3 of six.
- [implementation report](implementation-report.md) — file-by-file report,
  tests, and follow-ups.
- [[html-report-spec]] — the Phase 4 report that consumes `UpdateType`.
- [[index]] — knowledge-graph entrypoint.
