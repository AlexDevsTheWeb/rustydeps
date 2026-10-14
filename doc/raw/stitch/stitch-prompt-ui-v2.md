# TASK: UI/UX Prompt for Google Stitch — RustyDeps Interactive HTML Report (v2)

## Goal
Design a modern, high-density, **project-centric** interactive HTML report for **RustyDeps**, a Rust CLI dependency scanning tool. 

The report MUST NOT look like a generic AI-generated template (avoid glassmorphism, floating blur gradients, pastel neon translucent pills, and overly padded spacious tables). Instead, aim for a clean, industrial, developer-first aesthetic inspired by tools like **Vercel**, **Linear**, and **Raycast**.

---

## Key Design & UX Directives

### 1. Hierarchy: Project-Centric Accordion View
- Instead of a single flat table with repetitive project paths, organize dependencies **by Project / Repository** (`backend-api`, `web-dashboard`, etc.).
- Each project gets a clean **Project Card / Header** showing:
  - Project name & path (e.g. `~/Projects/backend-api` - `Cargo.toml`).
  - Ecosystem badge (`Cargo`, `npm`, `PyPI`, `Go`, `Composer`).
  - Risk breakdown counter (e.g., `1 Breaking`, `2 Safe`).
  - Expand/Collapse chevron.
- Expanding a project reveals the list of dependencies for that specific project.

### 2. Dependency List & Inline Drawer (No Tagged-off "Details" Button)
- Inside a project card, render a clean, high-density table or row list:
  - **Dependency Name** (Monospace font, e.g. `tokio`, `serde_json`).
  - **Type Badge** (`prod` / `dev`).
  - **Current Version** vs. **Latest Version** (e.g., `1.2.0 → 2.0.0`).
  - **Risk Indicator:** Clean, solid, high-contrast badges with solid status dots (e.g., `● Safe`, `● At-Risk`, `● Breaking`).
  - **AI Summary Preview:** 1-line concise summary of the update.
- Clicking on any dependency row toggles an **In-line AI Drawer / Expansion** directly underneath the row:
  - Shows detailed breaking changes analysis, migration impact, and recommended update command (e.g., `cargo update -p tokio`).
  - Copy-to-clipboard button for the migration command.

### 3. Visual Styling & Color Palette (Industrial Dark Mode)
- **Background:** Deep neutral dark (`#09090b` / `zinc-950`).
- **Cards & Surfaces:** Clean neutral dark (`#18181b` / `zinc-900`) with subtle 1px borders (`#27272a` / `zinc-800`).
- **Typography:**
  - UI Text: `Geist Sans`, `Inter`, or system sans-serif.
  - Code/Package names/Versions: `JetBrains Mono` or `SF Mono`.
- **Status Colors (Solid & High-Contrast, NO translucent pills):**
  - **Safe (Patch/Minor):** Solid dark green badge (`#052e16`) with bright green text (`#4ade80`).
  - **At-Risk (Deprecations/Minor Breaking):** Solid dark amber badge (`#451a03`) with bright amber text (`#fbbf24`).
  - **Breaking (Major Version Jump):** Solid dark red badge (`#450a0a`) with bright red text (`#f87171`).
  - **AI Insights:** Subtle purple highlight (`#3b0764` container, `#c084fc` text/icon).

### 4. Top Header & Global Controls
- **Header:**
  - `RustyDeps` logo/title in clean monospace + version tag (`v0.1.0`).
  - Summary Metrics: Total Projects, Total Dependencies, Scan Duration (e.g., `142ms`), Risk Breakdown (Safe / At-Risk / Breaking).
- **Controls Bar:**
  - Global Search bar (`Filter packages or projects...`) with keyboard shortcut cue (`⌘K` or `/`).
  - Risk Filter Tabs (`All`, `Breaking Only`, `At-Risk`, `Safe`).
  - Ecosystem Filter dropdown or toggle buttons (`All`, `Cargo`, `npm`, etc.).

### 5. Technical Constraints
- Output a single standalone HTML file (`report.html`) with embedded CSS (`<style>`) and minimal vanilla JavaScript (`<script>`).
- No external heavy JS frameworks (React, Vue, etc.).
- Responsive layout with maximum container width (e.g., `max-width: 1200px` centered) to prevent wide-screen clipping or horizontal scrolling.

---

## Expected Output
Provide a complete, modern, standalone `report.html` file adhering to all the above specifications.
