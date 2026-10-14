#!/usr/bin/env python3
import os
import re
import sys

# Regex to find Obsidian wiki links: [[page-name]] or [[page-name|Display Text]]
WIKI_LINK_RE = re.compile(r'\[\[([^\]|]+)(?:\|[^\]]+)?\]\]')

# Sections a page of a given type must have, with non-trivial content under
# each heading (not just the heading immediately followed by another heading
# or EOF). Catches a page that technically follows the template but never
# actually did the analysis.
REQUIRED_SECTIONS = {
    "bug": ["Root cause", "Prevention"],
    "feature": ["Motivation"],
    "decision": ["Decision", "Rationale"],
}


def get_section_body(content, heading):
    """Return the text of a '## <heading>' section, or None if the heading is absent."""
    pattern = re.compile(
        rf'^##\s+{re.escape(heading)}\s*$(.*?)(?=^##\s|\Z)',
        re.MULTILINE | re.DOTALL,
    )
    match = pattern.search(content)
    if not match:
        return None
    return match.group(1).strip()

def parse_simple_yaml(yaml_text):
    """
    Parses a simple key-value YAML block without third-party libraries.
    Handles basic string values and arrays like tags: [a, b].
    """
    data = {}
    for line in yaml_text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if ":" in line:
            key, val = line.split(":", 1)
            key = key.strip()
            val = val.strip()
            # Strip quotes
            if val.startswith('"') and val.endswith('"'):
                val = val[1:-1]
            elif val.startswith("'") and val.endswith("'"):
                val = val[1:-1]
            # Handle simple bracket arrays [tag1, tag2]
            elif val.startswith("[") and val.endswith("]"):
                val = [item.strip().strip('"').strip("'") for item in val[1:-1].split(",") if item.strip()]
            data[key] = val
    return data

def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    wiki_dir = os.path.join(repo_root, "wiki")

    if not os.path.exists(wiki_dir):
        print(f"Error: wiki directory not found at {wiki_dir}")
        sys.exit(1)

    print("=== OKF Wiki Validation ===")

    # 1. Build a map of all valid target pages (slug -> absolute path)
    valid_pages = {}
    for root, _, files in os.walk(wiki_dir):
        for file in files:
            if file.endswith(".md"):
                file_path = os.path.join(root, file)
                slug = os.path.splitext(file)[0]

                # Directly map slug to path
                valid_pages[slug] = file_path

                # If it's an index.md in a subfolder, also allow referencing it
                # by the subfolder name (e.g. [[conventions]] maps to conventions/index.md)
                if file == "index.md":
                    subfolder = os.path.basename(root)
                    if subfolder and subfolder != "wiki":
                        valid_pages[subfolder] = file_path
                        valid_pages[f"{subfolder}/index"] = file_path

    errors = 0
    warnings = 0

    # 2. Validate each markdown file
    for root, _, files in os.walk(wiki_dir):
        for file in files:
            if not file.endswith(".md"):
                continue

            file_path = os.path.join(root, file)
            rel_path = os.path.relpath(file_path, repo_root)

            with open(file_path, "r", encoding="utf-8") as f:
                content = f.read()

            # Frontmatter check
            frontmatter = {}
            if content.startswith("---"):
                parts = content.split("---", 2)
                if len(parts) >= 3:
                    yaml_content = parts[1]
                    try:
                        frontmatter = parse_simple_yaml(yaml_content)
                    except Exception as e:
                        print(f"ERROR [{rel_path}]: Failed to parse frontmatter: {e}")
                        errors += 1
                else:
                    print(f"ERROR [{rel_path}]: Malformed frontmatter block (missing closing '---')")
                    errors += 1
            else:
                print(f"ERROR [{rel_path}]: Missing YAML frontmatter block at the start")
                errors += 1

            # Required field check
            if frontmatter:
                if "type" not in frontmatter:
                    print(f"ERROR [{rel_path}]: Required OKF field 'type' is missing in frontmatter")
                    errors += 1
                if "description" not in frontmatter:
                    print(f"WARNING [{rel_path}]: Recommended OKF field 'description' is missing in frontmatter")
                    warnings += 1

                page_type = frontmatter.get("type")
                for heading in REQUIRED_SECTIONS.get(page_type, []):
                    body = get_section_body(content, heading)
                    if body is None:
                        print(f"WARNING [{rel_path}]: type '{page_type}' page is missing expected '## {heading}' section")
                        warnings += 1
                    elif len(body) < 40:
                        print(f"ERROR [{rel_path}]: '## {heading}' section is empty or trivially short ({len(body)} chars) — analysis appears missing")
                        errors += 1

            # Link check
            links = WIKI_LINK_RE.findall(content)
            for link in links:
                # Strip spaces or hash anchors (e.g. [[page-name#section]])
                clean_link = link.split('#')[0].strip()

                # Exclude self-references or empty targets
                if not clean_link or clean_link == os.path.splitext(file)[0]:
                    continue

                if clean_link not in valid_pages:
                    print(f"ERROR [{rel_path}]: Broken wiki link [[{link}]] (page not found)")
                    errors += 1

    print("\n=== Validation Summary ===")
    print(f"Total Errors: {errors}")
    print(f"Total Warnings: {warnings}")

    if errors > 0:
        print("Result: FAILED. Please fix the errors before committing.")
        sys.exit(1)
    else:
        print("Result: PASSED. Wiki structure is OKF-compliant.")
        sys.exit(0)

if __name__ == "__main__":
    main()
