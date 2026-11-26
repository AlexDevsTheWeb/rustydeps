# Development Report — Landing Page v0.3.1

- **Date:** 2026-11-26
- **Branch:** `fix/landing-page-v0.3.1`
- **Scope:** GitHub Pages landing page refresh to v0.3.1 and a CLI banner
  ecosystem correction.
- **Source:** `doc/raw/2026-11-26/2026-11-26-update-landing-page.md`

## Changes

### `docs/index.html`

- Version and licensing: header/hero/footer now show `v0.3.1` and
  `MIT OR Apache-2.0`.
- Install: tagged Cargo install
  (`cargo install --git https://github.com/AlexDevsTheWeb/rustydeps --tag v0.3.1`)
  and the published GHCR image
  (`ghcr.io/alexdevstheweb/rustydeps:0.3.1`). The container tag omits `v`
  because the Docker workflow uses `type=semver,pattern={{version}}`.
- Feature grid expanded to six cards: multi-ecosystem scanning, SemVer risk,
  four-section AI briefings, local Ollama plus BYOK providers, disk cache and
  the fatal-error circuit breaker, and the self-contained report.
- New AI section: Ollama `config.json`, `--ai --ai-provider ollama`,
  `--no-cache`, `cache clear`, cache path, and the provider roster.
- Terminal demo rebuilt from real CLI output strings. Removed the invented
  `--report` flag and the false exit-code claim.
- Report preview: expanded `@prisma/client` drawer with the real prompt
  headings and the `⚡ Cached (llama3.2:3b)` badge.
- Quickstart: Cargo/Docker tabs reuse the existing `switchTab`, real scan flags
  (`-o`, `--no-open`, `--format json`, `--offline`), optional AI step, and a CI
  recipe.
- Links: release `v0.3.1`, GHCR package page, and README replace stale anchors.
- Removed fixed report-size claims (the file depends on the scan payload).
- Responsive fixes: command wrapping, `min-w-0` on flex/grid children, wrapping
  project headers, and `scroll-mt-16` anchor offsets.

### `src/main.rs`

- `print_banner` subtitle now lists “Rust, Node, Python, Go & PHP” instead of
  “Rust, Node & Python”, matching the five ecosystems and the landing demo.

## Tests executed

- HTML structure via Python `html.parser` — no unclosed or mismatched tags.
- Headless Chromium at 1365×900 and 390×844 — `scrollWidth == innerWidth`, no
  horizontal overflow; Cargo/Docker tab switching correct; 15 `data-copy`
  controls route their exact command to the clipboard handler. Headless
  clipboard write permission was denied by the browser, so the handler was
  verified at the function boundary rather than against the OS clipboard.
- Desktop/mobile screenshots reviewed: hero, feature grid, AI section, report
  preview, quickstart.
- External links: release tag, GHCR package, README — HTTP 200.
- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 144 unit + 4 integration passed.
- OKF validator — PASSED.

## Technical debt

- The landing page still depends on the Tailwind CDN and Google Fonts; it is not
  self-contained like the generated report.
- The report preview is hand-maintained markup and can drift from
  `templates/report.html`.
- `README.md` still carries a v0.2.0 banner and 0.2.0 badges; the brief scoped
  this work to the landing page only.
- The brief's Docker tag `:v0.3.1` does not match the published tag; the page
  uses `:0.3.1`. The brief should be corrected for future readers.

## Next recommended actions

- Open the draft PR into `development`.
- Refresh the README banner/badges to v0.3.1 in a follow-up.
- Consider generating the site's report preview from a real `--format html`
  report fixture so it cannot drift.
