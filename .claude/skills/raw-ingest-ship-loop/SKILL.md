---
name: raw-ingest-ship-loop
description: "Turn a raw/ document into a shipped, documented change end to end: read it in full, clarify scope, pick the feature, bug-fix, or hotfix branch, implement, commit, open a draft MR/PR, record the work in a dedicated wiki subfolder with an implementation report, validate, and commit the wiki. Use when asked to analyze and ingest a raw document, or to implement a raw note from start to finish."
---

# Raw → implement → ship → document loop

Standing procedure for turning a `raw/` document into a shipped, documented change. Triggered manually each time, with no steps skipped or reordered once started. Running only the wiki-ingest half is not acceptable.

## Prerequisites

Resolve these from project configuration (`CLAUDE.md`, `wiki/conventions/commits.md`). If any value is missing, stop and ask; do not guess.

- `<CODE_REPO>`: the code repository to change, and its own `CLAUDE.md` rules.
- `<PROTECTED_BRANCHES>`: branches that must never receive direct commits or pushes.
- `<INTEGRATION_BRANCH>` (for example `development`) and `<RELEASE_BRANCH_PATTERN>` (for example `release/*`).
- The commit-message convention and the ticket-key format.
- The wiki location. If the wiki is a separate repository, see the two-repo split in `CLAUDE.md`.

## Work journal

Once the raw document is read and the ticket is known, create the task's journal in the code repository's `tmp/<TICKET-KEY>/` by applying the `work-journal` workflow, before the first change. Update it at each milestone in this procedure. The journal is local and never committed; the wiki's `implementation-report.md` is written from it.

## Procedure

1. **Read the raw document in full.** Do not skim. Its acceptance criteria and scope boundaries are the contract for what gets built.

2. **Clarify before implementing, if needed.** If the document leaves scope, acceptance criteria, or the target repository ambiguous, ask the developer before writing code. Do not infer product intent from a terse note.

3. **Determine the branch type** using the decision table below. Ask for the ticket number if the document does not supply one; it is required for feature and bug-fix branch names.

4. **Create the branch and implement** in `<CODE_REPO>`, following its `CLAUDE.md` and conventions. Never work directly on a protected branch.

5. **Commit** using the project's commit-message convention.

6. **Push the branch and open a draft MR/PR.** A feature gets one draft MR/PR. A bug fix gets two, one into the release branch and one into the integration branch (see below). Open each as **draft/WIP**, never ready-for-review. Do not merge or undraft without the user's explicit go-ahead; opening the draft is the deliverable of this step.

7. **Document in the wiki** in a dedicated subfolder (see "Loop-driven pages always get a subfolder"). Update `wiki/index.md` and append to `wiki/log.md`.

8. **Write `implementation-report.md`** inside that subfolder, using the template below. Set the draft MR/PR description to a short summary plus a relative link to this file. Do not duplicate the full report into the MR/PR body.

9. **Validate** with `./scripts/validate_okf.py` (or the wiki-scribe helper in `skills/wiki-scribe/scripts/`). Fix every error before continuing. Never commit around a failure.

10. **Commit and push the wiki.** If the wiki is a separate repository with a permissive default branch, this can be a direct commit. If the wiki lives inside the code repository, follow that repository's normal branch rules.

When the wiki is a separate repository, steps 4 to 6 happen entirely in the code repository and steps 7 to 10 entirely in the wiki repository. Never combine a code commit and a wiki commit into one action; the two repositories have separate remotes and unrelated histories.

## Loop-driven pages always get a subfolder

This overrides the general "single file by default" grouping rule, **but only for pages created by this loop**. Manual or ad-hoc wiki pages keep the single-file default. Do not move existing standalone pages into subfolders just because this procedure exists.

Each feature or bug produced by this loop lives in its own subfolder from the start:

```
wiki/features/<TICKET-KEY>-short-description/
  <TICKET-KEY>-short-description.md   # the normal feature page, same template as always
  implementation-report.md
```

Bug fixes use the same shape under `wiki/bugs/`. Do not add an `index.md` inside the subfolder: `wiki/index.md` and `wiki/log.md` remain the single catalogue and history. Wiki links (double square brackets around the filename) are unaffected by the subfolder.

### `implementation-report.md` template

```markdown
---
title: "Implementation Report — <TICKET-KEY> <short description>"
type: report
status: draft | done
sources: [<TICKET-KEY>]
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: ["implementation-report"]
---

## Summary
One paragraph: what was asked, what was shipped.

## Done
What was implemented, file by file or area by area.

## Not done / deferred
Anything in scope that was not completed, and why.

## Blocked
Open questions, missing information, or dependencies that stalled progress. If there are none, say so explicitly rather than omitting the section.

## MR
Link(s) to the draft MR(s)/PR(s) opened in step 6.

## Related
Link back to the feature/bug page in the same subfolder.
```

`type: report` is a page type scoped to this loop. Add it to the frontmatter schema types table in `CLAUDE.md` if it is not already there.

## Branch strategy decision

This adds no branch type beyond the project's commit convention. It tells the agent which existing type applies.

| Situation | Branch type | Name | Branch from | Merge into |
|---|---|---|---|---|
| New feature or capability (the common case) | Feature | `feat/<TICKET-KEY>` | `<INTEGRATION_BRANCH>` | `<INTEGRATION_BRANCH>` |
| Bug found during QA or in a release under test | Bug fix | `fix/<TICKET-KEY>` | the active `<RELEASE_BRANCH_PATTERN>` branch | **both** the release branch **and** `<INTEGRATION_BRANCH>` |
| Urgent production issue | Hotfix | `hotfix/<TICKET-KEY>` | Ask the user to confirm branch-from and merge-into before creating one; do not assume | Ask the user |

**How to decide:** read the raw document's own framing first. A note describing new user-facing behavior is a feature. A note describing something broken in an already shipped or QA-tested flow is a bug fix. If it is genuinely ambiguous, ask rather than guess: the two paths have different branch sources and merge targets, and a wrong guess means redoing the MRs/PRs.

**If no release branch exists:** the bug-fix path is dormant until one does. If a raw document describes a QA bug fix and no release branch exists, stop and ask the user how to proceed rather than inventing a branch source.

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

- Every git-safety rule in the project's `CLAUDE.md` still applies in full: run git against the specific subrepository, never commit or push to protected branches directly, and ask for the branch name when it cannot be inferred.
- Opening an MR/PR is not merging it. Step 6 stops at "draft MR/PR open". Merging and undrafting still require the user's explicit decision.
- Automation does not lower the bar on step 2. Ask when the raw document is unclear.

## Report

Report the scope implemented, the branch type and name, the commits, the draft MR/PR references, the wiki subfolder and pages updated, the validation result, and any blocked or deferred items.
