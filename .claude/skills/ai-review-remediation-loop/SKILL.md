---
name: ai-review-remediation-loop
description: "Address genuine unresolved findings from an AI reviewer bot on an open MR/PR: fetch its discussions, cross-check each finding against current code, implement only new findings on a fresh fix branch, open a draft MR/PR into the reviewed branch, and record a review follow-up in the existing wiki page. Never reply to or resolve the original threads. Use when asked to process, address, or loop over AI reviewer or review-bot comments on an MR/PR."
---

# AI review remediation loop

Standing procedure for fixing unresolved AI-reviewer findings on an open MR/PR and folding them into the existing feature/bug wiki page. The procedure is mandatory end to end: fetch, cross-check, implement, verify, draft MR/PR, document, validate, commit. Do not stop at "fix the code".

## Prerequisites

Resolve these from project configuration (`CLAUDE.md`, `wiki/conventions/commits.md`, the platform CLI setup). If any value is missing, stop and ask; do not guess it.

- `<REVIEWER_BOT_IDENTITY>`: the account name of the AI reviewer bot.
- `<HOST_CLI>`: the source-control CLI and host (for example GitLab `glab` or GitHub `gh`) with working authentication.
- The repository path and namespace (`<CODE_REPO>`), the MR/PR number (`<iid>`), and the branch under review.
- The protected-branch list and the commit-message convention.
- The wiki location, including `wiki/conventions/` and `wiki/log.md`.

## Work journal

Use the same journal across every review round of one feature or bug. Its slug is the ticket key, or `mr-<iid>` when there is none. If the journal exists, resume it by reconciling it with git rather than starting over; the previous rounds are its `log.md` history. Create it with the `work-journal` workflow before the first round's first change, and update it at each milestone. The journal is local and never committed.

## Procedure

1. **Fetch unresolved findings.** Use the platform API scoped to the MR/PR. On GitLab, for example, `<HOST_CLI> api "projects/<namespace%2Frepo>/merge_requests/<iid>/discussions?per_page=100"`; on GitHub, the PR review-comments and reviews API. Keep only notes authored by `<REVIEWER_BOT_IDENTITY>`. Two kinds exist and must be handled differently:
   - **Diff-anchored findings** are resolvable. Only unresolved ones are actionable.
   - **General notes** (for example a finding about a file outside the current diff) may not be resolvable through the API and may reappear on every pass. The only way to stop them recurring is to fix the underlying code, not to reply to the note.

2. **Cross-check every finding against the current code before implementing it. Never trust the finding text alone.** Read the referenced file and lines directly. Typically a third to a half of unresolved notes on a given pass are **stale duplicates**: the reviewer's diff base can lag behind a fix that was already applied in an earlier round. Two signals that a note is stale:
   - The described code pattern no longer matches the file.
   - The note's recorded commit SHA is older than the branch's current tip.

   Leave stale findings untouched: no reply, no resolve (see step 7). Implementing the same fix twice is wasted work.

3. **Implement only the genuinely new findings.**
   - **A reviewer-suggested fix is not automatically safe.** A suggested diff can introduce a regression, for example by duplicating protection that already exists elsewhere or by ignoring surrounding code. Apply the same scrutiny as to any other change: understand why it is needed given the existing code, and do not paste it in verbatim.
   - **Check local convention before applying a stylistic suggestion.** Look at the nearest sibling files, not just a repo-wide grep count. If the codebase is already inconsistent on that point, the finding may be low priority and the established local pattern may stay.
   - **A general (non-resolvable) note may require changes outside the MR/PR's own diff.** That is still in scope; implement it on the same remediation branch.
   - If a finding needs an action outside this workflow's reach (rotating a real secret in a third-party dashboard, setting a CI variable), implement everything that can be done in code and state explicitly which manual step remains. Do not describe the finding as fully resolved.

4. **Verify.** Run the project's build, lint, and test commands on every touched file. Never skip this, however small the fix looks.

