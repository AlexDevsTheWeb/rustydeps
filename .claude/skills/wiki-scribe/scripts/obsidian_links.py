"""Convert simple Obsidian wikilinks for MkDocs without changing source files."""

from __future__ import annotations

import re
import os
from pathlib import Path
from typing import Any

WIKILINK = re.compile(r"(?<!\!)\[\[([^\]|#]+)(#[^\]|]+)?(?:\|([^\]]+))?\]\]")
CURRENT_HEADING = re.compile(r"(?<!\!)\[\[#([^\]|]+)(?:\|([^\]]+))?\]\]")
FENCE = re.compile(r"^\s*(```+|~~~+)")


def _pages(docs_dir: Path) -> dict[str, list[Path]]:
    pages: dict[str, list[Path]] = {}
    for path in docs_dir.rglob("*.md"):
        relative = path.relative_to(docs_dir).with_suffix("")
        keys = {relative.as_posix(), path.stem}
        if path.name == "index.md":
            keys.add(relative.parent.name or "index")
            if not relative.parent.parts:
                keys = {"index", ""}
        for key in keys:
            pages.setdefault(key, []).append(path)
    return pages


def _heading_anchor(value: str) -> str:
    value = re.sub(r"[`*_]", "", value).lower()
    value = re.sub(r"[^a-z0-9\s-]", "", value)
    return re.sub(r"\s+", "-", value).strip("-")


def _resolve(target: str, source: Path, docs_dir: Path, pages: dict[str, list[Path]]) -> Path:
    target = target.strip().replace("\\", "/")
    if target.startswith("/"):
        target = target[1:]
    candidates = pages.get(target, [])
    if not candidates:
        candidates = pages.get(Path(target).stem, [])
    if len(candidates) > 1:
        local = source.parent / f"{Path(target).stem}.md"
        root = docs_dir / f"{Path(target).stem}.md"
        preferred = [path for path in candidates if path in (local, root)]
        if len(preferred) == 1:
            return preferred[0]
    if len(candidates) != 1:
        if not candidates:
            raise ValueError(f"unresolved Obsidian link [[{target}]] in {source}")
        names = ", ".join(str(p.relative_to(docs_dir)) for p in candidates)
        raise ValueError(f"ambiguous Obsidian link [[{target}]] in {source}: {names}")
    return candidates[0]


def _convert_line(line: str, source: Path, docs_dir: Path, pages: dict[str, list[Path]]) -> str:
    code_spans: list[str] = []

    def protect(match: re.Match[str]) -> str:
        code_spans.append(match.group(0))
        return f"\x00CODE{len(code_spans) - 1}\x00"

    protected = re.sub(r"`[^`]*`", protect, line)

    protected = CURRENT_HEADING.sub(
        lambda match: f"[{match.group(2) or match.group(1)}](#{_heading_anchor(match.group(1))})",
        protected,
    )

    def replace(match: re.Match[str]) -> str:
        target, anchor, label = match.groups()
        resolved = _resolve(target, source, docs_dir, pages)
        href = Path(os.path.relpath(resolved, source.parent)).as_posix()
        if anchor:
            href += f"#{_heading_anchor(anchor[1:])}"
        return f"[{label or resolved.stem}]({href})"

    converted = WIKILINK.sub(replace, protected)
    return re.sub(r"\x00CODE(\d+)\x00", lambda m: code_spans[int(m.group(1))], converted)


def convert_markdown(markdown: str, *, source: Path, docs_dir: Path) -> str:
    pages = _pages(docs_dir)
    output: list[str] = []
    fenced = False
    for line in markdown.splitlines(keepends=True):
        if FENCE.match(line):
            fenced = not fenced
        output.append(line if fenced else _convert_line(line, source, docs_dir, pages))
    return "".join(output)


def on_page_markdown(markdown: str, *, page: Any, config: Any, files: Any) -> str:
    docs_dir = Path(config.docs_dir)
    source = docs_dir / page.file.src_uri
    return convert_markdown(markdown, source=source, docs_dir=docs_dir)
