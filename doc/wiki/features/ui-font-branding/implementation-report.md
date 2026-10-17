---
title: "Implementation Report — UI Font Scaling & Branding"
type: report
description: "File-by-file report of the report type-scale bump, the embedded SVG favicon and inline brand logo, and the self-containment test change."
status: done
version: 0.2.0
sources: [issue21]
license: CC-BY-4.0
created: 2026-10-17
updated: 2026-10-17
tags: [implementation-report, report, ui, branding, favicon]
---

# Implementation Report — UI Font Scaling & Branding

## Summary

`doc/raw/2026-10-17/2026-10-17-issue21.md` implements issue #21: raise the
report's smallest fonts and add a favicon and a brand logo. Shipped on branch
`feat/ui-branding-fonts`: `templates/report.html` gets a 12/13/14/16px type
scale, an embedded data-URI SVG favicon and an inline SVG header logo (replacing
the `.rust-dot`), and `src/report/tests.rs` gains a branding test plus a relaxed
self-containment assertion. The report stays a single, fully offline file.

## Done

- **`templates/report.html`**
  - All 47 `font-size` declarations remapped to **12/13/14/16px**
    (10→12, 11/11.5→13, 12/12.5/13→14, 14→16), covering both CSS rules and the
    inline `style="font-size: …"` attributes on the drawer/meta lines.
  - `<link rel="icon" type="image/svg+xml" href="data:image/svg+xml,…">` in
    `<head>`; the SVG namespace is percent-encoded
    (`http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg`) so no literal URL scheme appears.
  - `.logo-icon` CSS rule (18px, `#38bdf8`, `flex-shrink: 0`) and an inline SVG
    sun/spark mark before the `RustyDeps` wordmark; the `.rust-dot` markup and
    its CSS rule were removed.
- **`src/report/tests.rs`**
  - `renders_a_self_contained_document`: the blanket `<link ` ban is replaced by
    "exactly one `<link>`, and it is the `data:` favicon"; the `http://` /
    `https://` / `<script src` bans are unchanged.
  - New `renders_brand_logo_and_favicon`: asserts `class="logo-icon"`,
    the wordmark, the favicon `<link>`, and the absence of `<img>`.
- **`doc/raw/2026-10-17/2026-10-17-issue21.md`** — the immutable source brief.
- **Wiki** — this subfolder, [[ui-font-branding]], and updates to [[index]],
  [[log]] and [[architecture]].

## Not done / deferred

- **Visual regression coverage.** No screenshot/visual-diff harness; the change
  is verified structurally (favicon/logo present, no external references) and by
  generating a report. Manual browser review is the acceptance step.
- **No `<h1>`/title enlargement.** The brief's "keep `text-xl`/`text-2xl` bold"
  line assumes a header shape this template does not have; the wordmark was left
  at the scaled size rather than artificially enlarged.
- **Fixed `.dep-row` grid columns** are unchanged, so long package names still
  rely on the existing ellipsis truncation at the larger size.

## Blocked

None.

## Verification

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 90 passed, 0 failed (89 before; +1 branding).
- `cargo build --release` and
  `./target/release/rustydeps scan . --offline --no-open -o /tmp/report.html` —
  exit 0; the 67608-byte report contains 0 `http://`/`https://` references,
  exactly one `<link>` (the data favicon), one `.logo-icon`, and no `font-size`
  below 12px.
- OKF validator PASSED (0 errors, 0 warnings).

## MR

Draft PR #22 into `development` from `feat/ui-branding-fonts`:
<https://github.com/AlexDevsTheWeb/rustydeps/pull/22>.

## Related

- [[ui-font-branding]] — the feature page for this work.
- [[html-report-spec]] — the report's visual contract.
- [[architecture]] — the report generator that renders this template.
