# Development Report — 2026-10-15T151654Z

> RustyDeps — release automation (tag-triggered GitHub Actions) and a full
> README. Source: `doc/raw/2026-10-15/2026-10-15-phase7-release.md`. Raw
> documents are immutable; a follow-up supersedes this with a new dated file.

## Summary

Repo-infrastructure change on branch `feat/release-automation`. Added a
tag-triggered release workflow that builds Linux and macOS binaries and attaches
them (with a changelog summary) to a GitHub Release, and rewrote the README. The
brief's title said "Phase 6" while its filename said "phase7"; per the
developer's choice this is documented as release tooling, not a roadmap phase,
so the roadmap's Phase 6 (AI Changelog Summarizer) is unchanged.

Commits on the branch:

- `docs(raw): add release automation task document`
- `docs(readme): rewrite with banner, badges, ecosystems and quickstart`
- `ci(release): publish binaries on version tags`
- `docs(wiki): document the release automation and README`
- `docs(raw): add release automation development report` (this file)

## Modifications

- **`README.md`** — rewritten: ASCII banner, v0.1.0/license/Rust/Release badges,
  the live website link (alexdevstheweb.github.io/rustydeps), a
  supported-ecosystems table (Cargo, npm, PyPI, Go, Composer), a risk legend,
  quickstart and usage blocks, a feature overview (auto-open/`--no-open`,
  `--offline`/`--no-fetch`, `--format json`, the HTML report, `--ai`,
  `--verbose`), a Releasing section, and the dual-license section.
- **`.github/workflows/release.yml`** (new) — `build` matrix over
  `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
  (`cargo build --release --locked`, tar.gz + sha256, artifact upload) and a
  `release` job that generates notes and creates the GitHub Release.
- **Wiki** — new `features/release-automation/` (feature page + report);
  `index.md` and `log.md` updated.

Resolved gaps in the brief (blank in the source): trigger on tags matching `v*`;
Linux `x86_64-unknown-linux-gnu` plus both macOS arches; **tag not pushed** now.
The README "Releasing" section documents how to push the tag.

## Verification

| Check | Result |
|-------|--------|
| Workflow YAML parse | OK — jobs build/release; targets verified |
| `bash -n` on both `run:` blocks | OK |
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 85 passed, 0 failed |
| `python3 validate_okf.py` (staging shim) | **PASSED** — 0 errors, 0 warnings |

No live release run: it requires pushing a `v0.1.0` tag, deliberately deferred.

## Technical debt

- No pull-request CI workflow (fmt/clippy/test) — releases are the only workflow
  today.
- Releases are not published to crates.io; installation is `cargo install --git`
  or a downloaded binary.
- Windows binaries are not built.
- The badge URLs for the release workflow return "no status" until the first
  tagged run.

## Next recommended actions

1. Push `v0.1.0` once this workflow is on the default branch to produce the
   first GitHub Release.
2. Optionally add a CI workflow for PRs (fmt/clippy/test) and a crates.io publish
   workflow.
3. Phase 6 (AI Changelog Summarizer) remains the next roadmap phase.
