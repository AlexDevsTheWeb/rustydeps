# Development Report — 2026-10-14T144514Z

> RustyDeps Phase 1 — Rust crate initialization, CLI setup, and filesystem
> scanner. Source: `doc/raw/2026-10-14-phase-1/2026-10-14-phase-1.md`. Raw
> documents are immutable; a follow-up supersedes this with a new dated file.

## Summary

Initialized the Rust binary crate, built the `clap` (derive) CLI with a `scan`
subcommand, and implemented the `ignore`-based filesystem scanner that discovers
supported dependency manifests. Addressed the two scaffolding technical-debt
items (dual licensing, legacy tooling removal), updated the LLM-Wiki, and
validated everything.

## Modifications

### Step 0 — Technical debt

- Added `LICENSE-MIT` and `LICENSE-APACHE` (canonical Apache-2.0 text fetched
  from `apache.org`); `Cargo.toml` declares `license = "MIT OR Apache-2.0"`.
- Removed the unrelated `.claude/skills/stitch-to-mui/` directory.

### Step 1 — Crate initialization

- `cargo init --bin --name rustydeps --vcs none`; `Cargo.toml` set to **edition
  2021** with: `clap` (derive, env), `ignore`, `tokio` (full), `anyhow`,
  `thiserror`, `serde` (derive), `serde_json`, `tracing`,
  `tracing-subscriber` (env-filter). Added `tempfile` under `[dev-dependencies]`
  for scanner tests (not in the task's dependency list, required by Step 3's
  test instruction).
- `Cargo.lock` committed (binary crate convention).

### Step 2 — CLI (`src/cli.rs`, `src/main.rs`)

- `Cli` (`clap::Parser`) with the `scan` subcommand: `path` (default `.`),
  `--ai`, `--output`/`-o` (default `report.html`), `--format html|json`,
  `--verbose`/`-v`.
- `main` is `#[tokio::main] async fn main() -> anyhow::Result<()>`; initializes
  `tracing-subscriber` (stderr; `RUST_LOG` wins, else `debug` under `--verbose`
  and `info` otherwise) and dispatches the subcommand.
- `scan` discovers manifests, logs a count, and prints paths as text or JSON
  (`serde_json`). `--output`/`--ai` are accepted and logged as reserved for
  later phases.

### Step 3 — Scanner (`src/scanner.rs`)

- `ManifestType` enum (`CargoToml`, `PackageJson`, `GoMod`, `RequirementsTxt`,
  `PyprojectToml`, `ComposerJson`) with `from_file_name`/`file_name`.
- `DiscoveredManifest { path, manifest_type }` (serde-serializable).
- `scan_directory` uses `ignore::WalkBuilder` with `hidden(true)`, git/ignore
  rules on, `require_git(false)` (so `.gitignore` applies even outside a git
  repo), and prunes `node_modules`, `target`, and `.git`; returns a path-sorted
  `Vec<DiscoveredManifest>`; errors on a missing root.
- **6 unit tests** (`tempfile` fixtures): manifest round-trip, discovery,
  `.gitignore` respected, well-known dirs skipped, hidden dirs skipped, missing
  root errors.

### Step 4 — Wiki

- `doc/wiki/architecture.md` — CLI/scanner sections rewritten to describe the
  implemented modules; added the `ManifestType`/`DiscoveredManifest` DTOs.
- `doc/wiki/roadmap.md` — Phase 1 marked Done; Phase 2 marked Next.
- `doc/wiki/log.md` — appended the Phase 1 entry.

## Tests executed

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | Clean — no diff |
| `cargo clippy --all-targets -- -D warnings` | Clean — no warnings |
| `cargo test` | **6 passed**, 0 failed |
| `cargo run -- scan .` (smoke) | Listed `./Cargo.toml`; `target/` correctly skipped |
| `cargo run -- scan . --format json` (smoke) | Valid JSON on stdout; logs on stderr |
| `cargo run -- scan /nope` (smoke) | `Error: scan root does not exist: /nope`, exit 1 |
| `python3 validate_okf.py` (`.claude`, staging shim) | **PASSED** — 0 errors, 0 warnings |
| `python3 check_techdocs_links.py` (same shim) | PASSED — 5 pages |

Toolchain: Rust 1.99.0 stable (installed via rustup during this task; it was
absent). `rustfmt` 1.10.0 and `clippy` 0.1.99 present.

## Technical debt

- **`--output`, `--format html`, `--ai` are inert.** The scanner runs and prints
  results, but there is no report engine (Phase 4) or AI module (Phase 5); the
  CLI logs this explicitly rather than silently ignoring the flags.
- **`thiserror` is a declared dependency but not yet used** — it is reserved for
  the typed parser/registry errors in Phase 2/3. Keeping it honours the task's
  `Cargo.toml` spec; remove it if Phase 2 does not adopt it.
- **No CI workflow** yet for `fmt`/`clippy`/`test`; gates are currently manual.
- **`.claude/` remains tracked tooling.** `settings.local.json` is excluded via
  the global gitignore; `stitch-to-mui` has now been removed.
- **`require_git(false)`** is a deliberate divergence from ripgrep-like
  semantics: `.gitignore` is honoured even outside a git repo. Cheap to revisit
  if git-only semantics are preferred.

## Next recommended actions

1. **Phase 2:** implement the `PackageParser` trait with `CargoParser` and
   `NpmParser`, and the `reqwest`/`tokio` registry clients.
2. Add a CI workflow running `cargo fmt --check`, `cargo clippy --all-targets --
   -D warnings`, and `cargo test`.
3. Add a `rust-toolchain.toml` to pin the toolchain for reproducibility.
