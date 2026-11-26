---
title: "v0.3.1 — GitHub Pages Landing Page Refresh"
type: feature
description: "The static landing page updated to v0.3.1: real install commands, AI/Ollama/cache features, a report drawer preview, and a corrected CLI banner."
status: active
version: 0.3.1
license: CC-BY-4.0
created: 2026-11-26
updated: 2026-11-26
tags: [feature, landing-page, docs, github-pages, v0.3.1]
---

# v0.3.1 — landing page refresh

## Motivation

`docs/index.html` (the GitHub Pages site) still advertised v0.1.0. Its hero
install was `cargo install --path .`, its terminal demo used a non-existent
`--report` flag, its feature grid predated the AI work, and it claimed an
`exit 1` CI gate the binary does not implement. The brief is
`doc/raw/2026-11-26/2026-11-26-update-landing-page.md`.

The page is a single tracked file, `docs/index.html`; there is no site build or
asset pipeline. Tailwind is loaded from its CDN, and the existing inline
handlers provide the copy toast and install-tab switching.

## Content changes

- **Version and licensing** — header, hero, and footer now say `v0.3.1`; the
  licence is `MIT OR Apache-2.0`, matching `Cargo.toml`.
- **Install** — the hero shows the tagged Cargo install
  (`cargo install --git … --tag v0.3.1`) and the published container command
  (`ghcr.io/alexdevstheweb/rustydeps:0.3.1`). The GHCR tag omits the `v`
  because `.github/workflows/docker.yml` uses `type=semver,pattern={{version}}`.
- **Feature grid** — six cards cover multi-ecosystem scanning, SemVer risk
  classes, four-section AI briefings, local Ollama plus BYOK providers, disk
  cache with the fatal-error circuit breaker, and the self-contained report.
- **AI section** — an Ollama `config.json` example, the
  `--ai --ai-provider ollama` command, `--no-cache`, `cache clear`, the cache
  path, and the provider roster (Ollama, Gemini, OpenAI, OpenRouter, custom).
- **Terminal demo** — mirrors the real output contract: `Scanned '<label>': N
  dependencies (<eco>)` lines, the `94 dependencies: …` tally, the
  `AI analyses: …` line, and the report-written/auto-open lines. The stale
  `--report` flag and the false exit-code claim were removed.
- **Report preview** — adds an expanded drawer for `@prisma/client` with the
  four real prompt headings and the `⚡ Cached (llama3.2:3b)` badge, plus the
  existing filters, risk dots, copy buttons, and Update All.
- **Quickstart** — Cargo/Docker tabs reuse the page's existing `switchTab`,
  real scan flags (`-o`, `--no-open`, `--format json`, `--offline`), an optional
  AI step, and a Docker CI recipe.
- **Links** — release `v0.3.1`, the GHCR package page, and the repository README
  replace the stale anchors.

The page no longer states a fixed report size: the rendered file depends on the
scan payload, so the copy says "single file, no CDN" instead of a byte count.

## CLI banner correction

`print_banner` in `src/main.rs` still read "Rust, Node & Python". It now lists
"Rust, Node, Python, Go & PHP", matching the five supported ecosystems and the
terminal demo on the landing page.

## Verification

- HTML structure parsed with `html.parser`: no unclosed or mismatched tags.
- Headless Chromium at 1365 px and 390 px: no horizontal overflow, both install
  panes switch correctly, and 15 copy controls pass their `data-copy` value to
  the shared clipboard handler (headless clipboard permissions are unavailable,
  so the handler was exercised at the function boundary).
- Desktop and mobile screenshots reviewed for the hero, feature grid, AI
  section, report preview, and quickstart.
- Release, GHCR, and README links returned HTTP 200.
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo test` (144 unit + 4 integration) pass.

See [[v0.3.1-surgical-fixes]] for the runtime behavior this page describes.
