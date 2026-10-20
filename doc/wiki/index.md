---
title: "RustyDeps Wiki — Index"
type: index
description: "Entrypoint and knowledge-graph map for the RustyDeps LLM-Wiki."
status: active
version: 0.3.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-19
tags: [index, rustydeps, llm-wiki]
---

# RustyDeps Wiki

This is the entrypoint to the RustyDeps knowledge base: an atomic, human- and
LLM-friendly wiki that is compliant with the Open Knowledge Foundation (OKF)
principles. Read this page first; it links every other page.

RustyDeps is a high-performance Rust CLI that scans local repositories for
dependency manifests, evaluates SemVer upgrades against the official registries,
and produces an interactive HTML report with optional AI changelog summaries.

## Map

| Page | What it covers |
|------|----------------|
| [[architecture]] | System and module design, the `PackageParser` trait, registry clients, report engine, AI module, and core DTOs. |
| [[html-report-spec]] | Finalized UX/UI contract for the interactive HTML report: industrial dark mode, project-centric grouping, risk indicators, and interactions. |
| [[roadmap]] | The six delivery phases from Rust setup to multi-ecosystem expansion. |
| [[okf-compliance]] | Open formats, open licensing, metadata traceability, and how to validate this wiki. |
| [[log]] | Append-only history of wiki changes. |

## Features

| Page | What it covers |
|------|----------------|
| [[phase-2-parsers-registry]] | Manifest parsers (`PackageParser`, Cargo, npm) and the crates.io/npm registry clients; [implementation report](features/phase-2-parsers-registry/implementation-report.md). |
| [[phase-3-semver-classifier]] | SemVer classifier (`Safe`/`AtRisk`/`Breaking`/`UpToDate`/`Unknown`), latest-stable targeting, templated AI summaries, CLI banner, and `--offline` mode; [implementation report](features/phase-3-semver-classifier/implementation-report.md). |
| [[phase-4-html-report]] | Self-contained interactive HTML report: project cards, risk dots, inline drawer, filters, and update commands; [implementation report](features/phase-4-html-report/implementation-report.md). |
| [[phase-5-ecosystems]] | PyPI, Go, and Composer parsers and registry clients behind the existing traits; [implementation report](features/phase-5-ecosystems/implementation-report.md). |
| [[release-automation]] | Tag-triggered GitHub Actions release pipeline (Linux/macOS binaries) and the full README; [implementation report](features/release-automation/implementation-report.md). |
| [[docker-image]] | Multi-stage Alpine `Dockerfile`, `.dockerignore`, and the GHCR multi-arch build/publish workflow for CI usage; [implementation report](features/docker-image/implementation-report.md). |
| [[cli-scanning-ux]] | Interactive scan progress (spinners, a per-repository `pos/len` counter, per-manifest metrics), the non-TTY/JSON fallback, and the v0.2.0 version bump; [implementation report](features/cli-scanning-ux/implementation-report.md). |
| [[ui-font-branding]] | HTML report type-scale bump (12/13/14/16px) plus an inline SVG brand logo and an embedded data-URI SVG favicon, keeping the report self-contained; [implementation report](features/ui-font-branding/implementation-report.md). |
| [[report-ux-refinements]] | Orphan-free CLI progress (`MultiProgress` + suspended writes) and report card refinements (collapsed by default, clean headers, path + Update All in the body, rows sorted by risk); [implementation report](features/report-ux-refinements/implementation-report.md). |
| [[ai-changelog-summarizer]] | Milestone 0.3.0 per-library AI briefings: layered BYOK JSON config, OpenAI/Gemini/Ollama client, per-version disk cache, graceful fallback, and the report AI card; [implementation report](features/ai-changelog-summarizer/implementation-report.md). |

## Source documents

- `doc/raw/rustydeps-spec.md` — core architectural requirements.
- `doc/raw/2026-10-14-scaffolding/2026-10-14-scaffolding.md` — the scaffolding
  task that created this wiki.
- `doc/raw/2026-10-15/2026-10-15-phase2.md` — the Phase 2 brief
  implemented by [[phase-2-parsers-registry]].
- `doc/raw/2026-10-15/2026-10-15-phase3.md` — the Phase 3 brief
  implemented by [[phase-3-semver-classifier]].
- `doc/raw/2026-10-15/2026-10-15-phase4.md` — the Phase 4 brief
  implemented by [[phase-4-html-report]].
- `doc/raw/2026-10-15/2026-10-15-phase5.md` — the Phase 5 brief
  implemented by [[phase-5-ecosystems]].
- `doc/raw/2026-10-15/2026-10-15-phase7-release.md` — the release/README brief
  implemented by [[release-automation]].
- `doc/raw/2026-10-15/opencode-prompt-08-issue15-docker.md` — the Docker image
  and GHCR publishing brief (issue #15) implemented by [[docker-image]].
- `doc/raw/2026-10-15/2026-10-15-issue15-docker.md` — development report for the
  Docker work.
- `doc/raw/2026-10-15/opencode-prompt-08-issue15-docker.md` — the issue #15
  Docker image brief implemented by [[docker-image]].
- `doc/raw/2026-10-17/2026-10-17-issue19.md` — the issue #19 CLI scanning UX
  brief implemented by [[cli-scanning-ux]].
- `doc/raw/2026-10-17/2026-10-17-issue21.md` — the issue #21 report font
  scaling and branding brief implemented by [[ui-font-branding]].
- `doc/raw/2026-10-17/2026-10-17-issue21-fix.md` — the issue #21 follow-up
  (CLI progress rendering + report card refinements) implemented by
  [[report-ux-refinements]].
- `doc/raw/2026-10-19/2026-10-19-v0.3.0.md` — Milestone 0.3.0 brief for the
  per-library AI summarizer and BYOK config implemented by
  [[ai-changelog-summarizer]].

## Conventions

- One topic per page; atomic and self-contained.
- Every page carries OKF YAML frontmatter (see [[okf-compliance]]).
- Every page links out and is linked from here, so no page is orphaned.
- `doc/raw/` is read-only input; documentation lives here in `doc/wiki/`.
