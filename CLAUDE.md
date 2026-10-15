# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

Shared project state, tech stack, code guidelines, workflow, and repo structure
live in `AGENTS.md` — read that first; it's written to be agent-agnostic (Claude
Code, Codex, opencode, ...) so it stays the single source of truth as tooling
changes. This file only adds Claude-Code-specific mechanics and the wiki
schema on top of it.

## Claude-Code-specific mechanics

- **Skills:** pre-existing skills live in `.claude/skills/` and should be
  discovered on startup. The ones that matter for this repo:
  - `wiki-scribe` — write/update `doc/wiki/` pages and validate them.
  - `work-journal` — crash-safe task journal under `tmp/<slug>/`.
  - `raw-ingest-ship-loop` / `delivery-engineer` — raw doc → implement → commit
    → draft MR/PR → document.
- **Helper scripts (stdlib-only Python 3):**
  - `.claude/skills/wiki-scribe/scripts/validate_okf.py` — mandatory before any
    documentation commit.
  - `build_techdocs.py`, `check_techdocs_links.py`, `check_wiki_coverage.py`,
    `obsidian_links.py`, `publish_docs_branch.py` — TechDocs conversion, link
    checks, coverage, publishing (only when their prerequisites exist).
  - `.claude/skills/work-journal/scripts/journal.py` — create/inspect journals.
- **Wiki bundle path quirk:** the wiki-scribe scripts hardcode a `wiki/`
  directory two levels above `scripts/` (they were authored for another repo
  layout). In this repo the bundle is `doc/wiki/`. Validate it with a disposable
  staging shim — copy `scripts/` next to a `wiki` symlink to `doc/wiki/` — rather
  than editing the scripts. See `doc/wiki/okf-compliance.md`.

## Wiki page schema

Every page in `doc/wiki/` starts with flat YAML frontmatter (the validator's
parser is flat-only, so no nested maps):

```yaml
---
title: "Page title"
type: <page type>
description: "One-line summary."
status: draft | active | deprecated
version: 0.1.0
license: CC-BY-4.0
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [tag1, tag2]
---
```

Required: `type`. Recommended: `description`. Page types:

| `type`         | Use                                                          |
|----------------|--------------------------------------------------------------|
| `index`        | Catalogue / knowledge-graph entrypoint (`index.md`).         |
| `architecture` | System and module design.                                    |
| `roadmap`      | Phase and milestone tracking.                                |
| `convention`   | Standards and rules (e.g. OKF compliance).                   |
| `reference`    | Lookup material (DTOs, commands, schemas).                   |
| `log`          | Append-only history (`log.md`).                              |
| `report`       | Implementation / development reports.                        |

## Rust/build details

- No `Cargo.toml` yet — the crate is created in Phase 1 (`doc/wiki/roadmap.md`).
- Target layout once initialized: `src/main.rs` (CLI entry, `clap` derive),
  `src/parser/` (`PackageParser` trait + per-ecosystem impls), `src/registry/`
  (`reqwest` clients), `src/report/` (HTML engine), `src/ai/` (summarizer), and
  `src/model.rs` (shared DTOs `Dependency`, `UpdateType`).
- Errors: `thiserror` in libraries/modules, `anyhow` in the binary. No
  `unwrap()` in production paths.
- Gates: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`.
