#!/usr/bin/env python3
import argparse, pathlib, re, sys
ROOT = pathlib.Path(__file__).resolve().parents[1]
GUIDE = ROOT / 'docs' / 'guide'
CHAPTER_RE = re.compile(r"\b[Cc]hapter\s+([1-9]|1[0-2])\b")

def process(path):
    t = path.read_text(encoding='utf-8')
    def repl(m):
        n = int(m.group(1))
        return f"[Chapter {n}](./CHAPTER-{n:03d}.md)"
    parts = re.split(r"(^```.*$)", t, flags=re.M)
    out=[]; in_code=False; changed=False
    for p in parts:
        if p.startswith('```'): in_code=not in_code; out.append(p); continue
        if in_code: out.append(p); continue
        new = CHAPTER_RE.sub(repl, p)
        changed |= new != p
        out.append(new)
    if changed:
        path.write_text(''.join(out), encoding='utf-8')

if __name__=='__main__':
    for md in GUIDE.glob('*.md'):
        process(md)
