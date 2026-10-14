---
name: work-journal
description: "Keep a crash-safe local work journal in tmp/<ticket-or-slug>/ for any multi-step delivery, bug-fix, ingest, or review-remediation task: create it before the first change, update handoff.md, memory.md, and log.md at each milestone, and reconcile it with git when resuming. Use when starting, resuming, or continuing a multi-step implementation, remediation, or ingest, or when asked to pick up previous work."
---

# Work journal

Keep the state of a multi-step task on disk so that a new session can resume it without rebuilding context. The journal is built as the work progresses. It is not written at the end.

## Layout

The journal lives in the target repository, not in this catalogue:

```text
<repo-root>/tmp/<slug>/
├── handoff.md   # session state: where the task stands and the next step
├── memory.md    # durable facts: decisions with reasons, constraints, references, ruled-out approaches
└── log.md       # append-only history of milestones, commands, and outcomes
```

- `<slug>` is the ticket key when one exists (for example `PROJ-123`), otherwise a short kebab-case name. Use one folder per ticket. A remediation's later rounds reuse the same folder and record each round in `log.md`.
- `handoff.md` here is session state. It is not the `handoff` skill, which publishes a reviewed payload to a central knowledge base.
- `tmp/` is git-ignored and local. Never commit it, and never publish its files verbatim.

Create the journal with the bundle's script so the layout stays consistent. Run it from inside the target repository:

```bash
python3 <bundle>/scripts/journal.py init <slug>
```

`<bundle>` is this skill's directory in the runtime (for example `.claude/skills/work-journal`). The templates are in `templates/`.

## Lifecycle

1. **Initialize before the first change.** Record the task goal and its source (raw document, ticket, MR/PR, or request), the ticket, the repository path, and the planned branch. If `init` reports that `tmp/` is not git-ignored, stop and ask the user to add `tmp/` to the repository's `.gitignore`. Do not change `.gitignore` silently.
2. **Update at every milestone:** branch created, decision made, validation run, commit made, draft MR/PR opened, reviewer finding triaged, blocker found. Each update changes `handoff.md` (status, done items, next step, last verified HEAD) and appends to `log.md`. Record a decision in `memory.md` with its reason at the moment it is made, not later from memory.
3. **Keep the next step concrete.** `Next step` names one action with the file, command, or question involved. "Continue the work" is not a next step.
4. **Resume by reconciling, not by trusting.** When a session finds an existing journal, read `handoff.md`, `memory.md`, and the end of `log.md`, then run `journal.py state <slug>`. Then apply these rules:
   - Git is authoritative for code state. If the recorded HEAD differs from the current HEAD, read `git log <recorded>..HEAD` and any uncommitted changes before acting, and update the journal to match.
   - The journal is authoritative for intent and reasons. Keep a recorded decision unless the code shows it is no longer valid. If it is invalid, record the change in `memory.md`.
   - Treat the Validation entries as stale whenever HEAD or the working tree has changed since they ran. Run them again.
   - If the journal and git contradict each other in a way you cannot explain, stop and report the contradiction.
5. **Close.** When the work is merged or published, record that in `handoff.md` and stop updating it. Ask the user before deleting `tmp/<slug>/`. Never delete it on your own initiative.

## Feeding the other workflows

- Commit bodies are written from `log.md` and the staged diff, not from memory. The commit standard in the delivery, remediation, ingest, handoff, and wiki workflows depends on this.
- The wiki `implementation-report.md` and the review follow-up section are written from `handoff.md`, `memory.md`, and `log.md`.
- A KB handoff is built by the `handoff` skill from a verified subset of these files. The journal itself is never published.

## Safeguards

- Never write secrets, tokens, passwords, or gateway keys into a journal file. Record where a credential lives instead.
- Record only what was verified. Mark anything unverified as unverified.
- Keep the journal inside the task's repository. Do not write journal files into the wiki, `raw/`, or this catalogue.

## Report

State the journal path, whether it was created or resumed, the reconciliation result (recorded versus current branch and HEAD, commits since the last verified state, working-tree status), and the next step.
