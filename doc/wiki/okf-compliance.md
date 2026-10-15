---
title: "RustyDeps — OKF Compliance"
type: convention
description: "Open formats, open licensing, metadata traceability, and wiki validation."
status: active
version: 0.1.0
license: CC-BY-4.0
created: 2026-10-14
updated: 2026-10-14
tags: [okf, convention, licensing, validation]
---

# OKF compliance

This wiki follows Open Knowledge Foundation (OKF) principles: content is stored
in open, portable formats; it is openly licensed; and every page carries
traceable metadata. See [[index]] for the catalogue and [[architecture]] for the
subject matter.

## Open formats

- **Pages:** atomic Markdown, one topic per page, human- and LLM-readable.
- **Metadata:** flat YAML frontmatter at the top of every page.
- **Structured data:** JSON Schema for machine-readable contracts where a schema
  is warranted (e.g. the report's embedded data blob).

## Frontmatter schema

```yaml
---
title: "Page title"
type: index | architecture | roadmap | convention | reference | log | report
description: "One-line summary."
status: draft | active | deprecated
version: 0.1.0
license: CC-BY-4.0
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [tag1, tag2]
---
```

`type` is required; `description` is strongly recommended. Fields are flat
key/value pairs — nested maps are avoided because the validator's parser is
flat-only.

## Licensing

- **Code:** dual-licensed MIT / Apache-2.0 (Rust ecosystem convention).
- **Wiki and documentation:** CC-BY-4.0 — reuse with attribution.
- **Source documents (`doc/raw/`):** same CC-BY-4.0 terms unless a file states
  otherwise.

## Traceability

- `doc/raw/` holds immutable source documents; a superseded idea is a new dated
  file, never an edit of an existing one.
- Every wiki change is appended to [[log]] and reflected in [[index]], so
  provenance is recoverable.
- Claims about current behaviour should be checked against the code, not trusted
  from a stale page.

## Validation

The wiki is validated by the pre-existing skill script
`.claude/skills/wiki-scribe/scripts/validate_okf.py`. That script expects a
`wiki/` directory two levels above its own `scripts/` directory; this repository
keeps the bundle at `doc/wiki/`, so validate it with a disposable staging shim
rather than editing the script:

```bash
# From the repository root. STAGE is any scratch dir.
STAGE="$(mktemp -d)"
mkdir -p "$STAGE/scripts"
cp .claude/skills/wiki-scribe/scripts/*.py "$STAGE/scripts/"
ln -s "$(pwd)/doc/wiki" "$STAGE/wiki"
python3 "$STAGE/scripts/validate_okf.py"
```

`STAGE/wiki` maps the bundle the script expects onto `doc/wiki/`, so the
unmodified `.claude/` validator runs against the real wiki. A green run prints
`Result: PASSED. Wiki structure is OKF-compliant.`; fix every error before
committing documentation.

## Related

- [[index]] — knowledge-graph entrypoint.
- [[architecture]] — the system this wiki documents.
- [[roadmap]] — phase plan.
