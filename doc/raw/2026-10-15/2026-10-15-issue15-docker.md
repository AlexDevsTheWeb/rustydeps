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

## Changes made

- **`Dockerfile`** — two stages. `rust:1-alpine` installs `musl-dev`,
  `openssl-dev` and `pkgconf`, compiles the dependency graph against a stub
  `src/main.rs` (so editing `src/` does not rebuild third-party crates), then
  copies `src/` and `templates/` and rebuilds `--release --locked`.
  `alpine:3` runtime installs `ca-certificates`, `libssl3` and `libgcc`, copies
  the binary to `/usr/local/bin/rustydeps`, sets `WORKDIR /workspace` and
  exposes `ENTRYPOINT ["rustydeps"]` + `CMD ["scan", "."]`, with OCI labels.
- **`.dockerignore`** — context trimmed to `Cargo.toml`, `Cargo.lock`, `src/`
  and `templates/`.
- **`.github/workflows/docker.yml`** — build on `main` pushes, `v*` tags and
  PRs to `main`; QEMU + Buildx; GHCR login with `GITHUB_TOKEN`; metadata tags
  `latest`/branch/semver/`pr-<n>`; pushes `linux/amd64` + `linux/arm64`, while
  PRs load a single-platform image and smoke-test it.
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

## Tests executed

| Check | Result |
|-------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 85 passed, 0 failed |
| `cargo build --release --locked` (Dockerfile builder command) | succeeds |
| `./target/release/rustydeps --version` | `rustydeps 0.1.0` |
| `./target/release/rustydeps scan . --offline --no-open -o /tmp/…html` | exit 0, 65 KB report written |
| `cargo tree --target x86_64-unknown-linux-gnu -i openssl-sys` | `reqwest → native-tls → openssl-sys`, confirming the runtime `libssl3` need |
| `docker.yml` YAML parse + inspection | OK — triggers, `permissions`, 8 steps, metadata tags |
| `bash -n` on both `docker.yml` `run:` blocks | OK |
| Lowercase image-reference step executed locally | `name=ghcr.io/alexdevstheweb/rustydeps` |
| OKF validator (`validate_okf.py`, staging shim) | **PASSED** — 0 errors, 0 warnings |

**Not executed — `docker build` / `docker run`.** No container runtime (Docker,
Podman, Colima, Lima) is installed on the development machine, so the brief's
gates 2 and 3 could not be run locally; the new PR job is the first place the
image is built and run. The image build is therefore **unverified** until that
job runs.

## Technical debt

- The image build and the GitHub-side actions have never executed, so the
  Dockerfile is unproven in practice (first proven by the PR job).
- `reqwest` uses `native-tls`, which forces OpenSSL into both stages; switching
  to `rustls` would allow a `scratch` runtime and drop ~10 MB.
- The runtime stage runs as root; a non-root user would need a documented uid
  strategy for CI volume mounts.
- No image scanning (Trivy/Grype), no SBOM/provenance attestations, and the
  base images are unpinned rolling tags with no Dependabot configuration.
- The workflow publishes to GHCR only; no Docker Hub mirror.
- Multi-arch builds run under QEMU on `ubuntu-latest`; native arm64 runners
  would cut the wall-clock time substantially.

## Next recommended actions

1. Merge once the PR job's build + smoke test is green, then watch the first
   `main` run publish `latest`.
2. Tag `v0.1.0` afterwards so the image gains SemVer tags (`0.1.0`, `0.1`).
3. Add a Dependabot config covering the `docker` ecosystem, plus Trivy image
   scanning in the workflow.
4. Consider the `rustls` migration (drops OpenSSL, enables a distroless/scratch
   runtime) as its own change, and revisit a non-root image user.
5. Phase 6 (AI Changelog Summarizer) remains the next roadmap phase.
