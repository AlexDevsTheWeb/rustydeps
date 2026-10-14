#!/usr/bin/env python3
"""Validate the Obsidian-to-TechDocs link conversion for this repository."""

from pathlib import Path
import sys

from obsidian_links import convert_markdown


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    docs_dir = root / "wiki"
    errors = 0
    for source in sorted(docs_dir.rglob("*.md")):
        try:
            convert_markdown(source.read_text(encoding="utf-8"), source=source, docs_dir=docs_dir)
        except ValueError as error:
            print(f"ERROR: {error}")
            errors += 1
    if errors:
        print(f"TechDocs link check failed: {errors} error(s)")
        return 1
    print(f"TechDocs link check passed: {len(list(docs_dir.rglob('*.md')))} pages")
    return 0


if __name__ == "__main__":
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    raise SystemExit(main())
