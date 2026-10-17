---
title: "CLI Scanning UX & v0.2.0"
type: feature
description: "Interactive spinners and a per-repository progress bar for scanning, per-repository dependency metrics, a non-TTY/JSON fallback, and the v0.2.0 version bump."
status: active
version: 0.2.0
license: CC-BY-4.0
created: 2026-10-17
updated: 2026-10-17
tags: [feature, cli, ux, progress, indicatif, release]
---

# CLI scanning UX & v0.2.0

## Motivation

GitHub issue #19 ("Improvement: review CLI interface when scanning") reported
that a large scan looks frozen: the scanner walks the tree, parses manifests and
queries five registries, but prints nothing until the whole run finishes. On a
big monorepo that silence is indistinguishable from a hang. The fix is a layer
of interactive progress feedback that shows what the tool is doing and closes
with a per-repository dependency tally — while keeping stdout clean for
machine-readable runs.

The same change prepares release **v0.2.0**, the first release since the tool
gained the five-ecosystem scan and the Docker image.

## What shipped

- **Interactive progress** (`src/progress.rs`): a thin wrapper over `indicatif`.
  An animated spinner covers the two indeterminate steps (manifest discovery and
  registry resolution) and a determinate counter covers per-repository reporting.
- **Per-repository metrics** (`src/main.rs`): one line per discovered manifest,
  e.g. `✔ Scanned 'mono/backend': 18 dependencies (Cargo)`, plus a
  `Scanning repositories [bar] pos/len` counter over the manifest total.
- **Non-TTY & JSON fallback**: animations are drawn only when stderr is an
  interactive terminal *and* the format is not JSON. In CI or a pipe the bars
  collapse to no-ops, and `--format json` additionally suppresses the banner and
  the per-repository status lines so stdout stays a single JSON document.
- **`indicatif`** added to `Cargo.toml`.
- **Version 0.2.0** in `Cargo.toml` and the README banner/badge/Docker examples.

## How it fits together

The scan pipeline is `discover → parse → resolve → classify → report`. The
progress layer is deliberately thin: `main::run_scan` builds a
`progress::ScanProgress` reporter, brackets discovery and resolution with
spinners, and calls `report_repositories` to print the per-manifest lines while
advancing the counter.

```text
spinner  "Discovering dependency manifests…"   (scan_directory)
spinner  "Parsing manifests…"                  (parse_manifests)
bar      "Scanning repositories  [..] X/N"     (one line per manifest + inc)
spinner  "Resolving latest versions…"          (registry lookups, skipped offline)
```

The bar's total `N` is the manifest count, known only *after* discovery — an
`ignore::WalkBuilder` walk cannot report a total up front — which is why
discovery is a spinner and the counter is a separate phase.

## Design decisions

- **`std::io::IsTerminal` for TTY detection, not `atty`.** The trait is in
  std (stable since 1.70), so no extra crate is needed. The check is on
  **stderr**, because that is where the bars are drawn.
- **Bars draw to stderr; human status lines go to stdout.** `--format json`
  writes its document to stdout, so putting the animations on stderr keeps the
  JSON stream byte-clean even on an interactive terminal.
- **`ScanProgress` wraps `indicatif` rather than scattering `ProgressBar`
  calls.** A disabled reporter returns `ProgressBar::hidden()`, so call sites
  never branch on interactivity; the JSON/TTY decision lives in one place.
- **The per-repository line is `✔ Scanned '<relative path>': N dependencies
  (<ecosystem>)`.** The relative path (via `progress::project_label`) matches how
  the HTML report names a project, and the ecosystem label matches the report's
  project pill (`Cargo`, `npm`, `PyPI`, `Go`, `Composer`) via
  `ManifestType::ecosystem_label`.
- **No new CLI flag.** The brief did not ask for a `--quiet`/`--no-progress`
  switch, and interactivity is already inferred from the TTY, so none was added.

## Known limitations

- **The counter is over manifests, not over filesystem work.** Discovery is a
  single walk that cannot be subdivided without changing `scanner.rs`'s API, so
  the `X/N` counter reflects repositories reported, not directories visited.
- **Log and spinner share stderr.** With `--verbose`, `tracing` INFO/DEBUG lines
  can interleave with a spinner frame; the bar redraws on the next tick. This is
  cosmetic and only appears in an interactive, verbose run.

## Related

- [[index]] — knowledge-graph entrypoint.
- [[architecture]] — the CLI pipeline this change instruments.
- [[release-automation]] — the tag-triggered release that will publish v0.2.0.
- [[report-ux-refinements]] — the follow-up that removes orphan progress-bar
  frames.
- [implementation report](implementation-report.md) — file-by-file detail and
  verification.
