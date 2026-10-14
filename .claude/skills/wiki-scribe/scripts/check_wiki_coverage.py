#!/usr/bin/env python3
"""
Cross-repo wiki coverage check.

Looks at merge commits landed on development branch and flags
any that reference a ticket or a fix/feat commit scope with no
matching mention anywhere under wiki/. Two intended callers:

  1. A CI job, run on every push to development, checking only
     the commits introduced by that push (--since-ref) — catches an
     undocumented merge immediately instead of weeks later.
  2. A periodic/manual freshness sweep across a longer window (--since-days),
     run from either repo, to catch anything job #1 missed or that predates it.

Exit code 0 = no gaps found. Exit code 1 = gaps found. Callers decide whether
a non-zero exit should fail their pipeline (see --fail-on-gap) or just be
reported — the CI job for case 1 is expected to warn, not block merges.
"""
import argparse
import os
import re
import subprocess
import sys

TICKET_RE = re.compile(r"[A-Z]+-\d+", re.IGNORECASE)
SCOPE_RE = re.compile(r"^(?:feat|fix|hotfix)\(([^)]+)\):", re.IGNORECASE)


def run_git(repo, *args):
    result = subprocess.run(
        ["git", "-C", repo, *args],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def get_commits(code_repo, since_ref=None, since_days=None, branch="development"):
    fmt = "%H%x1f%ad%x1f%s"
    args = ["log", branch, "--no-merges", f"--pretty=format:{fmt}", "--date=short"]
    if since_ref:
        args = ["log", f"{since_ref}..{branch}", "--no-merges", f"--pretty=format:{fmt}", "--date=short"]
    elif since_days:
        args += [f"--since={since_days} days ago"]

    out = run_git(code_repo, *args)
    commits = []
    for line in out.splitlines():
        if not line.strip():
            continue
        sha, date, subject = line.split("\x1f", 2)
        commits.append({"sha": sha, "date": date, "subject": subject})
    return commits


def extract_keys(subject):
    """Return the set of lookup keys (ticket ids and/or commit scopes) for a commit subject."""
    keys = set(m.group(0).upper() for m in TICKET_RE.finditer(subject))
    scope_match = SCOPE_RE.match(subject.strip())
    if scope_match:
        scope = scope_match.group(1).strip()
        if not TICKET_RE.fullmatch(scope):
            keys.add(scope)
    return keys


def build_wiki_haystack(wiki_dir):
    """Concatenate every wiki page's path + content into one lowercase blob to search against."""
    chunks = []
    for root, _, files in os.walk(wiki_dir):
        for f in files:
            if f.endswith(".md"):
                path = os.path.join(root, f)
                with open(path, "r", encoding="utf-8") as fh:
                    chunks.append(f.lower() + "\n" + fh.read().lower())
    return "\n".join(chunks)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--code-repo", required=True, help="Path to the code repository checkout")
    parser.add_argument("--wiki-repo", default=None, help="Path to wiki repo checkout (default: this script's repo root)")
    parser.add_argument("--branch", default="development", help="Code repo branch to scan (default: development)")
    parser.add_argument("--since-ref", default=None, help="Only commits after this git ref (e.g. previous pipeline's SHA)")
    parser.add_argument("--since-days", type=int, default=None, help="Only commits from the last N days")
    parser.add_argument("--fail-on-gap", action="store_true", help="Exit 1 if any gap is found (default: report only, exit 0)")
    args = parser.parse_args()

    wiki_repo = args.wiki_repo or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    wiki_dir = os.path.join(wiki_repo, "wiki")

    if not os.path.isdir(args.code_repo):
        print(f"error: code repo path not found: {args.code_repo}", file=sys.stderr)
        sys.exit(2)
    if not os.path.isdir(wiki_dir):
        print(f"error: wiki dir not found: {wiki_dir}", file=sys.stderr)
        sys.exit(2)

    commits = get_commits(args.code_repo, since_ref=args.since_ref, since_days=args.since_days, branch=args.branch)
    haystack = build_wiki_haystack(wiki_dir)

    gaps = []
    for c in commits:
        keys = extract_keys(c["subject"])
        if not keys:
            continue  # nothing to look up (e.g. "chore: bump deps") — not this tool's job
        if any(key.lower() in haystack for key in keys):
            continue
        gaps.append({**c, "keys": sorted(keys)})

    print("=== Wiki Coverage Check ===")
    print(f"Code repo branch: {args.branch} | commits scanned: {len(commits)}")

    if not gaps:
        print("Result: PASSED. No undocumented ticket/scope commits found in range.")
        sys.exit(0)

    print(f"\n{len(gaps)} commit(s) with no matching wiki page:\n")
    for g in gaps:
        print(f"  {g['sha'][:7]}  {g['date']}  {', '.join(g['keys']):<20}  {g['subject']}")

    print(f"\nResult: {'FAILED' if args.fail_on_gap else 'GAPS FOUND'}. Ingest these into wiki/bugs or wiki/features.")
    sys.exit(1 if args.fail_on_gap else 0)


if __name__ == "__main__":
    main()
