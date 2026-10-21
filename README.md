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
      --ai-provider <PROVIDER>
                        openai | gemini | openrouter | ollama | custom
      --ai-endpoint <URL> Override the configured AI base URL
      --no-cache        Ignore cached AI analyses and regenerate them
  -o, --output <PATH>   Report output path (default: rustydeps-report.html)
      --format <FORMAT> html | json (default: html)
      --offline         Skip registry lookups (alias: --no-fetch)
      --no-open         Do not open the generated report in the browser
  -v, --verbose         Verbose (debug) logging
  -h, --help            Print help
  -V, --version         Print version

rustydeps cache clear   Delete every cached AI changelog entry
```

## AI changelog summaries (BYOK)

RustyDeps is **local-first and BYOK** (Bring Your Own Key). It ships no shared
key, runs no RustyDeps proxy, and collects no telemetry: your machine talks
directly to the provider you configure, and every generated analysis is cached
on disk.

- **Config file:** `~/.config/rustydeps/config.json`
  (`%APPDATA%\rustydeps\config.json` on Windows). A project-local
  `.rustydeps.json` is used only when that file does not exist; it is git-ignored
  because it may contain a key.
- **Cache:** `~/.cache/rustydeps/changelogs/{ecosystem}_{package}_{target_version}.json`
  (`%LOCALAPPDATA%\rustydeps\changelogs\` on Windows). A repeat scan of the same
  package version makes no request and costs no tokens. Run
  `rustydeps cache clear` to delete every entry (it prints
  `✔ Cleared X cached AI changelog entries.`), or pass `--no-cache` to ignore
  cached reads for one scan and regenerate every analysis; fresh results still
  refresh the cache. The report badge names the model that produced a cached
  entry, for example `⚡ Cached (llama3.2:3b)`.
- **Request scope:** only outdated, registry-resolved packages are analyzed, at
  most four concurrently, with a 45-second timeout per request. The generated
  Markdown appears in the package drawer; every failure degrades gracefully to
  the standard changelog link.
- **Ollama preflight:** before analyzing, an Ollama run probes
  `http://localhost:11434/api/tags` with a 1-second timeout. If the daemon is
  offline, one warning explains how to start it (`ollama serve` or
  `brew services start ollama`) and no package-level requests are attempted;
  cached analyses are still applied.

### Supported providers

| Provider | `provider` value | Default endpoint | Default model | API key |
|----------|------------------|------------------|---------------|---------|
| OpenAI | `openai` | `https://api.openai.com/v1` | `gpt-4o-mini` | Required |
| Google Gemini (AI Studio) | `gemini` / `google` | `https://generativelanguage.googleapis.com/v1beta/openai` | `gemini-3.5-flash` | Required |
| OpenRouter | `openrouter` | `https://openrouter.ai/api/v1` | `google/gemini-2.5-flash` | Required |
| Ollama | `ollama` | `http://localhost:11434/v1` | `llama3.1` | Not required |
| Custom OpenAI-compatible proxy | `custom` / `openai-compatible` | Your URL | Your model | Optional |

Any provider can override `model` and `endpoint`; the custom provider requires
both. The endpoint validator accepts a bare origin (`https://api.openai.com`), a
provider base (`.../v1`, `.../v1beta/openai`), or a full
`.../chat/completions` URL, with or without a trailing slash. Existing query
strings are preserved.

> **Free and quota-free:** Ollama runs the model on your machine, so it needs no
> account, no key, no network, and has no quota. Switch to it for one scan with
> `--ai --ai-provider ollama` (the provider switch resets the model and endpoint
> to Ollama defaults), or persist it in the config as shown below. A failing
> hosted provider is stopped after the first authentication/quota/model error, so
> a scan never floods the console with one warning per package.

### Copy-paste configuration

Save one of these as `~/.config/rustydeps/config.json` (or as
`.rustydeps.json` in a project root).

