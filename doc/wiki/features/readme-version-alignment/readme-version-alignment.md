---
title: "README Version Alignment — v0.3.1"
type: feature
description: "README badges, Cargo install tag, Docker tags, and release examples aligned with the shipped v0.3.1 release."
status: active
version: 0.3.1
license: CC-BY-4.0
created: 2026-11-27
updated: 2026-11-27
tags: [feature, readme, docs, badges, v0.3.1]
---

# README version alignment

## Motivation

`README.md` still advertised v0.2.0 after the v0.3.1 release: the banner, the
version badge, the Docker examples, and the release instructions all pointed at
an older tag. The brief is
`doc/raw/2026-11-27/2026-11-27-v0.4.0-and-fixes.md` (Worktree 1).

## Changes

- Banner version `v0.2.0` → `v0.3.1`.
- Shields.io version badge `0.2.0` → `0.3.1`.
- Tagged installation: `cargo install --git … --tag v0.3.1`.
- Docker run, Compose, GitLab CI, and GitHub Actions examples now use
  `ghcr.io/alexdevstheweb/rustydeps:0.3.1`.
- Docker tag documentation now lists the tags the workflow actually produces:
  `0.3.1`, `0.3`, and `latest` — `docker/metadata-action` uses
  `type=semver,pattern={{version}}` and `{{major}}.{{minor}}`.
- Release instructions use `v0.3.1` in the tag example and commands.

No source code changed. The release workflow and Docker workflow already derive
their tags from the pushed `v*` tag; only the README lagged behind.

## Verification

`grep -nE '0\.[0-9]+\.[0-9]+|v0\.[0-9]' README.md` leaves only intentional
v0.3.1 references. `cargo fmt --all --check` is unaffected by a Markdown edit.

## Related

- [[landing-page-v0.3.1]] — the GitHub Pages site already moved to v0.3.1.
- [[release-automation]] — the tag-driven release pipeline.
- [[docker-image]] — the multi-arch GHCR image and its tag rules.
- [[implementation-report]] — delivery record.
- [[index]] — knowledge-graph entrypoint.
