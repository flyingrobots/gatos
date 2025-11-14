#!/usr/bin/env python3
"""
Linkify bare "Chapter N" mentions in docs/guide/*.md to point at the corresponding
chapter files, e.g.:

  "... see Chapter 3." -> "... see [Chapter 3](./CHAPTER-003.md)."

Heuristics:
- Skips code fences and Markdown headings.
- Avoids touching lines that already link to CHAPTER-*.md.
- Only links patterns of the form /\b[Cc]hapter\s+([1-9]|1[0-2])\b/.

Usage:
  python scripts/linkify_chapters.py --check                 # exits non-zero if changes needed
  python scripts/linkify_chapters.py --write                 # applies changes in-place (docs/guide)
  python scripts/linkify_chapters.py --write --paths docs/guide docs/HELLO-OPS.md  # explicit targets
"""
import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
GUIDE_DIR = ROOT / "docs" / "guide"

def chapter_file(n: int) -> str:
    return f"CHAPTER-{int(n):03d}.md"

CHAPTER_RE = re.compile(r"\b[Cc]hapter\s+([1-9]|1[0-2])\b")

def should_skip_line(line: str) -> bool:
    # Skip if already links to a chapter
    if "](./CHAPTER-" in line or "](CHAPTER-" in line:
        return True
    # Skip headings
    if line.lstrip().startswith("#"):
        return True
    return False

def linkify_line(line: str) -> str:
    # Avoid lines that already have chapter links
    if should_skip_line(line):
        return line

    def repl(m: re.Match) -> str:
        num = int(m.group(1))
        target = chapter_file(num)
        # Avoid replacements inside existing Markdown links
        start, end = m.span()
        left = line.rfind('[', 0, start)
        right = line.find(')', start)
        close = line.find(']', 0, start)
        # If we have a '[' before and a ']' after the start and then a '(' soon after ']', assume link context
        if left != -1 and close != -1 and close > left:
            paren = line.find('(', close)
            if paren != -1 and paren < end:
                return m.group(0)
        return f"[Chapter {num}](./{target})"

    return CHAPTER_RE.sub(repl, line)

def process_file(path: pathlib.Path) -> tuple[bool, str]:
    text = path.read_text(encoding="utf-8")
    out_lines = []
    changed = False
    in_code = False
    for line in text.splitlines(keepends=True):
        if line.strip().startswith("```"):
            in_code = not in_code
            out_lines.append(line)
            continue
        if in_code:
            out_lines.append(line)
            continue
        new_line = linkify_line(line)
        if new_line != line:
            changed = True
        out_lines.append(new_line)
    return changed, "".join(out_lines)

def iter_markdown(paths: list[pathlib.Path]):
    for p in paths:
        if p.is_file() and p.suffix == ".md":
            yield p
        elif p.is_dir():
            for md in p.rglob("*.md"):
                yield md

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true", help="apply changes in-place")
    ap.add_argument("--check", action="store_true", help="exit non-zero if changes required")
    ap.add_argument("--paths", nargs="*", default=[str(GUIDE_DIR)], help="files/dirs to process")
    args = ap.parse_args()

    changed_any = False
    targets = [pathlib.Path(p) for p in args.paths]
    for md in iter_markdown(targets):
        chg, new_text = process_file(md)
        if chg:
            changed_any = True
            if args.write:
                md.write_text(new_text, encoding="utf-8")
            print(f"would change: {md}")

    if args.check and changed_any:
        return 2
    return 0

if __name__ == "__main__":
    sys.exit(main())
