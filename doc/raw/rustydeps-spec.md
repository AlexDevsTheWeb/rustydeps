# RustyDeps — Core Specification

> Source document. Created 2026-10-14 during scaffolding. Immutable: a superseded
> revision gets a new dated file, never an edit of this one.

## 1. Purpose

RustyDeps is a high-performance, cross-platform CLI tool written in **Rust**.
It scans local repositories for dependency manifests (`Cargo.toml`,
`package.json`, `go.mod`, and later more ecosystems), evaluates available
version upgrades using SemVer against the official registries, and generates an
interactive HTML report. An optional AI module summarizes release notes and
changelogs so a developer can judge an upgrade without leaving the report.

The tool answers one question fast: *which of my dependencies are out of date,
how risky is each jump, and what changed?*

## 2. Architecture

### 2.1 CLI

- Built with `clap` using the derive-macro pattern.
- Accepts one or more root directories to scan, an output path for the HTML
  report, and flags to enable the AI summarizer and control concurrency.
- CLI handlers return `anyhow::Result`; internal modules return typed errors.

### 2.2 Filesystem walk

- Traversal uses the `ignore` crate so `.gitignore`, `.ignore`, and global
  ignore files are respected automatically; no hand-rolled recursive walk.
- Walks concurrently and collects discovered manifest paths.

### 2.3 Parser trait

- A `PackageParser` trait decouples ecosystem parsing from the rest of the
  pipeline. Each ecosystem (Cargo, npm, later PyPI/Go/Composer) implements the
  trait to turn a manifest into a list of dependencies (name, current version
  requirement, ecosystem).
- New ecosystems are added by implementing the trait, not by editing a match
  arm in the scanner (Open/Closed).

### 2.4 Registry clients

- One async client per registry, built on `reqwest`.
- Queries run concurrently under a `tokio` runtime, with bounded concurrency
  and per-registry rate limiting/politeness.
- Clients return the latest (and optionally all) published versions plus
  release metadata for changelog summarization.

### 2.5 SemVer classification

- Each dependency's current version is compared to the latest available using
  SemVer rules and the manifest's declared version requirement.
- Jumps are classified into three buckets:
  - **Safe** — patch/minor within the allowed range.
  - **At-Risk** (Risky) — minor/major outside the declared range but
    non-breaking-looking.
  - **Breaking** — major version change or a violation of the declared range.
- The classifier is pure and unit-testable: inputs are versions plus the
  requirement, output is an `UpdateType`.

### 2.6 Report engine

- Generates a single self-contained, interactive HTML report (filter/sort by
  ecosystem, severity, and repository).
- Templated from the in-memory result set; no network access required to view.
- Designed to be opened straight from the filesystem.

### 2.7 AI module

- Optional. Given a dependency and its version jump, fetches the relevant
  changelog/release notes and produces a short summary highlighting breaking
  changes and migration notes.
- Gated behind an explicit CLI flag so the default run stays offline and
  deterministic; failures here never fail the scan.

## 3. Shared DTOs

```rust
/// A single dependency found in a manifest.
pub struct Dependency {
    pub name: String,
    pub ecosystem: Ecosystem,       // Cargo, Npm, ...
    pub current: String,            // version or requirement as declared
    pub latest: Option<String>,     // resolved from the registry
    pub manifest_path: PathBuf,     // where it was found
    pub repository: PathBuf,        // owning repo root
}

/// Severity of an available upgrade.
pub enum UpdateType {
    Safe,
    AtRisk,
    Breaking,
}
```

## 4. Quality bars

- `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` clean.
- `thiserror` for module-level errors, `anyhow` at the binary boundary.
- No `unwrap()` in production paths.
- Unit tests for every module; the classifier and parsers are the highest-value
  test targets.

## 5. Out of scope (initially)

- Writing changes back to manifests.
- Auto-applying upgrades.
- Authentication against private registries (extension point via registry
  clients).
