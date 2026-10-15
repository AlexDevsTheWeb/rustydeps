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
| `builder` | `rust:1-alpine` | `apk add openssl-dev openssl-libs-static pkgconf` (`gcc`, `musl-dev` and `ca-certificates` are already in the base image); compiles the dependency graph against a stub `src/main.rs`, then touches the entry point and builds the real crate with `cargo build --release --locked`, asserting `--version` output before the layer is accepted. |
| `runtime` | `alpine:3` | `apk add ca-certificates`; copies the statically linked `/usr/local/bin/rustydeps`; `WORKDIR /workspace`; `ENTRYPOINT ["rustydeps"]`, `CMD ["scan", "."]`. |

The dependency graph is compiled on its own layer against a stub `src/main.rs`
before the real sources are copied, so editing `src/` does not rebuild every
third-party crate. The stub has a trap: it is newer than the sources `COPY`
preserves mtimes for, so cargo's mtime-based freshness check would skip the real
build and leave the stub binary in place. The Dockerfile therefore runs
`touch src/main.rs` and asserts `--version` output after the build. See
[Verification](#verification).

`templates/report.html` is copied in the builder only: Askama embeds the
template into the binary at compile time, so the runtime stage needs no template
directory.

`CMD ["scan", "."]` resolves to `/workspace`, the image's working directory, so
`docker run --rm -v "$PWD:/workspace" IMAGE` scans the mount and writes
`rustydeps-report.html` next to the user's manifests.

### Workflow

Triggered by pushes to `main`, pushes of `v*` tags, and pull requests targeting
`main` or `development` (`development` is this repository's integration branch,
so a feature PR's build test would otherwise never run). Steps: checkout,
compute a lowercase image reference, QEMU + Buildx setup, GHCR login
(`GITHUB_TOKEN`, skipped on pull requests), `docker/metadata-action` tag
extraction, and `docker/build-push-action`.

| Context | Platforms | Push | Tags |
|---------|-----------|------|------|
| PR to `main`/`development` | `linux/amd64` | no (`load: true`) | `pr-<n>` |
| Push to `main` | `linux/amd64`, `linux/arm64` | yes | `main`, `latest` |
| Push of `v*` | `linux/amd64`, `linux/arm64` | yes | `1.2.3`, `1.2`, `latest` |

Pull requests load the single-platform image into the runner's daemon and run
`--version` plus an offline scan of the checkout, so a missing shared library or
a broken entrypoint fails review rather than surfacing after merge.

`development` is in the pull-request filter because feature branches integrate
there rather than into `main`; without it a feature PR would build no image at
all.

## Design decisions

- **Alpine rather than distroless.** Distroless has no shell, which the issue
  explicitly rules out for GitLab-style job images. Alpine contributes `sh`
  (BusyBox) and is still small.
- **`alpine:3` rather than the briefed `alpine:3.19`.** 3.19 is past Alpine's
  release window by now, and the rolling `alpine:3`/`rust:1-alpine` pair tracks
  the same Alpine release, keeping the build-stage libc and run-stage libc in
  step instead of pinning the runtime to an older musl than the builder.
- **OpenSSL is linked statically, and the runtime stage carries none of it.**
  On Linux `reqwest` uses `native-tls`, so the dependency graph contains
  `openssl-sys` — verified with `cargo tree --target x86_64-unknown-linux-gnu -i
  openssl-sys`. The `x86_64-unknown-linux-musl` target links statically by
  default (`crt-static`), so the linker wants `libssl.a`/`libcrypto.a`, which is
  what `openssl-libs-static` provides; Alpine builds OpenSSL with `no-zlib`
  (aports `main/openssl/APKBUILD`), so no `-lz` flag is needed. Because the
  resulting binary carries libc, libgcc and OpenSSL internally, the runtime
  stage installs only `ca-certificates`, which HTTPS lookups read at run time.
  Migrating to `rustls` would remove OpenSSL from the build graph but is a
  product-wide TLS change, out of scope here.
- **The dependency-cache stub needs `touch`.** Cargo decides freshness by
  mtime, and Docker `COPY` preserves the source mtimes from the build context.
  The stub written inside the builder layer is therefore *newer* than the real
  sources, so the second `cargo build` was a no-op (0.14 s, no recompile) and
  the stub — a `rustydeps` that accepts any argument, prints nothing and exits
  0 — was copied into the runtime stage. `touch src/main.rs` forces the
  recompile, and `./target/release/rustydeps --version | grep -q '^rustydeps '`
  in the same `RUN` makes any recurrence a build failure rather than an empty
  image.
- **The PR smoke test asserts on container output.** A green step is not
  evidence by itself: the first version of the step completed in under a second
  with no captured output. It now captures `--version`, requires it to contain
  `rustydeps`, has the scan write its report through the bind mount, and asserts
  the file is non-empty and HTML. That assertion is what exposed the stub bug
  above.
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
- `cargo build --release --locked` — the builder stage's command — succeeds, and
  `--version` plus an offline scan pass against the local release binary.
- `.github/workflows/docker.yml` parses as YAML; its triggers, permissions, step
  list and metadata tags were inspected, and every `run:` block passes `bash -n`;
  the lowercase step was executed locally and yields
  `name=ghcr.io/alexdevstheweb/rustydeps`.
- **The image itself is verified by CI** (Docker workflow run 38801998432 on
  PR #16, 3m44s): the build compiles the crate and passes the in-Dockerfile
  `--version` assertion, and the smoke test reports
  `container reports: rustydeps 0.1.0` and
  `report written through the mount: 65482 bytes`, having asserted the file is
  non-empty HTML.
- The `cargo build`-skips-the-rebuild mechanism (the stub bug below) was
  reproduced locally in a scratch crate: a stub built first plus an
  older-mtime source made cargo print `Finished ... in 0.01s` and keep the stub,
  while `touch` forced `Compiling` and a printing binary.
- Two failing runs preceded the green one and are part of the record:
  - 38800896598 — smoke test failed with an empty `--version`, revealing that
    the shipped binary was the stub.
  - 38801611324 — surfaced the static-link failure
    (`ld: cannot find -lssl`) that the stub had been hiding, because an unused
    crate is not linked.
- **Not verified:** the multi-arch (`linux/arm64`) build and an actual push to
  GHCR, both of which only run on `main`/tag events; the PR job builds
  `linux/amd64` and loads it locally.

## Related

- [[release-automation]] — the sibling tag-triggered release pipeline for native
  binaries and the README it introduced.
- [[architecture]] — the CLI and modules the image packages.
- [[roadmap]] — the delivery phases; like release tooling this is not a phase.
- [implementation report](implementation-report.md) — file-by-file detail.
- [[index]] — knowledge-graph entrypoint.
