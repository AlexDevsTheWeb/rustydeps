# Development Report — 2026-10-15T064033Z

> RustyDeps — raw-document reorganization plus a wiki-reference refresh. Source:
> the user-created `doc/raw/2026-10-15/opencode-prompt-wiki-html-spec.md` and
> the accompanying `doc/raw/` moves. Raw documents are immutable; a follow-up
> supersedes this with a new dated file.

## Summary

Documentation-only change. The prompt
`doc/raw/2026-10-15/opencode-prompt-wiki-html-spec.md` was assessed: it is
byte-for-byte identical to the already-shipped
`doc/raw/stitch/opencode-prompt-wiki-html-spec.md`, and its content was
implemented in [[html-report-spec]] (commit `d69bc89`, merged as PR #3). No new
spec page was therefore required. Instead, the reorganization of `doc/raw/`
that the user made in the working tree was committed, and the wiki paths that
named the old raw locations were repointed.

## Modifications

- **`doc/raw/`** — moved `2026-10-15-phase-2/2026-10-15-phase2.md` and
  `stitch/opencode-prompt-wiki-html-spec.md` into `doc/raw/2026-10-15/`;
  removed the superseded Stitch iteration prompts (`stitch-prompt-ui-v2.md`,
  `-v2-compact`, `-v3-compact`, `-v4-refinements`); added the new Phase 3 brief
  `doc/raw/2026-10-15/2026-10-15-phase3.md`.
- **`doc/wiki/html-report-spec.md`** — intro and Related section now point at
  `doc/raw/2026-10-15/opencode-prompt-wiki-html-spec.md`; `updated` bumped.
- **`doc/wiki/index.md`**, **`doc/wiki/roadmap.md`**,
  **`doc/wiki/features/phase-2-parsers-registry/implementation-report.md`** —
  Phase 2 brief path updated to `doc/raw/2026-10-15/2026-10-15-phase2.md`.
- **`doc/wiki/log.md`** — appended an entry for the reorg; earlier entries keep
  the old paths because the log is append-only.
- This report.

Commits on branch `docs/raw-reorg`:

- `docs(raw): reorganize Phase 2 and UI prompt raw documents`
- `docs(wiki): repoint raw-document references after reorg`
- `docs(raw): add raw-reorg documentation report` (this file)

## Verification

| Command | Result |
|---------|--------|
| `python3 validate_okf.py` (`.claude`, staging shim → `doc/wiki`) | **PASSED** — 0 errors, 0 warnings |
| `diff` old vs new prompt doc | IDENTICAL — confirms the task was already implemented |
| `grep -rn "doc/raw/stitch\|doc/raw/2026-10-15-phase-2" doc/wiki` | only append-only log history retains the old paths |

No Rust build/lint/test run: no code changed in this task. `cargo` gates were
last green on the Phase 2 code (PR #4).

## Technical debt

- The raw-immutability rule means the historical development report
  `doc/raw/development-report-20261014T162310Z.md` still names the old
  `doc/raw/stitch/` paths. That is intentional; a reader should treat raw
  reports as point-in-time records.
- The new `doc/raw/2026-10-15/2026-10-15-phase3.md` brief is committed but not
  yet implemented.

## Next recommended actions

1. **Phase 3:** implement the committed brief — SemVer classifier, AI summary
   module, ASCII banner, and the `--offline`/`--no-fetch` flag.
2. When Phase 4 builds the report generator, use [[html-report-spec]] as the
   design contract (unchanged by this task).
