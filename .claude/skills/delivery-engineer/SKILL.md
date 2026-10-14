---
name: delivery-engineer
description: Implement a clearly scoped feature or bug fix from a raw document, ticket, or explicit request; create a non-protected branch, verify the change, and open a draft MR/PR. Use when asked to implement, build, or fix product code.
---

# Delivery engineer

Implement code changes in the target code repository. Follow that repository's `CLAUDE.md` and its documented wiki conventions (for example, `wiki/conventions/` for naming, code style, components, stores, branches, and commits) before changing code.

## Preconditions

1. Read the supplied raw document, ticket, or user instructions in full. Its acceptance criteria and scope boundaries are the implementation contract.
2. Identify the target repository and read its local guidance.
3. Obtain the ticket number when the repository's branch convention requires one.
4. If the target repository, scope, acceptance criteria, branch name, or product intent is ambiguous, stop and ask. Do not infer missing product decisions from a terse request.

## Work journal

Before the first change, create the task's journal in the target repository's `tmp/<slug>/` (`<slug>` is the ticket key, or a short kebab-case name) by applying the `work-journal` workflow. If a journal already exists, resume it by reconciling it with git instead of starting over. Update `handoff.md`, `memory.md`, and `log.md` at each milestone. The journal is local and never committed.

## Workflow

1. Use the repository's branch decision tree to classify the change as a feature, bug fix, or hotfix. Create the appropriate branch; never work directly on a protected branch.
2. Decide whether a new user-facing feature needs feature-flag gating when the project supports feature flags. State the conclusion explicitly, including when no flag is needed.
3. Implement only the agreed scope, following local conventions.
4. Apply the `code-review` workflow to the proposed diff when a review is appropriate, and address confirmed findings before opening the MR/PR.
5. Run the repository's required build, lint, and test commands for every change. Resolve failures or report blockers faithfully; do not commit unverified changes as complete.
6. Commit using the repository's conventional-commit rules, push the branch, and open the MR/PR as **draft/WIP**. A feature normally has one draft MR/PR; follow the repository release process if a bug fix requires two.
7. After the implementation branch and draft MR/PR exist, apply the `wiki-scribe` companion workflow rather than leaving documentation as a separate manual task. Provide it a self-contained handoff: implementation summary, ticket/branch/MR-PR references, open questions or deferred scope, and the raw-ingest-ship-loop documentation requirements (dedicated subfolder, `implementation-report.md`, `index.md`/`log.md`, validation, and wiki commit/push).

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

The repository's own commit rules apply on top of this standard.

## Boundaries and safeguards

- Do not write wiki content as part of this workflow; use `wiki-scribe` for the documentation half.
- Do not fetch or triage AI-reviewer findings on an existing MR/PR; use `review-remediator`.
- Do not merge or undraft an MR/PR without the user's explicit approval.
- Never commit or push directly to a protected branch.
- Opening a draft MR/PR is the ship-step deliverable; it is not permission to merge.

## Report

Report the scope implemented, feature-flag decision, validation commands and results, branch and commit, draft MR/PR reference, and the documentation handoff outcome or blocker.
