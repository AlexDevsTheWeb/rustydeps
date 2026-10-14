---
name: wiki-scribe
description: "Maintain an OKF/Obsidian-compatible project wiki: write or update linked pages, keep its index and append-only log current, validate before documentation commits, and document delivery or review-remediation follow-ups. Use for documentation changes and wiki-grounded project answers."
---

# Wiki scribe

Maintain the project's OKF (Obsidian-compatible Knowledge Format) wiki. Before the first action, read the target repository's `CLAUDE.md` in full; it is authoritative for frontmatter, page types, filenames, folder layout, and linked standing loops. A request from a companion workflow is authoritative input about what was implemented, fixed, or discovered, but still verify live code when documenting current behavior.

## Scope

- Write and update pages in the documented wiki areas, including architecture, patterns, conventions, features, bugs, decisions, and boundaries.
- Keep `wiki/index.md` as the catalogue and `wiki/log.md` as the append-only history current for every wiki change.
- Answer wiki questions from the team knowledge base first, through `team-kb-search`. If the KB MCP server does not respond, answer from the wiki instead: read `wiki/index.md`, then the relevant pages, and cite them using `[[page-name]]`.
- Before writing a page, check the team knowledge base through `team-kb-search` for the same topic, so that knowledge is not duplicated. If the KB MCP server does not respond, check `wiki/index.md` for an existing page instead, and record in the report that the KB check was skipped.
- Complete documentation steps from `[[raw-ingest-ship-loop]]`: create a new page in a dedicated subfolder, include `implementation-report.md`, and update the index/log.
- Complete documentation steps from `[[ai-review-remediation-loop]]`: append `## Review follow-up` to the **existing** feature or bug page; never create a new subfolder for that follow-up.

## Documentation workflow

1. Read local wiki guidance and identify the correct page type, placement, and related pages.
2. Add valid YAML frontmatter to each page (`type` is required and `description` is recommended) according to the repository's schema.
3. Add outbound and inbound links so no new page is orphaned.
4. Treat `raw/` as read-only: it may be input, never a documentation target.
5. When a claim needs current code verification, read the code repository directly rather than trusting a possibly stale wiki page.
6. Keep `wiki/index.md` and `wiki/log.md` synchronized with the change.
7. Before **every documentation commit**, run the bundle-relative validator:

   ```bash
   python3 skills/wiki-scribe/scripts/validate_okf.py
   ```

   When the bundle is staged elsewhere, run its `scripts/validate_okf.py` from the bundle root or an equivalent path. Fix all reported errors; never commit around a validation failure.
8. Commit and push only under the target repository's documented branch and commit rules.

## Bundled helper scripts

All helper scripts live in this skill bundle's `scripts/` directory. Use them only when their prerequisites apply:

- `scripts/validate_okf.py` — validate OKF frontmatter, links, and required page content; mandatory before each documentation commit.
- `scripts/check_wiki_coverage.py` — inspect a configured code repository for undocumented feature/fix work.
- `scripts/check_techdocs_links.py` — validate that OKF links can be converted for TechDocs.
- `scripts/build_techdocs.py` — create a disposable MkDocs/TechDocs tree from the wiki.
- `scripts/obsidian_links.py` — library used to convert supported Obsidian links without modifying sources.
- `scripts/publish_docs_branch.py` — publish converted documentation only when the required GitLab CI environment variables, repository conventions, and explicit authorization are present.

Do not assume a GitLab instance, a docs branch, a wiki layout, or credentials exist merely because a helper script supports them.

## Commit standard

Every documentation commit must be precise and exhaustive: a reviewer should be able to judge the change from the message and the diff alone.

- **Subject:** `docs(<scope>): <imperative summary>` in the wiki repository's convention, at most 72 characters, no trailing period. Name the page or area changed, for example `docs(features): add implementation report for PROJ-123`.
- **Body is required.** List each page created or changed, and what changed in it. State the index and log updates, and the links added.
- **Source:** name the source of each claim: the raw document, the journal entry, the MR/PR, or the code file verified. Do not record a claim as current without checking the code.
- **Validation:** name the validator command and its result.
- **Staging:** stage by explicit path. Read `git diff --cached` in full before committing and confirm that every hunk is described in the message. Do not use `git add -A` or `git commit -a`.
- Never put secrets, tokens, or the contents of `tmp/` in a message.
- If the body cannot be written accurately from the diff, do not commit. Close the gap first.

The wiki repository's own commit and branch rules apply on top of this standard.

## Boundaries and safeguards

- Do not implement, edit, or commit application code; direct code work to `delivery-engineer` or `review-remediator`.
- Do not open or merge application-code MR/PRs. This workflow documents application-code changes; it does not create them.
- Do not choose branch strategy or enforce application-code commit formats outside the wiki repository.
- Do not claim an external documentation publication occurred until its prerequisites and result are verified.

## Report

State pages changed, index/log updates, links added, code verification performed, validator command and result, commit/push references when applicable, and any integration or validation blocker.
