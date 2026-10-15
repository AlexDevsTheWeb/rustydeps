---
title: "Release Automation & README"
type: feature
description: "A tag-triggered GitHub Actions release workflow that builds Linux/macOS binaries, plus a full README."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [feature, release, ci, github-actions, readme]
---

# Release automation & README

## Motivation

By the end of Phase 5 the tool scanned five ecosystems and produced a report, but
the repository had no way to distribute it: the README was a three-line stub, and
there was no CI to build or publish binaries. This change adds a tag-triggered
release pipeline so publishing `v0.1.0` produces ready-to-run Linux and macOS
binaries, and rewrites the README so a first-time visitor can install, run, and
understand the tool.

## What shipped

- **README** (`README.md`) — ASCII banner, v0.1.0/license/Rust badges, the live
  website link, a supported-ecosystems table, a risk legend, quickstart and usage
  blocks, a feature overview, and the release/development/license sections.
- **Release workflow** (`.github/workflows/release.yml`) — a two-job pipeline
  that builds and publishes binaries when a `v*` tag is pushed.

## Components

### README

Summarises the five supported ecosystems (Cargo, npm, PyPI, Go, Composer) with
their manifests and registries, the `breaking`/`at-risk`/`safe`/`up-to-date`/
`unknown` classification, and the CLI features added through Phase 5
(`--offline`/`--no-fetch`, `--no-open` auto-open, `--format json`, the
self-contained HTML report, `--ai`).

### Release workflow

Triggered by `push` of a `v*` tag (for example `v0.1.0`):

- **build** — a matrix over `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`,
  and `aarch64-apple-darwin`. Each job installs the target toolchain, runs
  `cargo build --release --locked`, packages the binary with the README and
  licenses into a `.tar.gz`, writes a `.sha256`, and uploads the artifacts.
- **release** — downloads the artifacts, generates release notes (a changelog
  since the previous tag, the asset list, and install steps), and creates the
  GitHub Release with the archives and checksums attached (`contents: write`).

The tag is not pushed as part of this change: pushing it later runs the release
from a commit that actually contains the workflow.

## Design decisions

- Tag-triggered (`v*`) rather than manual dispatch: matches how the roadmap
  versions the tool (the brief's tag example was `v0.1.0`).
- Linux `x86_64-unknown-linux-gnu` (glibc) and both macOS arches via
  `macos-latest`, so Intel and Apple-Silicon users get a native binary.
- Release notes are generated in a shell step from `git log` since the previous
  tag rather than relying on the action's note generation, so the changelog and
  install instructions are deterministic.
- The tag push is left to the developer (the release is a public, outward-facing
  action), which is why this PR ships only the workflow and README.

## Verification

- Workflow YAML parses; `jobs`, the tag trigger, and the three matrix targets were
  inspected, and both `run:` blocks pass `bash -n`.
- No Rust code changed; the gates stay green: `cargo fmt --all --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test` (85 tests).
- The README renders the banner, badges, tables, and code blocks as expected.

See the [implementation report](implementation-report.md).

## Related

- [[architecture]] — the CLI and modules the README documents.
- [[roadmap]] — the delivery phases; this is release tooling, not a phase.
- [[phase-5-ecosystems]] — the ecosystems the README advertises.
- [implementation report](implementation-report.md) — file-by-file detail.
- [[index]] — knowledge-graph entrypoint.
