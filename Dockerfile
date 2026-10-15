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

# The musl target statically links by default (`crt-static`), so the linker
# needs OpenSSL as an *archive*: `openssl-dev` supplies the headers and
# `openssl-libs-static` the `libssl.a`/`libcrypto.a` the final link pulls in.
# Alpine builds OpenSSL with `no-zlib` (see aports `main/openssl/APKBUILD`), so
# no `-lz` is required. `pkgconf` lets `openssl-sys` find all of it.
# (`gcc`, `musl-dev` and `ca-certificates` already come with `rust:1-alpine`.)
RUN apk add --no-cache openssl-dev openssl-libs-static pkgconf

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
#
# `touch` is load-bearing, not decoration: the stub `src/main.rs` from the layer
# above is newer than the sources `COPY` preserves from the build context, so
# cargo's mtime fingerprints would consider the crate up to date, skip the
# rebuild and leave the stub binary — a `rustydeps` that prints nothing and
# exits 0. The `--version` assertion turns any recurrence of that into a build
# failure instead of a silently empty image.
COPY src ./src
COPY templates ./templates
RUN touch src/main.rs \
 && cargo build --release --locked \
 && ./target/release/rustydeps --version | grep -q '^rustydeps '

# ---------------------------------------------------------------------------
# Stage 2 — runtime
# ---------------------------------------------------------------------------
FROM alpine:3 AS runtime

# ca-certificates — HTTPS calls to crates.io, npm, PyPI, proxy.golang.org and
# packagist.org, read at run time from the trust store.
# Nothing else is needed: the `crt-static` musl build links libc, libgcc and
# OpenSSL into the binary, so there are no shared-library dependencies to
# install. If the build ever switches to dynamic linking, `libssl3` has to come
# back — the workflow smoke test runs this image, so it would fail loudly.
# BusyBox `sh` comes with the base image, which CI systems (GitLab CI and
# friends) need in a job image.
RUN apk add --no-cache ca-certificates

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
