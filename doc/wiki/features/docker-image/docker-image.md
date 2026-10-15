---
title: "Docker Image & CI Publishing"
type: feature
description: "Multi-stage Alpine image for rustydeps, a GHCR build/publish workflow, and the README Docker section for CI usage."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-15
updated: 2026-10-15
tags: [feature, docker, ci, github-actions, ghcr, packaging]
---

# Docker image & CI publishing

## Motivation

Issue #15 asked for an official `rustydeps` image so CI pipelines (GitLab CI,
GitHub Actions, Bitbucket, Jenkins) can run a scan without a Rust toolchain.
Installing from source (`cargo install --git ...`) compiles the crate on every
job, and on ephemeral runners with no shared cache that cost repeats on every
pipeline. The issue also carried two hard requirements that constrain the
result: the image must include CA certificates for registry lookups, and it must
provide `sh`, because GitLab CI and similar systems need a shell inside the job
image.

## What shipped

- **`Dockerfile`** — two-stage musl build: `rust:1-alpine` compiles the release
  binary, `alpine:3` carries the binary, `ca-certificates` and the shared
  libraries it links against.
- **`.dockerignore`** — trims the build context to `Cargo.toml`, `Cargo.lock`,
  `src/` and `templates/`.
- **`.github/workflows/docker.yml`** — multi-arch (`linux/amd64`, `linux/arm64`)
  build on every `main` push, `v*` tag and pull request; publishes to
  `ghcr.io/alexdevstheweb/rustydeps` except on pull requests, where it loads the
  image and smoke-tests it instead.
- **`README.md`** — a Docker section (pull/run commands, GitLab CI and GitHub
  Actions snippets, image notes), a workflow badge, and a note in *Releasing*
  that a `v*` tag now publishes the image too.

## Components

### Dockerfile

| Stage | Base | Purpose |
|-------|------|---------|
| `builder` | `rust:1-alpine` | `apk add musl-dev openssl-dev pkgconf`; compiles the dependency graph, then the crate, with `cargo build --release --locked`. |
| `runtime` | `alpine:3` | `apk add ca-certificates libssl3 libgcc`; copies `/usr/local/bin/rustydeps`; `WORKDIR /workspace`; `ENTRYPOINT ["rustydeps"]`, `CMD ["scan", "."]`. |

The dependency graph is compiled on its own layer against a stub `src/main.rs`
before the real sources are copied, so editing `src/` does not rebuild every
third-party crate. `templates/report.html` is copied in the builder only: Askama
embeds the template into the binary at compile time, so the runtime stage needs
no template directory.

`CMD ["scan", "."]` resolves to `/workspace`, the image's working directory, so
`docker run --rm -v "$PWD:/workspace" IMAGE` scans the mount and writes
`rustydeps-report.html` next to the user's manifests.

### Workflow

Triggered by pushes to `main`, pushes of `v*` tags, and pull requests targeting
`main`. Steps: checkout, compute a lowercase image reference, QEMU + Buildx
setup, GHCR login (`GITHUB_TOKEN`, skipped on pull requests), `docker/metadata-action`
tag extraction, and `docker/build-push-action`.

| Context | Platforms | Push | Tags |
|---------|-----------|------|------|
| PR to `main` | `linux/amd64` | no (`load: true`) | `pr-<n>` |
| Push to `main` | `linux/amd64`, `linux/arm64` | yes | `main`, `latest` |
| Push of `v*` | `linux/amd64`, `linux/arm64` | yes | `1.2.3`, `1.2`, `latest` |

Pull requests load the single-platform image into the runner's daemon and run
`--version` plus an offline scan of the checkout, so a missing shared library or
a broken entrypoint fails review rather than surfacing after merge.

## Design decisions

- **Alpine rather than distroless.** Distroless has no shell, which the issue
  explicitly rules out for GitLab-style job images. Alpine contributes `sh`
  (BusyBox) and is still small.
- **`alpine:3` rather than the briefed `alpine:3.19`.** 3.19 is past Alpine's
  release window by now, and the rolling `alpine:3`/`rust:1-alpine` pair tracks
  the same Alpine release, keeping the build-stage libc and run-stage libc in
  step instead of pinning the runtime to an older musl than the builder.
- **OpenSSL in the runtime image.** On Linux `reqwest` uses `native-tls`, so the
  binary links `openssl-sys` — verified with
  `cargo tree --target x86_64-unknown-linux-gnu -i openssl-sys`. Hence
  `openssl-dev` while building and `libssl3`/`libcrypto3` at run time. Switching
  the client to `rustls` would remove that dependency but is a product-wide TLS
  change, out of scope here (see *Not done*).
- **Runs as root.** CI mounts a workspace owned by an arbitrary host user; a
  fixed non-root uid could not write the report into it. Recorded in the README
  so the choice is deliberate rather than accidental.
- **`push` and `load` are mutually exclusive.** A multi-platform image cannot be
  loaded into the local daemon, hence single-platform PR builds with `load`.
- **Image reference lowercased in a step.** GHCR rejects mixed-case repository
  names and the owner (`AlexDevsTheWeb`) is mixed case, so the workflow derives
  `ghcr.io/alexdevstheweb/rustydeps` with `tr` rather than trusting
  `github.repository_owner`. The test uses `tr` instead of bash's `${var,,}` so
  it is valid under any shell.
- **Cache via `type=gha`.** Cross-arch compilation runs under QEMU, so the
  dependency layer needs a cache to stay inside a sane job duration.

## Verification

- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test` (85 tests) are clean.
- `cargo build --release --locked` — the builder stage's exact command — succeeds.
- The smoke-test commands were run against the local release binary: `--version`
  prints `rustydeps 0.1.0`, and `scan . --offline --no-open -o <tmp>` exits 0 and
  writes a 65 KB report.
- `.github/workflows/docker.yml` parses as YAML; its triggers, permissions, step
  list and metadata tags were inspected, and both `run:` blocks pass `bash -n`;
  the lowercase step was executed locally and yields
  `name=ghcr.io/alexdevstheweb/rustydeps`.
- **Not verified locally:** `docker build` / `docker run`. No container runtime
  is installed on the development machine, so the image itself is first built by
  the new workflow. See the [implementation report](implementation-report.md).

## Related

- [[release-automation]] — the sibling tag-triggered release pipeline for native
  binaries and the README it introduced.
- [[architecture]] — the CLI and modules the image packages.
- [[roadmap]] — the delivery phases; like release tooling this is not a phase.
- [implementation report](implementation-report.md) — file-by-file detail.
- [[index]] — knowledge-graph entrypoint.
