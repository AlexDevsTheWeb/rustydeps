# Development Report — 2026-10-15T142210Z

> RustyDeps — Phase 5 implemented: PyPI, Go, and Composer ecosystem support.
> Source: `doc/raw/2026-10-15/2026-10-15-phase5.md`. Raw documents are
> immutable; a follow-up supersedes this with a new dated file.

## Summary

Feature work on branch `feat/phase-5-ecosystems`. Added three parsers
(`PypiParser`, `GoParser`, `ComposerParser`) and three registry clients
(`PypiClient`, `GoRegistryClient`, `ComposerClient`) behind the existing
`PackageParser`/`RegistryClient` traits, extended `Ecosystem`, and taught the
classifier and report about the new ecosystems. A scan now resolves five
ecosystems, and the report renders them with no template change. Per the
developer's choice, the ecosystems are "Phase 5"; the AI changelog summarizer
becomes the next phase in the roadmap.

Commits on the branch:

- `docs(raw): add Phase 5 ecosystems task document`
- `feat(ecosystems): add PyPI, Go and Composer support`
- `docs(wiki): document Phase 5 PyPI, Go and Composer ecosystems`
- `docs(raw): add Phase 5 development report` (this file)

## Modifications

- **`src/parser/pypi.rs`, `go.rs`, `composer.rs` (new)** — the three parsers.
- **`src/parser/mod.rs`** — registered them; a test asserts every
  `ManifestType` has a parser.
- **`src/registry/pypi.rs`, `go.rs`, `composer.rs` (new)** — the three clients
  with pure `parse_*` functions and offline tests.
- **`src/model.rs`** — `Ecosystem` gains `Pypi`, `Go`, `Composer`.
- **`src/classifier/mod.rs`** — accepts a leading `v` and `!`; `latest_stable`
  returns the winning entry verbatim.
- **`src/main.rs`** — generic `resolve_ecosystem` helper; all five ecosystems
  resolve concurrently.
- **`src/report/mod.rs`** — ecosystem labels and update commands.
- **Wiki** — new `features/phase-5-ecosystems/`; `architecture.md`,
  `roadmap.md` (Phase 5/6 reorder), `index.md`, `log.md`.

Design decisions (rationale in the wiki feature page):

- One `PypiParser` claims both `pyproject.toml` and `requirements.txt`.
- Go keeps the `v` prefix so exact-pin minor bumps classify as `AtRisk`.
- Go proxy `@v/list` (highest stable) over `@latest`; Packagist `p2` over the
  legacy endpoint.
- No report template change; the Stitch `.ecosystem-pill` is generic.

## Verification

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 83 passed, 0 failed |
| live `scan --format json` (6 manifests) | resolved all five ecosystems |
| live `scan -o report.html` | PyPI/Go/Composer pills + correct commands; 0 external refs |
| `python3 validate_okf.py` (staging shim) | **PASSED** — 0 errors, 0 warnings |

Sample resolutions: `flask =2.3.0 → 3.1.3` breaking, `requests >=2.28.0 →
2.34.2` safe, `pydantic ~2.5 → 2.14.1` at-risk, `gin v1.9.1 → v1.12.0` at-risk
(indirect ones skipped), `monolog/monolog ^3.0 → 3.12.1` safe,
`phpunit/phpunit ^10.5 → 13.4.2` breaking, `php`/`ext-json` skipped.

## Technical debt

- `[project.optional-dependencies]` (extras) and Poetry dependency groups are
  not parsed.
- Go `replace` directives are not resolved and `// indirect` dependencies are
  intentionally skipped.
- Composer platform filtering is name-prefix based.
- PyPI/Go/Composer versions are compared with the same semver logic as Cargo
  and npm; PEP 440 and Go pseudo-versions are handled approximately.

## Next recommended actions

1. **Phase 6:** implement the AI Changelog Summarizer (live provider calls,
   changelog fetching), replacing the Phase 3 templated summaries and widening
   `AtRisk` with deprecation data.
2. Optionally add Poetry dependency groups / PEP 621 extras and Go `replace`
   resolution if user demand appears.
