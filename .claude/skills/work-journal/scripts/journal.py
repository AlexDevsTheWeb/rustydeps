#!/usr/bin/env python3
"""Create and inspect a task work journal under <repository>/tmp/<slug>/.

Standard library only. Run it from inside the target repository.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

BUNDLE_ROOT = Path(__file__).resolve().parents[1]
TEMPLATES_DIR = BUNDLE_ROOT / "templates"
JOURNAL_FILES = ("handoff.md", "memory.md", "log.md")
SLUG_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,99}")
BRANCH_LINE = re.compile(r"^- Branch: (\S+)", re.MULTILINE)
HEAD_LINE = re.compile(r"^- Last verified HEAD: (\S+)", re.MULTILINE)


class JournalError(Exception):
    """A condition the caller must resolve before the journal can be used."""


def run_git(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=False)


def repository_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise JournalError("not inside a git repository; run this from the target repository")
    return Path(result.stdout.strip())


def journal_dir(root: Path, slug: str) -> Path:
    if not SLUG_PATTERN.fullmatch(slug):
        raise JournalError(f"invalid slug '{slug}': use letters, digits, '.', '_', or '-' only")
    return root / "tmp" / slug


def read_field(text: str, pattern: re.Pattern[str]) -> str | None:
    """Return the recorded value, or None when the template placeholder is still unfilled."""
    match = pattern.search(text)
    if match is None or match.group(1).startswith("<"):
        return None
    return match.group(1)


def init_journal(slug: str) -> int:
    root = repository_root()
    if run_git(["check-ignore", "-q", "tmp/"], root).returncode != 0:
        raise JournalError("tmp/ is not git-ignored in this repository; add 'tmp/' to .gitignore first")

    target = journal_dir(root, slug)
    status = "resumed" if target.is_dir() else "created"
    target.mkdir(parents=True, exist_ok=True)

    new_files = []
    for name in JOURNAL_FILES:
        destination = target / name
        if not destination.exists():
            shutil.copyfile(TEMPLATES_DIR / name, destination)
            new_files.append(name)

    print(f"journal: {target}")
    print(f"status: {status}")
    print(f"new files: {', '.join(new_files) if new_files else 'none'}")
    return 0


def show_state(slug: str) -> int:
    root = repository_root()
    target = journal_dir(root, slug)
    handoff = target / "handoff.md"
    if not handoff.is_file():
        raise JournalError(f"no journal at {target}; run 'init {slug}' first")

    text = handoff.read_text(encoding="utf-8")
    recorded_branch = read_field(text, BRANCH_LINE)
    recorded_head = read_field(text, HEAD_LINE)

    current_branch = run_git(["rev-parse", "--abbrev-ref", "HEAD"], root).stdout.strip() or "none"
    current_head = run_git(["rev-parse", "HEAD"], root).stdout.strip() or "none"
    changed_paths = run_git(["status", "--porcelain"], root).stdout.splitlines()

    commits_since = "unknown: no verified HEAD recorded"
    if recorded_head:
        known = run_git(["cat-file", "-e", f"{recorded_head}^{{commit}}"], root).returncode == 0
        if known:
            commits_since = run_git(["rev-list", "--count", f"{recorded_head}..HEAD"], root).stdout.strip()
        else:
            commits_since = f"unknown: recorded HEAD {recorded_head} is not in this repository"

    branch_matches = recorded_branch == current_branch
    head_matches = recorded_head is not None and current_head.startswith(recorded_head)

    print(f"journal: {target}")
    print(f"recorded branch: {recorded_branch or 'not recorded'}")
    print(f"current branch: {current_branch}")
    print(f"recorded HEAD: {recorded_head or 'not recorded'}")
    print(f"current HEAD: {current_head}")
    print(f"commits since recorded HEAD: {commits_since}")
    print(f"working tree: {'clean' if not changed_paths else f'{len(changed_paths)} changed path(s)'}")
    if not (branch_matches and head_matches):
        print("reconcile: the journal does not match git state; read the commits and changes before acting")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    subcommands = parser.add_subparsers(dest="command", required=True)

    init_parser = subcommands.add_parser("init", help="create the journal from the templates")
    init_parser.add_argument("slug")

    state_parser = subcommands.add_parser("state", help="compare the journal with git state")
    state_parser.add_argument("slug")

    args = parser.parse_args(argv)
    try:
        if args.command == "init":
            return init_journal(args.slug)
        return show_state(args.slug)
    except JournalError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
