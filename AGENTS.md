# AGENTS.md

Instructions for any coding agent working in this repo — Claude Code, Codex,
opencode, or others. Tool-specific files (e.g. `CLAUDE.md`) point here for the
shared flow and only add mechanics specific to that tool on top.

## Project state

RustyDeps is a high-performance, cross-platform CLI tool written in **Rust**
that scans local repositories for dependency manifests (`Cargo.toml`,
`package.json`, `go.mod`, ...), evaluates available version upgrades (SemVer) by
querying the official registries in parallel, and generates an interactive HTML
report with optional AI-driven changelog summaries. Full scope and constraints:
`doc/wiki/architecture.md` and `doc/raw/rustydeps-spec.md`.

The repository is at the **scaffolding stage**: the agent directives, `doc/`
knowledge base, and OKF LLM-Wiki exist, but the Rust crate (`Cargo.toml`,
`src/`) has not been initialized yet. Delivery is tracked in six phases in
`doc/wiki/roadmap.md` (currently: Phase 1, "Rust Setup & CLI").

## Tech stack and principles

- **Language:** Rust (2021/2024 edition). Idiomatic, safe, and high-performance
  code.
- **Architecture:** trait-based — a `PackageParser` trait decouples the
  ecosystem parsers (Cargo, npm, ...). Asynchronous concurrency with `tokio`;
  HTTP client `reqwest`; filesystem traversal with `ignore` (which respects
  `.gitignore` automatically), not hand-rolled walks.
- **CLI:** `clap` using the derive-macro pattern.
- **Documentation:** the LLM-Wiki lives in `doc/wiki/` (atomic Markdown, human-
  and LLM-friendly, OKF-compliant). See `doc/wiki/index.md`.
- **Raw knowledge:** source documents, initial specs, and prompt files live in
  `doc/raw/` and are immutable.
- **Skills and validation tooling:** pre-existing skills and Python validation
  scripts live in `.claude/skills/`. Scan them on startup.

## Code guidelines and quality

- **Error handling:** `thiserror` for internal modules and libraries; `anyhow`
  for CLI binary handlers. No loose `unwrap()` in production code — use `?` or
  an explicit `match`/`if let`.
- **Formatting and lints:** `cargo fmt` for layout; zero-warning policy under
  `cargo clippy`.
- **Tests:** unit tests are required for every Rust module.

## Commands

Once the crate is initialized (`cargo init`), the standard commands are:

- `cargo build` — compile the CLI
- `cargo run -- <args>` — run the CLI locally
- `cargo test` — run unit tests
- `cargo fmt --all` — format the workspace
- `cargo clippy --all-targets -- -D warnings` — lint with a zero-warning gate

There is no crate yet; these commands apply from Phase 1 onward.

## Workflow

Before starting any task, read `doc/wiki/index.md` and follow this flow:

1. Read documentation — the wiki and whatever raw document you were pointed at.
2. Analyze and plan the change against the current scope/phase
   (`doc/wiki/roadmap.md`).
3. Implement.
4. Ingest the change into `doc/wiki/` (update pages, log it in
   `doc/wiki/log.md`) before considering the work done. Use the `.claude/skills/`
   Python tools when building or validating the wiki.
5. Open a draft PR/MR: branch off the integration branch (`development`) as
   `feat/<scope>`, `fix/<scope>`, or `hotfix/<scope>`; never commit or push
   directly to a protected branch (`main`).

**This loop is mandatory for every feature/bug, not just large ones.** Steps 4
(wiki ingestion) and 5 (draft PR/MR) are not optional cleanup — don't report a
task as finished until both are done or the user has explicitly said to skip
them for that task.

### Wiki tooling

- Validate the wiki before every documentation commit:
  `python3 .claude/skills/wiki-scribe/scripts/validate_okf.py`. The script
  expects a `wiki/` bundle at its parent root; when validating `doc/wiki/`,
  stage a shim (see `doc/wiki/okf-compliance.md`) so the unmodified script runs
  against the bundle. Never commit around a validation failure.
- Keep a crash-safe work journal for multi-step tasks by applying the
  `work-journal` skill (`tmp/<slug>/`, git-ignored).

### Development report

At the end of every development step, write a
`doc/raw/development-report-<TIMESTAMP>.md` capturing: changes made, tests
executed, technical debt, and next recommended actions.

## Repo structure

- `doc/raw/` — immutable source documents. Never edit a file here after it is
  created; a superseded idea gets a new dated file instead.
- `doc/wiki/` — living knowledge base (llm-wiki-style, OKF-compliant). This is
  what to read for current project understanding; see `doc/wiki/index.md`.
- `.claude/` — pre-existing skills and Python validation scripts.
- `.planning/` — GSD workflow state (if present); don't hand-edit state files.
- `src/` — Rust source (not yet created; appears in Phase 1).
- `tmp/` — local, git-ignored agent scratch space (work journals). Never commit.

## Notes

- The crate is ESM-free by nature: single binary, `edition = "2021"` (or newer)
  in `Cargo.toml`.
- No path aliases; modules are wired through `mod` declarations.
