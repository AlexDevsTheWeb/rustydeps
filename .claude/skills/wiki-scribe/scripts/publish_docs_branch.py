#!/usr/bin/env python3
"""Publish converted wiki to a docs branch via GitLab API single commit."""

from __future__ import annotations

import base64
import os
import shutil
import sys
from pathlib import Path

import urllib.request
import urllib.parse
import json

GITLAB_URL = os.environ.get("CI_SERVER_URL", "https://gitlab.example.com")
PROJECT_ID = os.environ.get("CI_PROJECT_ID")
TOKEN = os.environ.get("GITLAB_TOKEN")
BRANCH = "docs"


def api(method: str, path: str, data: dict | None = None) -> dict:
    url = f"{GITLAB_URL}/api/v4{path}"
    body = json.dumps(data).encode() if data else None
    req = urllib.request.Request(url, data=body, method=method)
    req.add_header("PRIVATE-TOKEN", TOKEN)
    if body:
        req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        error_body = e.read().decode()
        print(f"API error {e.code}: {error_body}", file=sys.stderr)
        raise


def main() -> int:
    if not all([GITLAB_URL, PROJECT_ID, TOKEN]):
        print("Error: Missing CI_SERVER_URL, CI_PROJECT_ID, or GITLAB_TOKEN env vars", file=sys.stderr)
        return 1

    root = Path(__file__).resolve().parents[1]
    source_dir = root / "wiki"

    # Build the converted docs locally
    output = Path("/tmp/techdocs-build")
    if output.exists():
        shutil.rmtree(output)
    docs_dir = output / "docs"
    docs_dir.mkdir(parents=True)

    from obsidian_links import convert_markdown

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

    (output / "mkdocs.yml").write_text(
        "site_name: Project Wiki\n"
        "site_description: Architecture and engineering knowledge base.\n"
        "docs_dir: docs\n"
        "plugins:\n"
        "  - techdocs-core\n",
        encoding="utf-8",
    )

    # Ensure branch exists
    try:
        api("GET", f"/projects/{PROJECT_ID}/repository/branches/{BRANCH}")
    except urllib.error.HTTPError:
        api("POST", f"/projects/{PROJECT_ID}/repository/branches",
            {"branch": BRANCH, "ref": "develop"})

    # Get existing files on docs branch
    existing: set[str] = set()
    page = 1
    while True:
        files = api("GET", f"/projects/{PROJECT_ID}/repository/tree?ref={BRANCH}&recursive=true&per_page=100&page={page}")
        if not files:
            break
        for f in files:
            if f["type"] == "blob":
                existing.add(f["path"])
        page += 1

    # Collect all files as actions
    actions = []
    for f in sorted(output.rglob("*")):
        if f.is_file():
            rel = str(f.relative_to(output))
            content = f.read_text(encoding="utf-8")
            action = "update" if rel in existing else "create"
            actions.append({
                "action": action,
                "file_path": rel,
                "content": content,
            })

    # Batch commits (GitLab API limit ~100 actions per commit, keep smaller for safety)
    BATCH = 50
    for i in range(0, len(actions), BATCH):
        batch = actions[i:i + BATCH]
        print(f"Committing batch {i // BATCH + 1} ({len(batch)} files)")
        api("POST", f"/projects/{PROJECT_ID}/repository/commits",
            {
                "branch": BRANCH,
                "commit_message": f"Update TechDocs batch {i // BATCH + 1}" if i > 0 else "Update TechDocs (converted wikilinks)",
                "actions": batch,
            })

    print(f"Published to {BRANCH} branch")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
