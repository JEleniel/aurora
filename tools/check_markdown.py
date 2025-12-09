#!/usr/bin/env python3
"""Simple Markdown linter for this repo.

Checks a focused subset of rules from `.markdownlint.json`:
- first-line-heading
- no-trailing-spaces
- single-trailing-newline
- blanks-around-fences (blank line before/after fenced code blocks)
- fenced-code-language (code fence must include a language)
- blanks-around-lists (blank line before lists when following a paragraph)
- blanks-after-headings (one blank line after headings)
- no-multiple-blanks (no consecutive blank lines)

Run from repo root: `python3 tools/check_markdown.py`
Exits with code 0 when no violations, 1 otherwise.
"""
import sys
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"


def find_md_files(root: Path):
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def check_file(p: Path):
    text = p.read_text(encoding="utf-8")
    lines = text.splitlines()
    issues = []

    # first-line-heading: first non-empty line must be a heading (#...)
    first_nonblank = None
    for i, l in enumerate(lines):
        if l.strip() != "":
            first_nonblank = (i, l)
            break
    if first_nonblank:
        if not first_nonblank[1].lstrip().startswith("#"):
            issues.append((first_nonblank[0]+1, "first-line-heading: first non-empty line is not a heading"))

    # trailing spaces
    for i, l in enumerate(lines):
        if l.endswith(" "):
            issues.append((i+1, "no-trailing-spaces: line ends with a space"))

    # single trailing newline: file must end with exactly one '\n'
    if not text.endswith("\n"):
        issues.append((len(lines), "single-trailing-newline: file does not end with a newline"))
    else:
        if text.endswith("\n\n"):
            issues.append((len(lines), "single-trailing-newline: file ends with multiple blank lines"))

    # no multiple blank lines
    for i in range(len(lines)-1):
        if lines[i].strip() == "" and lines[i+1].strip() == "":
            issues.append((i+1, "no-multiple-blanks: consecutive blank lines"))

    # blanks-after-headings and fenced-code and lists checks
    in_fence = False
    fence_start_idx = None
    for i, l in enumerate(lines):
        s = l.rstrip()
        # detect code fence
        m = re.match(r'^(?P<fence>`{3,})(?P<lang>.*)$', s)
        if m:
            fence = m.group('fence')
            lang = m.group('lang').strip()
            if not in_fence:
                # opening fence: ensure blank line before (unless at file start)
                if i > 0 and lines[i-1].strip() != "":
                    issues.append((i, "blanks-around-fences: missing blank line before code fence"))
                # fenced-code-language: opening fence should have a language
                if lang == "":
                    issues.append((i+1, "fenced-code-language: code fence has no language identifier"))
                in_fence = True
                fence_start_idx = i
            else:
                # closing fence: ensure blank line after (unless EOF)
                if i+1 < len(lines) and lines[i+1].strip() != "":
                    issues.append((i+1, "blanks-around-fences: missing blank line after code fence"))
                in_fence = False
                fence_start_idx = None
            continue

        if in_fence:
            continue

        # blank after headings
        if s.lstrip().startswith('#'):
            # next line must be blank if exists
            if i+1 < len(lines) and lines[i+1].strip() != "":
                issues.append((i+2, "blanks-after-headings: expected blank line after heading"))

        # blanks-around-lists: when encountering a list start ensure previous non-blank
        # is a heading or there is a blank line separating
        if re.match(r'^\s*([-*+]\s+|\d+\.\s+)', s):
            # find previous line index
            prev_idx = i-1
            while prev_idx >= 0 and lines[prev_idx].strip() == "":
                prev_idx -= 1
            if prev_idx >= 0:
                prev = lines[prev_idx]
                if not prev.lstrip().startswith('#') and not re.match(r'^\s*([-*+]\s+|\d+\.\s+)', prev):
                    # ensure there is a blank line immediately before the list
                    if i-1 >= 0 and lines[i-1].strip() != "":
                        issues.append((i+1, "blanks-around-lists: expected blank line before list"))

    return issues


def main():
    md_files = find_md_files(DOCS)
    total_issues = 0
    for p in md_files:
        issues = check_file(p)
        if issues:
            print(f"{p}:")
            for ln, msg in issues:
                print(f"  L{ln}: {msg}")
            print("")
            total_issues += len(issues)

    if total_issues:
        print(f"Found {total_issues} markdownlint-style issues.")
        sys.exit(1)
    print("No lint issues found.")
    sys.exit(0)


if __name__ == '__main__':
    main()
