```
 █▀█ █  █ █▀▀ ▀█▀ █  █ █▀▄ █▀▀ █▀█ █▀    v0.2.0
 █▀▄ █▄▄█ ▄██  █  ▀▄▄█ █▄▀ ██▄ █▀▀ ▄█
```

🦀 **Smart cross-registry dependency intelligence for Rust, Node, Python, Go & PHP.**

[![Release](https://github.com/AlexDevsTheWeb/rustydeps/actions/workflows/release.yml/badge.svg)](https://github.com/AlexDevsTheWeb/rustydeps/actions/workflows/release.yml)
[![Docker](https://github.com/AlexDevsTheWeb/rustydeps/actions/workflows/docker.yml/badge.svg)](https://github.com/AlexDevsTheWeb/rustydeps/actions/workflows/docker.yml)
![Version](https://img.shields.io/badge/version-0.2.0-ea580c)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-ea580c)

**[Documentation & demo → alexdevstheweb.github.io/rustydeps](https://alexdevstheweb.github.io/rustydeps/)**

**RustyDeps** is a lightning-fast, cross-platform CLI utility built in Rust that helps
fullstack developers monitor and manage dependencies across multiple local repositories
simultaneously.

It recursively scans project directories while respecting `.gitignore`, queries official
package registries concurrently, classifies version jumps into **Safe**, **At-Risk**, and
**Breaking**, and outputs a modern, self-contained interactive HTML dashboard — with
optional AI-driven release notes analysis.

## Supported ecosystems

| Ecosystem | Manifest(s) | Registry |
|-----------|-------------|----------|
| Cargo | `Cargo.toml` | [crates.io](https://crates.io) |
| npm | `package.json` | [registry.npmjs.org](https://registry.npmjs.org) |
| PyPI | `pyproject.toml`, `requirements.txt` | [pypi.org](https://pypi.org) |
| Go | `go.mod` | [proxy.golang.org](https://proxy.golang.org) |
| Composer | `composer.json` | [packagist.org](https://packagist.org) |

Each update is classified as one of:

| Risk | Meaning |
|------|---------|
| `breaking` (major) | A major version jump, or a change the declared requirement forbids. |
| `at-risk` (minor) | A bump outside the declared range that is not obviously breaking. |
| `safe` (patch) | A patch or minor bump inside the declared requirement. |
| `up-to-date` | The declared version already matches the latest release. |
| `unknown` | Unparseable versions, a downgrade, or missing registry data. |

## Quickstart

```bash
# Install from source via cargo
cargo install --git https://github.com/AlexDevsTheWeb/rustydeps

# Or clone and build a release binary
git clone https://github.com/AlexDevsTheWeb/rustydeps
cd rustydeps
cargo build --release
```

```bash
# Scan the current directory and open the report in your browser
rustydeps scan .

# Scan a specific repository, write the report, and don't auto-open it
rustydeps scan ~/code/my-app -o report.html --no-open

# Machine-readable output (no banner; pipe it into jq)
rustydeps scan . --format json

# Classify from local manifests only, without any network access
rustydeps scan . --offline
```

## Docker

An official multi-arch image (Linux `amd64` and `arm64`) is published to GitHub
Container Registry. It bundles the prebuilt binary, so a CI job needs neither a
Rust toolchain nor a compile step:

```bash
# Pull the latest release and scan the current directory
docker run --rm -v "$PWD:/workspace" ghcr.io/alexdevstheweb/rustydeps:latest

# The report is written into the mounted directory
docker run --rm -v "$PWD:/workspace" ghcr.io/alexdevstheweb/rustydeps:0.2.0 \
  scan /workspace --no-open -o /workspace/rustydeps-report.html

# Pin a version and skip registry lookups entirely
docker run --rm -v "$PWD:/workspace" ghcr.io/alexdevstheweb/rustydeps:0.2.0 \
  scan /workspace --offline --no-open
```

`/workspace` is the image's working directory, so `docker run --rm -v
"$PWD:/workspace" ghcr.io/alexdevstheweb/rustydeps` on its own is equivalent to
`rustydeps scan .` with the report landing beside your manifests. Image tags
mirror the release tags (`0.2.0`, `0.1`) plus `latest` for the default branch.

### CI usage

```yaml
# GitLab CI — the Alpine-based image ships `sh`, which job images require
rustydeps:
  image: ghcr.io/alexdevstheweb/rustydeps:0.2.0
  script:
    - rustydeps scan . --no-open -o rustydeps-report.html
  artifacts:
    paths: [rustydeps-report.html]
```

```yaml
# GitHub Actions
- uses: docker://ghcr.io/alexdevstheweb/rustydeps:0.2.0
  with:
    args: scan . --no-open
```

Notes:

- The image contains **CA certificates**, so registry lookups over HTTPS work out
  of the box; `--offline` skips them for a hermetic run.
- It runs as **root**, deliberately: CI mounts a workspace owned by an arbitrary
  host user, and a fixed non-root uid could not write the report there.
- Auto-open is pointless inside a container, so pass **`--no-open`** (or let the
  failed browser launch be logged as a warning).
- The image is built and published by the
  [Docker workflow](.github/workflows/docker.yml) on `main` pushes and `v*` tags;
  pull requests build both architectures and smoke-test the image without
  pushing.

## Features

- **Auto-open report** — after `scan` writes the HTML report, it opens in your default
  browser. Suppress it with `--no-open` (handy for CI or headless runs).
- **Offline mode** — `--offline` (alias `--no-fetch`) skips all registry lookups and
  classifies from the local manifests only; nothing leaves your machine.
- **JSON output** — `--format json` emits the classified result set for tooling and
  suppresses the banner so stdout stays clean.
- **Self-contained HTML report** — one file with inline CSS/JS: project-centric cards,
  risk dots, an inline changelog drawer with copyable upgrade commands, search and risk
  filters, and an "Update All" batch command. Opens offline, no CDN.
- **Granular per-library AI analysis** — with BYOK configuration, every outdated
  package gets its own Markdown briefing (breaking changes, security context,
  performance impact, migration advice) inside the report drawer. Analyses are
  cached per ecosystem + package + target version, so repeat scans cost nothing.
  Enable it with `ai.enabled` in the config or `--ai`; `--no-ai` disables it.
- **Concurrent, polite registry access** — bounded concurrency (10 in flight) with graceful
  degradation: a failed lookup is recorded as `unknown` and never aborts the scan.
- **`--verbose` / `-v`** — debug-level logging to stderr.

## Usage

```
rustydeps scan [PATH] [OPTIONS]

Arguments:
  [PATH]  Directory to scan (default: .)

Options:
      --ai              Enable per-library AI changelog summaries
      --no-ai           Disable AI summaries even when the config enables them
      --ai-model <MODEL> Override the configured AI model
  -o, --output <PATH>   Report output path (default: rustydeps-report.html)
      --format <FORMAT> html | json (default: html)
      --offline         Skip registry lookups (alias: --no-fetch)
      --no-open         Do not open the generated report in the browser
  -v, --verbose         Verbose (debug) logging
  -h, --help            Print help
  -V, --version         Print version
```

## AI changelog summaries (BYOK)

RustyDeps never ships a shared API key. Configure your own provider in
`~/.config/rustydeps/config.json`. If that file does not exist, a project-local
`.rustydeps.json` is used instead (it is git-ignored in case it holds a key):

```json
{
  "ai": {
    "enabled": true,
    "provider": "openai",
    "api_key": "sk-...",
    "model": "gpt-4o-mini",
    "endpoint": "https://api.openai.com/v1",
    "max_tokens": 400,
    "cache_enabled": true
  }
}
```

`provider` accepts `openai`, `gemini`, and `ollama`. Gemini uses Google's
OpenAI-compatible endpoint; Ollama defaults to `http://localhost:11434/v1` and
needs no API key. Every request is bounded at 10 seconds and only outdated,
registry-resolved packages are analyzed.

Environment variables override the file; CLI flags override both:

| Variable | Purpose |
|----------|---------|
| `RUSTYDEPS_AI_ENABLED` | `true`/`false` |
| `RUSTYDEPS_AI_PROVIDER` | `openai`, `gemini`, or `ollama` |
| `RUSTYDEPS_AI_API_KEY` | API key (falls back to `OPENAI_API_KEY` / `GEMINI_API_KEY`) |
| `RUSTYDEPS_AI_MODEL` | Model name |
| `RUSTYDEPS_AI_ENDPOINT` | OpenAI-compatible base URL |

Generated analyses are cached at
`~/.cache/rustydeps/changelogs/{ecosystem}_{package}_{target_version}.json`.
Remove that directory to force fresh analyses. If the provider is disabled,
misconfigured, offline, rate-limited, or slow, the scan still succeeds: the
drawer links to the package's published versions and omits the AI card.

## Releasing

Pushing a tag matching `v*` (for example `v0.3.0`) triggers the
[release workflow](.github/workflows/release.yml), which builds binaries for Linux
(`x86_64-unknown-linux-gnu`) and macOS (`x86_64-apple-darwin`, `aarch64-apple-darwin`),
then attaches them (with checksums and a changelog summary) to the GitHub Release.
The same tag also triggers the [Docker workflow](.github/workflows/docker.yml),
which publishes the multi-arch image to `ghcr.io/alexdevstheweb/rustydeps`
tagged with the release version (see [Docker](#docker)).

```bash
git tag v0.3.0
git push origin v0.3.0
```

## Development

```bash
cargo build
cargo test
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
