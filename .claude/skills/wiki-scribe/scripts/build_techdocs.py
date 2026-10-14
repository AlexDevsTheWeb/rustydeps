#!/usr/bin/env python3
"""Create a disposable MkDocs tree from the Obsidian-compatible wiki."""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path

from obsidian_links import convert_markdown


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    source_dir = root / "wiki"
    docs_dir = args.output / "docs"
    if args.output.exists():
        shutil.rmtree(args.output)
    docs_dir.mkdir(parents=True)

    for source in source_dir.rglob("*"):
        if not source.is_file():
            continue
        target = docs_dir / source.relative_to(source_dir)
        target.parent.mkdir(parents=True, exist_ok=True)
        if source.suffix.lower() == ".md":
            target.write_text(
                convert_markdown(
                    source.read_text(encoding="utf-8"),
                    source=source,
                    docs_dir=source_dir,
                ),
                encoding="utf-8",
            )
        else:
            shutil.copy2(source, target)

    (args.output / "mkdocs.yml").write_text(
        "site_name: GeoIntelligence Factory Frontend Wiki\n"
        "site_description: Architecture and engineering knowledge base for gintel-fe.\n"
        "docs_dir: docs\n"
        "plugins:\n"
        "  - techdocs-core\n",
        encoding="utf-8",
    )
    print(f"Generated TechDocs source at {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
