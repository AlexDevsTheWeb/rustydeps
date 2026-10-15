# Development Report — 2026-10-15T201008Z

> RustyDeps — Docker image and GHCR publishing for CI usage (issue #15).
> Source: `doc/raw/2026-10-15/opencode-prompt-08-issue15-docker.md`. Raw
> documents are immutable; a follow-up supersedes this with a new dated file.

## Summary

Packaging change on branch `feat/docker-support` implementing issue #15
("Feature request: publish an official Docker image for use in CI pipelines").
The repository gains a multi-stage Alpine `Dockerfile`, a `.dockerignore`, a
workflow that builds and publishes multi-arch images to GitHub Container
Registry, and a Docker section in the README. No Rust source changed.

Commits on the branch:

- `ci(docker): add multi-stage Alpine image for the CLI`
- `ci(docker): build and publish multi-arch images to GHCR`
- `docs(readme): document Docker usage and image publishing`
- `docs(raw): add the issue 15 docker development report` (this file)
- `docs(wiki): document the docker image and GHCR workflow`
- `docs(wiki): link the docker draft PR`
- `ci(docker): build-test the image on PRs into development`
- `docs: record the development pull-request trigger for the docker build`
- `ci(docker): assert on container output in the image smoke test`
- `fix(docker): stop shipping the stub binary as rustydeps`
- `fix(docker): link OpenSSL statically and slim the runtime stage`

## Changes made

- **`Dockerfile`** — two stages. `rust:1-alpine` installs `openssl-dev`,
  `openssl-libs-static` and `pkgconf`, compiles the dependency graph against a
  stub `src/main.rs` (so editing `src/` does not rebuild third-party crates),
  then `touch`es the entry point, copies `src/` and `templates/`, rebuilds
  `--release --locked` and asserts the binary's `--version` output.
  `alpine:3` runtime installs `ca-certificates`, copies the statically linked
  binary to `/usr/local/bin/rustydeps`, sets `WORKDIR /workspace` and exposes
  `ENTRYPOINT ["rustydeps"]` + `CMD ["scan", "."]`, with OCI labels.
- **`.dockerignore`** — context trimmed to `Cargo.toml`, `Cargo.lock`, `src/`
  and `templates/`, with the root-only pattern semantics documented.
- **`.github/workflows/docker.yml`** — build on `main` pushes, `v*` tags and
  PRs to `main` and `development`; QEMU + Buildx; GHCR login with `GITHUB_TOKEN`;
  metadata tags
  `latest`/branch/semver/`pr-<n>`; pushes `linux/amd64` + `linux/arm64`, while
  PRs load a single-platform image and run an output-asserting smoke test.
- **`README.md`** — Docker badge, `## Docker` section (run commands, GitLab CI
  and GitHub Actions snippets, image notes) and a *Releasing* note about the
  image.
- **Wiki** — new `features/docker-image/` (feature page + implementation
  report); `index.md` and `log.md` updated.

### Deviations from the brief

1. Runtime base is `alpine:3`, not the briefed `alpine:3.19`: 3.19 is past
   Alpine's release window, and the rolling tags keep the builder and runtime on
   the same Alpine release so the build libc and run libc cannot drift.
2. `gcr.io/distroless/cc-debian12` was rejected: distroless has no shell, and
   the issue text requires one for GitLab-style job images.
3. PR runs load and smoke-test the image (`--version` + an offline scan) rather
   than only building it, so a missing shared library fails review.
4. The report path requested by the brief (`doc/raw/2026-10-15/…`) is used
   instead of the repo's usual `doc/raw/development-report-<TIMESTAMP>.md`.
5. The pull-request trigger covers `main` **and** `development`: feature branches
   integrate into `development` here, so a `main`-only filter meant the build
   test never ran on the PR that adds it.

> Correction (same work step, before review): deviation 5 was added after the
> first push, when the PR workflow run was absent because this repository PRs
> into `development`. The workflow comment, this report, the wiki pages and the
> PR body were updated together.

### Corrections found by CI verification

The CI runs below are also corrections to this report's first version, which
recorded the image build as unverified:

- The dependency-cache stub was being shipped as `rustydeps` (cargo's mtime
  freshness check skipped the real build). Fixed with `touch src/main.rs` and the
  `--version` build assertion.
- The static musl link needed `openssl-libs-static` (`ld: cannot find -lssl`);
  fixing that made the runtime stage's `libssl3`/`libgcc` unnecessary, since the
  now-static binary carries libc, libgcc and OpenSSL internally.

## Tests executed

| Check | Result |
|-------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 85 passed, 0 failed |
| `cargo build --release --locked` (Dockerfile builder command) | succeeds |
| `./target/release/rustydeps --version` | `rustydeps 0.1.0` |
| `./target/release/rustydeps scan . --offline --no-open -o /tmp/…html` | exit 0, 65 KB report written |
| `cargo tree --target x86_64-unknown-linux-gnu -i openssl-sys` | `reqwest → native-tls → openssl-sys`, so the build needs OpenSSL |
| `docker.yml` YAML parse + inspection | OK — triggers, `permissions`, steps, metadata tags |
| `bash -n` on the `docker.yml` `run:` blocks | OK |
| Lowercase image-reference step executed locally | `name=ghcr.io/alexdevstheweb/rustydeps` |
| Docker workflow run 38801998432 (PR #16, 3m44s) | **green** — crate compiled, in-Dockerfile `--version` assertion passed, `container reports: rustydeps 0.1.0`, `report written through the mount: 65482 bytes` |
| Scratch-crate reproduction of the cargo mtime skip | bug and fix reproduced (see below) |
| OKF validator (`validate_okf.py`, staging shim) | **PASSED** — 0 errors, 0 warnings |

### CI verification (and the two defects it caught)

The brief's `docker build`/`docker run` gates could not run on the development
machine (no container runtime), so they were delegated to the new workflow. Two
failing runs on PR #16 were genuine defects, not flakes:

- **38800896598 — the stub binary was shipped as `rustydeps`.** The smoke test
  reported an empty `--version`; the build log showed the final
  `RUN cargo build --release --locked` finishing in 0.14 s with no recompile.
  Cargo's mtime freshness check sees the stub `src/main.rs` written inside the
  builder layer as *newer* than the sources `COPY` preserves from the context,
  so the crate was never rebuilt and `fn main() {}` went into the image. Fixed
  with `touch src/main.rs` and a build-time
  `./target/release/rustydeps --version | grep -q '^rustydeps '` assertion.
- **38801611324 — `ld: cannot find -lssl`.** Once the crate really compiled, the
  link failed: the `x86_64-unknown-linux-musl` target links statically by default
  (`crt-static`), so it needs `libssl.a` from `openssl-libs-static`. The stub had
  hidden it, since an unused crate is not linked. Alpine builds OpenSSL with
  `no-zlib` (aports `main/openssl/APKBUILD`), so no `-lz` flag is needed. The
  now-static binary let the runtime stage drop `libssl3`/`libgcc`, leaving only
  `ca-certificates`.

Both mechanisms were reproduced locally: a scratch crate built as a stub and then
given an older-mtime source kept the stub (`Finished … in 0.01s`), and `touch`
forced `Compiling` and a printing binary.

**Still unverified:** the `linux/arm64` build leg and a real push to GHCR — both
run only on `main`/tag events, not on the PR.

## Technical debt

- The image is verified on `linux/amd64` only: the arm64 leg and a real GHCR push
  run on `main`/tag events and have not executed yet.
- `reqwest` uses `native-tls`, which pulls OpenSSL into the build graph; with
  `rustls` the builder would need no OpenSSL at all (the runtime already needs
  none, since the binary is static).
- The runtime stage runs as root; a non-root user would need a documented uid
  strategy for CI volume mounts.
- No image scanning (Trivy/Grype), no SBOM/provenance attestations, and the
  base images are unpinned rolling tags with no Dependabot configuration.
- The workflow publishes to GHCR only; no Docker Hub mirror.
- Multi-arch builds run under QEMU on `ubuntu-latest`; native arm64 runners
  would cut the wall-clock time substantially.
- `docker/metadata-action` labels the published image
  `org.opencontainers.image.licenses=Apache-2.0` from the GitHub repository
  metadata, overriding the Dockerfile's dual-licence value. That needs a
  repository-settings fix, not a workflow change.

## Next recommended actions

1. Merge once the PR check is green (run 38801998432 passed), then watch the
   first `main` run publish `latest` and confirm the arm64 leg.
2. Tag `v0.1.0` afterwards so the image gains SemVer tags (`0.1.0`, `0.1`).
3. Add a Dependabot config covering the `docker` ecosystem, plus Trivy image
   scanning in the workflow.
4. Consider the `rustls` migration (drops OpenSSL from the build graph) as its
   own change, and revisit a non-root image user.
5. Phase 6 (AI Changelog Summarizer) remains the next roadmap phase.
