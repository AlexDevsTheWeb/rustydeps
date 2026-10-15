---
title: "RustyDeps Wiki — Index"
type: index
description: "Entrypoint and knowledge-graph map for the RustyDeps LLM-Wiki."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-15
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

## Conventions

- One topic per page; atomic and self-contained.
- Every page carries OKF YAML frontmatter (see [[okf-compliance]]).
- Every page links out and is linked from here, so no page is orphaned.
- `doc/raw/` is read-only input; documentation lives here in `doc/wiki/`.
