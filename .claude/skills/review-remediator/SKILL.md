---
name: review-remediator
description: "Safely remediate genuine unresolved findings from a configured AI reviewer on an open MR/PR: verify each against current code, make tested fixes on a fresh draft remediation branch, and never reply to or resolve original threads. Use when asked to address AI reviewer feedback."
---

# Review remediator

Run the repository's AI-review remediation loop. Before the first action, read `wiki/conventions/ai-review-remediation-loop.md` in full. It is authoritative. The reviewer-bot identity, source-control host, API details, and repository access are prerequisites: confirm them from project configuration before treating the workflow as active. If any are absent, stop and state what must be configured; do not fabricate review findings or API behavior.

## Scope

- Fetch unresolved findings authored by the configured reviewer bot for the named MR/PR.
- Cross-check every finding against the current code; unresolved findings frequently include stale duplicates from previous passes.
- Implement only genuine, currently applicable findings and follow local conventions.
- Run required build, lint, and test commands for every touched change.
- Create a fresh `fix/<branch-under-review>-comments` branch from the reviewed branch's current tip for every pass.
- Commit, push, and open a **draft** remediation MR/PR from that branch **into the branch under review**.

## Work journal

Use the same journal across every review round of one feature or bug, in `tmp/<slug>/` of the target repository (`<slug>` is the ticket key, or `mr-<iid>` when there is none). Resume an existing journal by reconciling it with git instead of starting over. Apply the `work-journal` workflow to create or resume it before the first change, and update it at each milestone. The journal is local and never committed.

## Workflow

1. Confirm the MR/PR, reviewed branch, reviewer identity, API configuration, and the governing remediation-loop document.
2. Retrieve unresolved reviewer findings and create a stale-versus-new assessment based on current code.
3. For each genuine finding, verify the proposed code change is safe and consistent with repository conventions before implementing it.
4. Run required validation. If an issue requires an external action (for example secret rotation or a CI variable), implement the available code-side work and record the exact manual follow-up; do not call the finding fully resolved.
5. Recreate the remediation branch from the reviewed branch's current tip, commit, push, and open the draft remediation MR/PR into the reviewed branch.
6. Apply `wiki-scribe` for the documentation half. Provide the stale-versus-new assessment, findings-to-fix table, branch/commit/draft-MR-PR references, and requirement to append `## Review follow-up (<date>, <Nth> pass)` to the **existing** feature/bug page. Never create a new subfolder for a review follow-up.

## Commit standard

Every commit must be precise and exhaustive: a reviewer should be able to judge the change from the message and the diff alone.

- **Subject:** `<type>(<scope>): <imperative summary>` in the repository's convention, at most 72 characters, no trailing period. Name the observable change, not the activity: `fix(auth): reject expired refresh tokens`, not `fix stuff` or `update`.
- **Body is required.** Describe what changed and why, file by file or area by area. Include each non-obvious decision with its reason, taken from the journal's `memory.md`.
- **Verification:** list each command that was run and its result. State explicitly anything that was not verified.
- **Scope:** list deferred items and any follow-up that remains.
- **References:** the ticket, the MR/PR, and the review note IDs addressed or skipped in this round.
- **Staging:** stage by explicit path. Read `git diff --cached` in full before committing and confirm that every hunk is described in the message and that no unrelated change is included. Do not use `git add -A` or `git commit -a`.
- **Granularity:** one logical change per commit, unless the repository's convention says otherwise.
- Never put secrets, tokens, or the contents of `tmp/` in a message.
- If the body cannot be written accurately from the diff and the journal, do not commit. Close the gap first.

The repository's own commit rules apply on top of this standard. The wiki repository's commits follow the same standard.

## Non-negotiable safeguards

- **Never reply to, react to, or resolve the original MR/PR discussion threads.** Such actions can re-trigger the reviewer. The next pass must be caused only by merging the draft remediation MR/PR into the branch under review.
- General non-resolvable notes cannot be marked resolved through the API; address their underlying code issue when applicable.
- Never commit or push directly to a protected branch.
- Do not merge or undraft an MR/PR without explicit user approval.
- Do not write wiki pages directly; apply `wiki-scribe` for the documentation work.

## Report

Report the configuration verified, stale-versus-new finding breakdown, fixes made and deferred manual actions, validation results, branch/commit/draft-MR-PR references, and documentation handoff result.
