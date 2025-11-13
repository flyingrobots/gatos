#!/usr/bin/env python3
"""
Linkify bare references like "SPEC §5" or "TECH-SPEC §7.1" in Markdown files
to anchored links, e.g.:

  SPEC §5         -> [SPEC §5](/SPEC#5)
  TECH-SPEC §7.1  -> [TECH-SPEC §7.1](/TECH-SPEC#7.1)

Usage:
  python scripts/linkify_refs.py --write [--paths docs]
  python scripts/linkify_refs.py --check
"""
import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

REF_RE = re.compile(r"\b(SPEC|TECH-SPEC)\s*§\s*([0-9]+(?:\.[0-9]+)*)\b")

def linkify(text: str) -> tuple[bool, str]:
    changed = False
    def repl(m: re.Match) -> str:
        nonlocal changed
        doc = m.group(1)
        sec = m.group(2)
        # Use absolute site-root links so paths work from nested pages in VitePress.
        # VitePress will prepend the configured base (e.g., /gatos/) on deploy.
        changed = True
        target = "/SPEC" if doc == "SPEC" else "/TECH-SPEC"
        return f"[{doc} §{sec}]({target}#{sec})"
    out = REF_RE.sub(repl, text)
    return changed, out

def process_file(path: pathlib.Path) -> tuple[bool, str]:
    text = path.read_text(encoding="utf-8")
    # Avoid code fences
    parts = re.split(r"(^```.*$)", text, flags=re.M)
    out_parts = []
    changed_any = False
    in_code = False
    for part in parts:
        if part.startswith("```"):
            in_code = not in_code
            out_parts.append(part)
            continue
        if in_code:
            out_parts.append(part)
            continue
        chg, out = linkify(part)
        if chg:
            changed_any = True
        out_parts.append(out)
    return changed_any, "".join(out_parts)

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--paths", nargs="*", default=["docs"])
    args = ap.parse_args()

    targets = [ROOT / p for p in args.paths]
    changed_any = False
    for t in targets:
        if t.is_file() and t.suffix == ".md":
            mds = [t]
        else:
            mds = list(t.rglob("*.md"))
        for md in mds:
            chg, out = process_file(md)
            if chg:
                changed_any = True
                if args.write:
                    md.write_text(out, encoding="utf-8")
                print(f"would change: {md}")

    if args.check and changed_any:
        return 2
    return 0

if __name__ == "__main__":
    sys.exit(main())