5. **Branch and commit** in the code repository, following the project's commit convention. Use one branch per feature/bug under review, named `fix/<branch-under-review>-comments`. **Recreate it from the current tip of the branch under review on every round.** Delete any local copy and create it fresh; do not build a new round on a stale local branch, because the previous round's branch was very likely already merged.

6. **Push and open a draft MR/PR** from `fix/<branch-under-review>-comments` **into the branch under review**, not into the main integration branch. This chains the remediation onto the still-open MR/PR instead of opening a second top-level one. Always open it as **draft**. Never merge or undraft without the user's explicit go-ahead.

7. **Never reply to or resolve the original MR/PR threads.** Each comment or resolve action is a change the reviewer reacts to, and it re-triggers a review pass. Leave every thread (fixed, stale, or general) exactly as found. The only intended trigger for the reviewer's next pass is **merging the draft remediation MR/PR into the branch under review**.

8. **Document by updating the existing feature/bug page**, not by creating a new one. Unlike the raw-ingest loop, no new subfolder and no `implementation-report.md` are created. Append a `## Review follow-up (<date>, <Nth> pass)` section to the page that documents the feature or bug under review, containing:
   - Which notes were stale (skipped) and which were genuinely new (fixed), with the reason for each.
   - A findings-to-fix table covering what was actually implemented.
   - The branch, commit, and draft MR/PR reference for this round.
   - Noteworthy observations: a recurring finding, a fix that regressed an earlier fix, or a manual step still outstanding. These inform how much to trust the next pass's suggestions.

   Update `wiki/index.md` only if this loop is touching the page for the first time, which is usually not the case. Append to `wiki/log.md` as usual, using the note IDs and MR/PR link as the `Source:` line instead of a `raw/` path.

9. **Validate** with `./scripts/validate_okf.py` (or the wiki-scribe helper in `skills/wiki-scribe/scripts/` if the project uses it). Fix every error before continuing.

10. **Commit and push the wiki** under the same rules as the raw-ingest loop's wiki step: if the wiki is a separate repository, commit there separately from the code repository and never combine the two commits into one action.

## Commit standard

Every commit must be precise and exhaustive: a reviewer should be able to judge the change from the message and the diff alone.

- **Subject:** `<type>(<scope>): <imperative summary>` in the repository's convention, at most 72 characters, no trailing period. Name the observable change, not the activity: `fix(auth): reject expired refresh tokens`, not `fix stuff` or `update`.
- **Body is required.** Describe what changed and why, file by file or area by area. Include each non-obvious decision with its reason, taken from the journal's `memory.md`.
- **Verification:** list each command that was run and its result. State explicitly anything that was not verified.
- **Scope:** list deferred items and any follow-up that remains.
- **References:** the ticket, the MR/PR, and, for remediation, the review note IDs addressed or skipped.
- **Staging:** stage by explicit path. Read `git diff --cached` in full before committing and confirm that every hunk is described in the message and that no unrelated change is included. Do not use `git add -A` or `git commit -a`.
- **Granularity:** one logical change per commit, unless the repository's convention says otherwise.
- Never put secrets, tokens, or the contents of `tmp/` in a message.
- If the body cannot be written accurately from the diff and the journal, do not commit. Close the gap first.

The repository's own commit rules apply on top of this standard. The wiki repository's commits follow the same standard.

## Safeguards

- Every git-safety rule in the project's `CLAUDE.md` still applies in full.
- Opening a draft MR/PR is not merging it. Merging or undrafting requires the user's explicit decision.
- "The reviewer suggested it" is not sufficient justification. Verify a suggested fix against the existing architecture before implementing it.
- Never reply to, react to, or resolve the original MR/PR threads, including for findings that were fully fixed. The wiki's review follow-up section is the record of what was addressed and why.
- Do not write wiki content outside the existing feature/bug page described in step 8.

## Report

Report the stale-versus-new breakdown of findings, the fixes made and the manual actions still outstanding, the validation commands and results, the branch, commit, and draft MR/PR reference, and the wiki update and validation outcome.
