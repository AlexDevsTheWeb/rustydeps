# Stitch Refinement Prompt — RustyDeps Dashboard HTML

Refine the current RustyDeps HTML dashboard to include these 3 technical features:

1. **Inline Expanded Drawer (Dependency Details):**
   - Inside package rows (especially for `major`/`breaking` updates), include a hidden expandable drawer (`.dependency-drawer` or `<details>`).
   - Content inside drawer:
     - Extended AI changelog summary card.
     - Terminal code block with a one-click copy button containing the exact CLI update command (e.g. `cargo update -p tokio --precise 2.0.0`).

2. **Search / Filter Empty State:**
   - Add a hidden empty state container (`#empty-state` / `.empty-state`) to display when search or filter pills return 0 matches.
   - Design: Minimalist icon, subtle text *"No dependencies match your filter"*, and a quick *"Reset filters"* button.

3. **Project "Update All" Action:**
   - Enhance the `Update All` button in each project header so clicking or hovering it displays a popover/modal or snippet containing all combined update commands for that manifest (e.g., `cargo update -p tokio && cargo update -p anyhow`).

Maintain the exact same industrial dark mode styling (Linear/Vercel inspired, high contrast, zero glossy gradients or translucent AI badges).
