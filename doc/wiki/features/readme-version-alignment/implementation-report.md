---
title: "Implementation Report — README Version Alignment v0.3.1"
type: report
description: "Delivery record for the README badge, install, Docker, and release version alignment."
status: done
version: 0.3.1
sources: [2026-11-27-v0.4.0-and-fixes]
license: CC-BY-4.0
created: 2026-11-27
updated: 2026-11-27
tags: [implementation-report, readme, docs, v0.3.1]
---

# Implementation Report — README Version Alignment

## Summary

Applied Worktree 1 of `doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md` on
`fix/readme-badges-v0.3.1`. `README.md` now matches the shipped v0.3.1 release
instead of v0.2.0.

## Done

- Banner and shields.io version badge moved to `v0.3.1` / `0.3.1`.
- Tagged Cargo install added: `cargo install --git … --tag v0.3.1`.
- Every Docker example (run, Compose, GitLab CI, GitHub Actions) uses the
  `0.3.1` GHCR tag.
- Docker tag note corrected to the tags `docker/metadata-action` emits
  (`0.3.1`, `0.3`, `latest`).
- Release instructions use `v0.3.1` in both the example and the commands.
- New wiki page [[readme-version-alignment]] and this report.

## Verification

- `git diff` — README-only changes; no source or workflow files touched.
- `cargo fmt --all --check` — clean.
- OKF validation — recorded in the development report.

## Technical debt

None introduced. The README still documents `--tag v0.3.1`; the next release
must bump it together with `Cargo.toml`.

## Next actions

Open the draft PR to `development` and record its link in the log.
