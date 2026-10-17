---
title: "UI Font Scaling & Branding (Favicon + Logo)"
type: feature
description: "Raises the HTML report's smallest font sizes for readability and adds an inline SVG brand logo plus an embedded data-URI SVG favicon, keeping the report self-contained."
status: active
version: 0.2.0
license: CC-BY-4.0
created: 2026-10-17
updated: 2026-10-17
tags: [feature, report, ui, branding, favicon, accessibility]
---

# UI font scaling & branding

## Motivation

Issue #21 reported two problems with the generated HTML report: the smallest
text (10–12px in table cells, badges, pills and drawer code) is hard to read on
high-resolution screens, and the document lacks a favicon and a brand mark next
to the "RustyDeps" wordmark. This change raises the type scale and adds the
favicon and logo without giving up the report's defining property: it is a
single, fully self-contained file.

## What shipped

- **A raised type scale in `templates/report.html`** — every `font-size` in the
  embedded CSS is mapped up so the smallest text is 12px and the body/table text
  is 14px (see the mapping below).
- **An inline SVG brand logo** in the header, replacing the plain 7px `.rust-dot`
  accent.
- **An embedded data-URI SVG favicon** in `<head>`.

## The type scale

The brief was written against a Tailwind build (`text-xs` → `text-sm`,
`text-sm` → `text-base`), but this template is the Google Stitch export with
bespoke px-valued CSS and no Tailwind classes. The intent was translated to the
file's own scale:

| Before | After | Brief analogue |
|--------|-------|----------------|
| 10px | 12px | below `text-xs` — lifted to a 12px floor |
| 11px, 11.5px | 13px | `text-[11px]` — lifted |
| 12px, 12.5px, 13px | 14px | `text-xs` → `text-sm` |
| 14px | 16px | `text-sm` → `text-base` |

Applied uniformly to all 47 `font-size` declarations (CSS rules and the few
inline `style="font-size: …"` attributes). The result is a four-step scale of
12 / 13 / 14 / 16px — no text below 12px. The header wordmark stays in the
`.logo-badge` at the scaled size; the brief's "keep `text-xl`/`text-2xl` bold"
line refers to a header shape this template does not have, so no artificial
enlargement was introduced.

## Branding

- **Favicon.** A Lucide "sun/spark" mark, percent-encoded as an SVG data URI in
  a `<link rel="icon">`. The SVG namespace is written as
  `http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg` so the document contains no literal
  URL scheme — the report's self-containment test forbids `http://` and
  `https://` outright.
- **Logo.** The same mark inline in the header (`.logo-icon`, 18px, sky-blue
  `#38bdf8`), placed before the `RustyDeps` wordmark. Inline SVG needs no
  `xmlns` in an HTML5 document, matching the template's existing inline icons.

## Design decisions

- **Keep the report self-contained.** The brief's snippets used a literal
  `xmlns='http://www.w3.org/2000/svg'`, which would fail the report's
  `!contains("http://")` invariant, and a `<link>` tag, which the original test
  banned outright. The favicon keeps a `<link>` but with a `data:` URI only, and
  the namespace is percent-encoded; the logo drops `xmlns`.
- **Relax the self-containment test, not the invariant.** The test now allows
  exactly one `<link>` (the `data:` favicon) while still forbidding any
  `http://`/`https://` reference, so no external resource can slip in.
- **The SVG logo replaces the `.rust-dot`.** Adding a second brand mark beside
  the existing orange dot would read as clutter; the icon supersedes the plain
  dot. `.rust-dot` CSS was removed with its markup.

## Known limitations

- **Visual verification is manual.** The report is generated offline and opened
  in a browser; there is no screenshot/visual-regression harness. Correctness is
  covered by structural assertions (favicon present, logo present, no external
  references, no `<img>`).
- **Fixed grid columns.** `.dep-row` uses fixed px columns; at 14px some long
  package names rely on the existing `text-overflow: ellipsis` truncation.

## Related

- [[index]] — knowledge-graph entrypoint.
- [[html-report-spec]] — the report's visual contract.
- [[architecture]] — the report generator that renders this template.
- [[phase-4-html-report]] — the Stitch export this template came from.
- [[report-ux-refinements]] — the follow-up that refines card disclosure and row
  ordering.
- [implementation report](implementation-report.md) — file-by-file detail and
  verification.
