---
title: "Implementation Report — Landing Page v0.3.1"
type: report
description: "File-by-file delivery report for the v0.3.1 GitHub Pages refresh and the CLI banner ecosystem correction."
status: done
version: 0.3.1
sources: [2026-11-26-update-landing-page]
license: CC-BY-4.0
created: 2026-11-26
updated: 2026-11-26
tags: [implementation-report, landing-page, docs, github-pages]
---

# Implementation Report — Landing Page v0.3.1

## Summary

Implemented `doc/raw/2026-11-26/2026-11-26-update-landing-page.md` on
`fix/landing-page-v0.3.1`. The single tracked site file `docs/index.html` now
describes v0.3.1 with real commands, the AI/Ollama/cache feature set, and a
report drawer preview. One CLI string was corrected so the terminal demo is
truthful.

## Done

- **`docs/index.html`** — rewritten in place:
  - v0.3.1 badges, dual-license labels, release/GHCR/README links.
  - Hero install cards for the tagged Cargo install and the `0.3.1` container.
  - Six-card feature grid (ecosystems, SemVer, AI summary, local/BYOK AI,
    cache + breaker, self-contained report).
  - New AI section with the Ollama config, scan/cache commands, cache path, and
    provider chips.
  - Terminal demo rebuilt from the real CLI strings; removed the invented
    `--report` flag, stale "54 KB" sizes, and the false exit-code claim.
  - Report preview gained an expanded AI drawer with the four prompt headings
    and the cached-model badge.
  - Quickstart with working Cargo/Docker tabs, real scan flags, optional AI
    step, and a CI recipe.
  - Responsive fixes: command wrapping, `min-w-0` flex/grid children, wrapping
    project headers, and `scroll-mt-16` anchor offsets.
- **`src/main.rs`** — `print_banner` subtitle now lists Rust, Node, Python, Go
  and PHP.

## Verification

- `html.parser` stack check — no unclosed or mismatched elements.
- Headless Chromium at 1365×900 and 390×844 — `scrollWidth == innerWidth`
  (no horizontal overflow); Cargo/Docker tab switch toggles the correct panel;
  all 15 `data-copy` controls route their exact command to the copy handler.
- External links — release tag, GHCR package, and README returned HTTP 200.
- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 144 unit + 4 integration passed.
- OKF validator — PASSED.

## Merge request

Draft PR [#32](https://github.com/AlexDevsTheWeb/rustydeps/pull/32)
(`fix/landing-page-v0.3.1` → `development`).

## Technical debt

- The page still depends on the Tailwind CDN and Google Fonts; it is not fully
  offline like the generated report.
- The report preview is hand-maintained markup; it can drift from
  `templates/report.html` again.
- The README banner and badges still show v0.2.0, outside this brief's scope.

## Next actions

- Merge through the draft PR into `development`.
- Refresh the README version banner/badges in a follow-up so the repository and
  the landing page agree.
