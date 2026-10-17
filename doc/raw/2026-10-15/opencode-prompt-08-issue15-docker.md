# Task Prompt — Issue #15: Docker Image & CI Container Publishing

## Context & Objective
Issue #15 requests an official Docker image for `rustydeps` to enable seamless integration into CI/CD pipelines (GitHub Actions, GitLab CI, Bitbucket Pipelines, Jenkins, etc.) without requiring a local Rust toolchain installation.

You will implement:
1. A multi-stage `Dockerfile` optimized for minimal final image size.
2. A `.dockerignore` file.
3. A GitHub Actions workflow (`.github/workflows/docker.yml`) to build and push multi-arch Docker images to GitHub Container Registry (`ghcr.io`).
4. Documentation in `README.md` explaining Docker usage.

---

## Technical Specifications

### 1. `Dockerfile` (Multi-Stage Build)
- **Stage 1 (Builder):**
  - Base image: `rust:1-alpine` or `rust:1-slim` (Alpine preferred for static linking with `x86_64-unknown-linux-musl` or lightweight libc).
  - Install required build dependencies if needed (`musl-dev`, `pkgconfig`).
  - Copy `Cargo.toml`, `Cargo.lock`, `src/`, `templates/`.
  - Build release binary: `cargo build --release`.
- **Stage 2 (Runtime):**
  - Base image: `alpine:3.19` (or `gcr.io/distroless/cc-debian12`).
  - Install `ca-certificates` (required for HTTPS requests to crates.io, npm, PyPI, etc.).
  - Copy binary from builder to `/usr/local/bin/rustydeps`.
  - Set `ENTRYPOINT ["rustydeps"]`.
  - Set `CMD ["scan", "."]`.

### 2. `.dockerignore`
- Ignore `target/`, `.git/`, `.claude/`, `doc/`, `tmp/`, `*.html`, `*.md`.

### 3. GitHub Actions Workflow (`.github/workflows/docker.yml`)
- Trigger on:
  - Push to `main` or `v*` tags.
  - Pull request to `main` (build test only, no push).
- Steps:
  - Checkout repository.
  - Set up QEMU and Docker Buildx for multi-architecture builds (`linux/amd64`, `linux/arm64`).
  - Log in to GitHub Container Registry (`ghcr.io`) using `GITHUB_TOKEN`.
  - Extract Docker metadata (tags: `latest`, `v0.1.0`, etc.).
  - Build and push image to `ghcr.io/${{ github.repository_owner }}/rustydeps`.

### 4. `README.md` Update
- Add a **Docker** section under **Installation / Usage**:
  ```bash
  # Run directly via Docker
  docker run --rm -v $(pwd):/workspace ghcr.io/alexdevstheweb/rustydeps:latest scan /workspace
  ```

---

## Quality Gates & Verification
1. `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` must pass cleanly.
2. Test building the Docker image locally: `docker build -t rustydeps:local .`.
3. Test running a container scan: `docker run --rm -v $(pwd):/workspace rustydeps:local scan /workspace --offline`.
4. Update Wiki and OKF docs (`python3 validate_okf.py`).
5. Open Draft PR #16 / feature branch `feat/docker-support`.
6. Write development report in `doc/raw/YYYY-MM-DD/YYYY-MM-DD-issue15-docker.md`.
