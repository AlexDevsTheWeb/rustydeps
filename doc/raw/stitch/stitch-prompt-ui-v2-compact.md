# Compact Prompt for Google Stitch (RustyDeps Report UI)

Design a modern, high-density developer dashboard for software dependency security and version updates (RustyDeps).

### Core Design Rules:
1. **Aesthetic & Color Palette**: Industrial Dark Mode (`#09090b` zinc-950, `#18181b` cards, `#27272a` 1px borders). Absolutely NO colorful background gradients and NO translucent/glassmorphism pill badges.
2. **Typography**: Monospaced font for package names, versions, and CLI commands (`JetBrains Mono`), clean sans-serif for UI text (`Inter`/`Geist`).
3. **Structure (Project-Centric)**: Group dependencies inside expandable **Project/Repository Cards** (e.g., `backend-api`, `web-frontend`). Avoid flat, repetitive tables.
4. **Minimalist Risk Badges**: High-contrast solid dot indicators:
   - Green `● Safe` (Patch/Minor)
   - Amber `● At-Risk` (Deprecations)
   - Red `● Breaking` (Major)
   - Purple badge `AI Summary` for release note insights
5. **Interactivity & Details**:
   - Click Project Card -> toggles dependency list.
   - Click Dependency Row -> expands inline drawer showing AI changelog summary and a copyable CLI command (e.g., `cargo update -p <name>`).
6. **Layout Constraints**: Enforce `max-width: 1200px` centered layout. Ensure zero horizontal scrolling and no truncated action columns.
