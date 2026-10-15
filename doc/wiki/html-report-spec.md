---
title: "RustyDeps — HTML Report Specification"
type: reference
description: "Finalized UX/UI specification for the interactive HTML report: industrial dark mode, project-centric grouping, minimalist risk indicators."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-15
tags: [report, ui, design, spec, html, phase-4]
---

# HTML report specification

The interactive HTML report is the user-facing output of a RustyDeps scan. It is
built in Phase 4 (see [[roadmap]]) by the report generator described in
[[architecture]] and implemented in [[phase-4-html-report]]; the final raw
prompt is `doc/raw/2026-10-15/opencode-prompt-wiki-html-spec.md`.

This page is the design contract for the generator: it fixes the visual
language, the component hierarchy, and the interaction model so the
implementation does not drift toward generic dashboard aesthetics.

## 1. Design philosophy — industrial dark mode

Developer-first, high-density, tool-ish. The reference aesthetic is Linear /
Raycast / Vercel, not a marketing dashboard.

**Explicit prohibitions** (these read as "AI-generated template" clichés):

- No glossy gradients, floating blur gradients, or glassmorphism.
- No translucent "AI" pills or saturated background fills behind risk tags.
- No glowing/bright purple "AI Insight" banners.
- No oversized padding or spacious, low-density tables.

**Typography:**

- Monospace (`JetBrains Mono`, falling back to `SF Mono`) for crate/package
  names, SemVer numbers, version jumps, and terminal commands.
- Geometric sans-serif (`Geist Sans`, falling back to `Inter`) for UI chrome,
  labels, and prose.

## 2. Color palette (Zinc-based)

| Token | Value | Use |
|-------|-------|-----|
| Background | `#09090b` (zinc-950) | Page background |
| Card surface | `#18181b` (zinc-900) | Project cards, rows |
| Border | `#27272a` (zinc-800) | 1px hairline borders |
| Text — primary | `#ffffff` | Destination version, headings |
| Text — muted | `#a1a1aa` (zinc-400) | Current version, secondary labels |
| Text — faint | `#71717a` (zinc-500) | Hints, the `AI` tag |
| Risk — breaking | `#f43f5e` / `#f87171` | `● major` dot |
| Risk — at-risk | `#fbbf24` (amber) | `● minor` dot |
| Risk — safe | `#10b981` / `#4ade80` (emerald) | `● patch` dot |

Risk colors are used on solid 6px dots and text only — never as a filled pill
background. All risk colors must pass sRGB contrast against the card surface.

## 3. Project-centric component hierarchy

Dependencies are **grouped by project / repository** (`backend-api`,
`web-dashboard`, `ml-worker`, ...), never shown as one flat table of paths.

```
Report
├─ Header bar        — title + version, scan metrics, global risk summary
├─ Controls bar      — ecosystem filter, risk filter pills, search
├─ Project card *n*  — per repository
│   ├─ Project header  — name, path, ecosystem badge, risk breakdown, Update All
│   └─ Dependency row *n*
│       └─ Inline drawer (expanded) — AI changelog + copyable CLI command
└─ Empty state       — shown when filters/search match nothing
```

### Header bar

`RustyDeps` title in monospace plus a version tag, scan metrics (total projects,
total dependencies, scan duration, e.g. `142ms`), and a compact risk summary of
dot badges (`● 3 breaking`, `● 2 at-risk`, `● 3 safe`).

### Project card header

Displays the project name and path, an **ecosystem badge** (`CARGO`, `NPM`,
`PYPI`, `GO`, ...), the manifest file path (e.g. `Cargo.toml`), a project-level
risk breakdown, and a project-level **Update All** action.

### Dependency row

`[package name] → [version jump] → [risk dot + label] → [1-line AI summary] →
[copy-CLI action + expand chevron]`. There is no heavy "Details" button; the
expand affordance is a subtle chevron revealed on hover. The version jump is
rendered as a muted current version pointing to a crisp destination version
(e.g. `1.2.0 → 2.0.0`).

## 4. Risk indicators

Minimalist, solid **6px dots** with sRGB-compliant contrast, paired with muted
text labels — no saturated background pills:

- `● major` — Breaking changes (red). Maps to `UpdateType::Breaking`.
- `● minor` — At-risk / deprecations / minor breaking (amber-yellow). Maps to
  `UpdateType::AtRisk`.
- `● patch` — Safe updates (green). Maps to `UpdateType::Safe`.

The three levels correspond to the classifier's `UpdateType` values; see
[[architecture]].

## 5. AI changelog summaries

Prominent purple "AI Insight" banners are removed. An AI summary is indicated by
a subtle `AI` tag in neutral grey (`#71717a` / `zinc-400`) beside a concise
one-line release summary. The full changelog break-down lives in the expanded
drawer, not the collapsed row.

## 6. Interactive components

### Inline dependency drawer

Clicking a dependency row expands an in-line drawer directly beneath it
(`.dependency-drawer` / `<details>`), containing:

- the extended AI changelog summary (breaking changes, migration impact);
- a terminal code block with the exact CLI update command (e.g.
  `cargo update -p tokio --precise 2.0.0`) and a one-click copy button.

### Filter and search bar

Global filter pills (`All`, `● Breaking`, `● At-Risk`, `● Safe`), an ecosystem
filter, and a search input with a `/` or `⌘K` shortcut cue.

### Empty state

When a search or filter matches zero dependencies, show a minimalist empty state
(icon + subtle text such as "No dependencies match your filter") with a
**Reset filters** action.

### Project "Update All"

The project-header **Update All** action reveals a popover/snippet containing
the combined update commands for that manifest (e.g.
`cargo update -p tokio && cargo update -p anyhow`).

## 7. Layout and technical constraints

- Single **standalone** `report.html` — embedded `<style>` and minimal vanilla
  `<script>`; no React/Vue or other heavy frameworks.
- Centered layout with a max content width (~1100–1200px) and a responsive
  layout that avoids horizontal scrolling on wide screens.
- Dark mode only.
- Requires no network access to view; opens directly from the filesystem.

## Related

- [[architecture]] — the report generator module this specification drives.
- [[phase-4-html-report]] — the implementation of this specification.
- [[roadmap]] — Phase 4, "Interactive HTML Report Engine".
- [[okf-compliance]] — licensing and validation of this wiki.
- [[index]] — knowledge-graph entrypoint.
- `doc/raw/2026-10-15/opencode-prompt-wiki-html-spec.md` — the raw prompt that
  led to this spec.
- `doc/raw/2026-10-15/opencode-prompt-stitch-integration.md` — the brief for
  adopting the Stitch UI v4 HTML export as the implementation template.
