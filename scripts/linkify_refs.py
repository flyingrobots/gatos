#!/usr/bin/env python3
import argparse, pathlib, re, sys
ROOT = pathlib.Path(__file__).resolve().parents[1]
REF_RE = re.compile(r"\b(SPEC|TECH-SPEC)\s*§\s*([0-9]+(?:\.[0-9]+)*)\b")

def linkify(text):
    return REF_RE.sub(lambda m: f"[{m.group(1)} §{m.group(2)}](./{'SPEC.md' if m.group(1)=='SPEC' else 'TECH-SPEC.md'}#{m.group(2)})", text)

def process(path):
    t = path.read_text(encoding='utf-8')
    parts = re.split(r"(^```.*$)", t, flags=re.M)
    out=[]; in_code=False; changed=False
    for p in parts:
        if p.startswith('```'): in_code=not in_code; out.append(p); continue
        if in_code: out.append(p); continue
        new = linkify(p); changed |= new != p; out.append(new)
    if changed: path.write_text(''.join(out), encoding='utf-8')

if __name__=='__main__':
    for md in (ROOT/'docs').rglob('*.md'):
        process(md)
