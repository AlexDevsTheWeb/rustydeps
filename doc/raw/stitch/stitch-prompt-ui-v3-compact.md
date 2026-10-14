# STITCH PROMPT V3 (ULTRA-CLEAN / LINEAR-STYLE UI)

Create a single-file standalone HTML/CSS/JS dashboard for "RustyDeps". Dark mode ONLY (Zinc-950 `#09090b`). Inspiration: Linear, Raycast, Vercel.

STRICT DESIGN RULES (NO AI-GENERATED UI CLICHÉS):
1. PALETTE & ACCENTS (MAX 2 ACCENTS PER ROW):
   - Background: `#09090b`. Cards: `#121215` with subtle border `#222226`.
   - NO bright purple/magenta "AI Insight" pills! Replace the purple badge with a tiny subtle muted gray icon (✨ or AI in 11px monospace, color `#71717a`), or inline muted italic text.
   - NO fat saturated colored pills for risk! Use ultra-clean minimal status indicators: a 6px colored dot + muted text (e.g., `● major` in pink/red `#f43f5e`, `● minor` in amber `#fbbf24`, `● patch` in emerald `#10b981`). NO background fills behind risk badges.
   - Version jumps: clean monospace, muted current version (`#a1a1aa`) -> destination version in crisp white or subtle accent (`#ffffff`).

2. LAYOUT & HIERARCHY:
   - Max container width: 1100px centered.
   - Header: Compact top bar with app title, total stats, and subtle dot badges (`● 3 breaking`, `● 2 at-risk`, `● 3 safe`).
   - Project-Centric Accordions: Grouped by Repository/Project (`backend-api`, `web-dashboard`, etc.).
   - Clean Rows: Each row has:
     [Package Name] -> [Version Jump] -> [Minimal Risk Dot + Label] -> [Muted AI Insight Text] -> [Action Icons (Copy CLI / Expand Chevron)].
   - NO heavy "Details v" buttons. Use a simple, subtle chevron icon `⌵` on hover.

3. INTERACTIVITY:
   - Click package row to expand an in-line dark drawer with changelog break-down & migration command (`cargo update -p tokio`).
   - Clean search bar with shortcut hint `/`.

Output ONLY executable standalone HTML with embedded `<style>` and `<script>`.
