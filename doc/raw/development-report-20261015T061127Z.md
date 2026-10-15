# Development Report — 2026-10-15T061127Z

> RustyDeps — Phase 2 implemented: manifest parsers (Cargo, npm) and registry
> clients (crates.io, npm) wired into `scan`. Source:
> `doc/raw/2026-10-15-phase-2/2026-10-15-phase2.md`. Raw documents are
> immutable; a follow-up supersedes this with a new dated file.

## Summary

Feature work on branch `feat/phase-2`. Added a trait-based manifest parser
subsystem, asynchronous registry clients with bounded concurrency, and CLI
wiring so `scan` now parses declared dependencies and resolves each package's
latest published version. Both parsers and both clients are unit-tested
offline, and the CLI was smoke-tested against the live registries.

Commits on the branch:

- `docs(raw): add Phase 2 task document`
- `feat(scan): parse manifests and resolve latest registry versions`
- `docs(wiki): document Phase 2 parsers and registry clients`

## Modifications

- **`src/model.rs` (new)** — `Ecosystem` (`Cargo`, `Npm`) with a lowercase
  label for console/JSON output.
- **`src/parser/mod.rs` (new)** — `PackageParser` trait, `ParsedDependency`
  DTO, `ParserError`, `default_parsers()`, and the failure-tolerant
  `parse_manifests()`.
- **`src/parser/cargo.rs` (new)** — `[dependencies]`, `[dev-dependencies]`,
  `[workspace.dependencies]`, and `workspace = true` inheritance.
- **`src/parser/npm.rs` (new)** — `dependencies`/`devDependencies`, skipping
  non-registry specs.
- **`src/registry/mod.rs` (new)** — `RegistryClient`, `RegistryError`,
  `DEFAULT_CONCURRENCY = 10`, and bounded `fetch_latest_versions`.
- **`src/registry/crates_io.rs` (new)** — crates.io client with the required
  `User-Agent`, `crate.max_version`, and a pure parse function.
- **`src/registry/npm.rs` (new)** — npm client with `dist-tags.latest`, scoped
  name encoding, and a pure parse function.
- **`src/main.rs`** — `scan` parses then resolves per ecosystem; prints a table
  (default `--format html`) or JSON.
- **`Cargo.toml`** — added `toml`, `reqwest` (json), `futures`.
- **Wiki** — new `doc/wiki/features/phase-2-parsers-registry/` (feature page +
  implementation report); updated `architecture.md`, `roadmap.md`, `index.md`,
  `log.md`.

Design decisions (rationale also captured in the wiki feature page):

- `toml` + serde instead of `cargo_toml` — fewer deps, direct workspace access.
- `futures` `buffer_unordered` instead of `tokio::spawn` + `Semaphore`.
- Native `async fn` in the trait — no `async-trait` dependency.
- Registry/parse failures degrade to `unknown`, never aborting the scan.

## Verification

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 28 passed, 0 failed |
| `cargo run -- scan tmp/phase-2/fixture --format json` | resolved real versions; 404 degraded gracefully |
| `python3 validate_okf.py` (`.claude`, staging shim → `doc/wiki`) | **PASSED** — 0 errors, 0 warnings |

Smoke test resolved serde 1.0→1.0.229, anyhow 1.0→1.0.104, tempfile 3→3.27.0
(crates.io) and express ^4.18.0→5.3.0, typescript ~5.4.0→7.0.2 (npm), and
recorded a crates.io 404 for a deliberately non-existent crate without
aborting.

## Technical debt

- `[build-dependencies]` and npm `optionalDependencies`/`peerDependencies` are
  not extracted yet.
- Cargo `crate.max_version` is reported verbatim, so a prerelease-only crate
  reports a prerelease; the "latest stable" refinement belongs with the SemVer
  work (Phase 3).
- `--format html` still prints the text summary; the HTML report engine is
  Phase 4.
- `scan` now performs network I/O by default. There is no offline/`--no-fetch`
  switch yet; consider one when the report/AI phases land.

## Next recommended actions

1. **Phase 3:** unify `Dependency`/`UpdateType` in `src/model.rs` and implement
   the pure SemVer classifier over the `ParsedDependency` + latest-version data
   this phase produces.
2. Consider a `--no-fetch`/offline flag so `scan` can list dependencies without
   registry calls.
3. Optionally deduplicate the same package appearing across manifests when the
   classifier lands.
