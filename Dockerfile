# syntax=docker/dockerfile:1

# RustyDeps container image.
#
# Stage 1 compiles the release binary against Alpine's musl libc; stage 2 keeps
# only the binary, the CA trust store and the shared libraries it links against.
# Both stages track the newest Alpine release (`rust:1-alpine` / `alpine:3`), so
# the build-stage libc and the run-stage libc never drift apart.

# ---------------------------------------------------------------------------
# Stage 1 — build
# ---------------------------------------------------------------------------
FROM rust:1-alpine AS builder

# `rust:1-alpine` already ships `gcc`, `musl-dev` and `ca-certificates` (see
# docker-rust's `Dockerfile-alpine.template`), so the only addition is
# `openssl-dev`, which provides the TLS headers: on Linux `reqwest` goes through
# the system TLS stack (native-tls), so `openssl-sys` needs the headers plus
# `pkgconf` to find and link them.
RUN apk add --no-cache openssl-dev pkgconf

WORKDIR /build

# Compile the dependency graph on its own layer so editing `src/` does not
# rebuild every third-party crate. `--locked` keeps the build reproducible
# against the committed `Cargo.lock`.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --locked \
 && rm -rf src

# Real sources. `templates/` is a build-time input only: Askama embeds the
# report template in the binary, so nothing is read from disk at run time.
COPY src ./src
COPY templates ./templates
RUN cargo build --release --locked

# ---------------------------------------------------------------------------
# Stage 2 — runtime
# ---------------------------------------------------------------------------
FROM alpine:3 AS runtime

# ca-certificates — HTTPS calls to crates.io, npm, PyPI, proxy.golang.org and
# packagist.org.
# libssl3/libcrypto3 — the shared OpenSSL the binary links against.
# libgcc — unwinding support for the Rust runtime.
# BusyBox `sh` comes with the base image, which CI systems (GitLab CI and
# friends) need in a job image.
RUN apk add --no-cache ca-certificates libssl3 libgcc

COPY --from=builder /build/target/release/rustydeps /usr/local/bin/rustydeps

LABEL org.opencontainers.image.title="rustydeps" \
      org.opencontainers.image.description="Fast, cross-platform CLI that scans local repositories for dependency manifests and reports available upgrades." \
      org.opencontainers.image.source="https://github.com/AlexDevsTheWeb/rustydeps" \
      org.opencontainers.image.url="https://alexdevstheweb.github.io/rustydeps/" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

# The scan target defaults to the mounted working directory:
#   docker run --rm -v "$PWD:/workspace" ghcr.io/alexdevstheweb/rustydeps
WORKDIR /workspace

ENTRYPOINT ["rustydeps"]
CMD ["scan", "."]