**Google Gemini (AI Studio)** — create a key at
<https://aistudio.google.com/apikey>:

```json
{
  "ai": {
    "enabled": true,
    "provider": "gemini",
    "api_key": "AIza...",
    "model": "gemini-3.5-flash"
  }
}
```

> Gemini retires model IDs over time. A `404 ... model ... is not found` warning
> means the configured `model` is no longer served by your account; set
> `ai.model` (or `--ai-model`) to a current ID. Gemini 3 thinking models are
> called with `reasoning_effort: "none"` so the 400-token summary budget is not
> consumed by hidden reasoning; lite variants are called without that field.

**OpenAI** — create a key at <https://platform.openai.com/api-keys>:

```json
{
  "ai": {
    "enabled": true,
    "provider": "openai",
    "api_key": "sk-...",
    "model": "gpt-4o-mini"
  }
}
```

**OpenRouter** — create a key at <https://openrouter.ai/keys>; any model from
its catalogue can be named, and `openrouter/auto` lets OpenRouter choose:

```json
{
  "ai": {
    "enabled": true,
    "provider": "openrouter",
    "api_key": "sk-or-v1-...",
    "model": "google/gemini-2.5-flash"
  }
}
```

**Ollama (100% free, local and offline)** — install Ollama, start the server
(`brew services start ollama`, or `ollama serve`), and pull a model
(`ollama pull llama3.2:3b`) first:

```json
{
  "ai": {
    "enabled": true,
    "provider": "ollama",
    "model": "llama3.2:3b",
    "endpoint": "http://localhost:11434/v1"
  }
}
```

**Custom OpenAI-compatible proxy** — point `endpoint` at your gateway and name
its model; `api_key` is optional for keyless local gateways:

```json
{
  "ai": {
    "enabled": true,
    "provider": "custom",
    "api_key": "optional",
    "model": "your-model",
    "endpoint": "https://llm.example/v1"
  }
}
```

The remaining optional keys are `max_tokens` (default `400`, clamped to
`1..=8192`) and `cache_enabled` (default `true`).

### Environment variables and CLI overrides

Environment variables override the file; CLI flags override both:

| Variable | Purpose |
|----------|---------|
| `RUSTYDEPS_AI_ENABLED` | `true`/`false`, `1`/`0`, `yes`/`no`, `on`/`off` |
| `RUSTYDEPS_AI_PROVIDER` | `openai`, `gemini`, `openrouter`, `ollama`, or `custom` |
| `RUSTYDEPS_AI_API_KEY` | Generic key; when unset, the provider-native variable is used |
| `RUSTYDEPS_AI_MODEL` | Model name |
| `RUSTYDEPS_AI_ENDPOINT` | Base URL or full chat-completions URL |

Native key fallbacks: `OPENAI_API_KEY`, `GEMINI_API_KEY` (or
`GOOGLE_API_KEY`), and `OPENROUTER_API_KEY`.

| Flag | Effect |
|------|--------|
| `--ai` | Force AI summaries on for this scan |
| `--no-ai` | Force AI summaries off; wins over `--ai` and the config |
| `--ai-model <MODEL>` | Override the configured model for this scan |
| `--ai-provider <PROVIDER>` | Override the provider; also resets model/endpoint to its defaults unless given explicitly |
| `--ai-endpoint <URL>` | Override the configured base URL or full chat-completions URL |
| `--no-cache` | Ignore cached analyses for this scan and regenerate them |

Only dependencies with an available upgrade are analyzed. Up-to-date packages
never consume a request, and their report drawers show a plain status card
instead of an "AI analysis unavailable" notice.

If a provider call fails, the warning names the resolved request URL, the HTTP
status, and a compact snippet of the response body, for example
`request to https://api.openai.com/v1/chat/completions returned HTTP 401:
{"error": ...}`. That is usually enough to spot a wrong endpoint, key, or model
name without enabling debug logging.

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
