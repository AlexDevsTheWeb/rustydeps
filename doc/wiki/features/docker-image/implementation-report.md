---
title: "Implementation Report — Docker Image & CI Publishing"
type: report
description: "File-by-file report of the multi-stage Dockerfile, .dockerignore, GHCR workflow, and README Docker section."
status: done
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [implementation-report, docker, ci]
---

# Implementation Report — Docker image

## Summary

`doc/raw/2026-10-15/opencode-prompt-08-issue15-docker.md` implements issue #15
("publish an official Docker image for use in CI pipelines"). Shipped on branch
`feat/docker-support`: a multi-stage Alpine `Dockerfile`, a `.dockerignore`, a
`.github/workflows/docker.yml` that builds and publishes multi-arch images to
GHCR, and a Docker section in `README.md`.

Two specifics in the brief were adjusted, both recorded in
[[docker-image]]: the runtime base is `alpine:3` instead of the briefed
`alpine:3.19` (3.19 is past its release window, and the rolling tag keeps the
builder and runtime on the same Alpine release), and the PR job runs a smoke
test rather than only compiling. Distroless was rejected because the issue
requires a shell in the image.

## Done

- `Dockerfile` (new) — `rust:1-alpine` builder (`musl-dev`, `openssl-dev`,
  `pkgconf`; dependency-graph layer against a stub `main.rs`; then `src/` +
  `templates/`) and an `alpine:3` runtime (`ca-certificates`, `libssl3`,
  `libgcc`) exposing `ENTRYPOINT ["rustydeps"]`, `CMD ["scan", "."]`,
  `WORKDIR /workspace`, and OCI labels.
- `.dockerignore` (new) — excludes `target/`, `.git/`, `.github/`, `doc/`,
  `docs/`, `.claude/`, `tmp/`, `*.html`, `*.md`, and editor/OS noise, keeping
  `Cargo.toml`, `Cargo.lock`, `src/` and `templates/` in the context.
- `.github/workflows/docker.yml` (new) — build on `main` pushes, `v*` tags and
  PRs to `main` and `development`; QEMU + Buildx; GHCR login with `GITHUB_TOKEN`
  (`packages: write`); metadata tags `latest`/branch/semver/`pr-<n>`; publish
  `linux/amd64` + `linux/arm64` on non-PR events, single-platform `load` plus a
  `--version`/offline-scan smoke test on PRs; `type=gha` cache.
- `README.md` — Docker workflow badge; a `## Docker` section (multi-arch pull,
  run, and pinned-version examples, GitLab CI and GitHub Actions snippets, plus
  notes on CA certificates, root, `--no-open`, and tags); *Releasing* now states
  that a `v*` tag also publishes the image.
- Wiki: this subfolder, [[docker-image]], and updates to [[index]] and [[log]].

## Not done / deferred

- **No local image build.** No container runtime is installed on the development
  machine, so `docker build -t rustydeps:local .` and
  `docker run --rm -v "$(pwd):/workspace" rustydeps:local scan /workspace --offline`
  from the brief's gates could not be executed; the new workflow's PR job is what
  first builds and runs the image.
- **Docker Hub.** GHCR only; the issue mentioned Docker Hub as an alternative,
  not an additional target.
- **Non-root user.** Deferred so volume mounts from arbitrary CI uids stay
  writable.
- **Trivy/SBOM scanning, provenance attestations, and a Dependabot config for
  the Docker base images** are not part of this change.
- **`rustls` migration.** Would drop the OpenSSL runtime dependency and allow a
  `scratch`/distroless runtime; it changes TLS across every platform, so it was
  left out of a packaging change.

## Blocked

None.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 85 passed, 0 failed.
- `cargo build --release --locked` (the Dockerfile builder command) — succeeds.
- `./target/release/rustydeps --version` — `rustydeps 0.1.0`; and
  `./target/release/rustydeps scan . --offline --no-open -o /tmp/rustydeps-report.html`
  — exit 0, report written (the non-container half of the CI smoke test).
- `cargo tree --target x86_64-unknown-linux-gnu -i openssl-sys` — confirms
  `reqwest → native-tls → openssl-sys` on Linux, justifying `libssl3` in the
  runtime stage.
- `.github/workflows/docker.yml` parses as YAML (triggers, `permissions`,
  `jobs.image.steps` and the metadata tags inspected); both `run:` blocks pass
  `bash -n`; the image-reference step was executed locally and produced
  `name=ghcr.io/alexdevstheweb/rustydeps`.
- OKF validator PASSED (0 errors, 0 warnings).

## MR

Draft PR #16 into `development` from `feat/docker-support`:
<https://github.com/AlexDevsTheWeb/rustydeps/pull/16>.

## Related

- [[docker-image]] — the feature page for this work.
- [[release-automation]] — the native-binary release pipeline this complements.
- [[architecture]] — the CLI and modules the image packages.
