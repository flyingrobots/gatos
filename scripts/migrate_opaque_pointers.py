#!/usr/bin/env python3
"""
Migrate OpaquePointer.cipher_meta -> encrypted_meta in JSON files.

Usage:
  python scripts/migrate_opaque_pointers.py <path1> [<path2> ...]

The script updates files in place. It is idempotent.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


def migrate_pointer(obj: Any) -> None:
    if isinstance(obj, dict):
        # Heuristic: migrate only when the object looks like an opaque pointer
        # or clearly contains the legacy key. We do not enforce a full schema here.
        if "cipher_meta" in obj and "encrypted_meta" not in obj:
            obj["encrypted_meta"] = obj.pop("cipher_meta")
        for v in obj.values():
            migrate_pointer(v)
    elif isinstance(obj, list):
        for item in obj:
            migrate_pointer(item)


def migrate_file(path: Path) -> bool:
    before = path.read_text(encoding="utf-8")
    data = json.loads(before)
    migrate_pointer(data)
    after = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    if after != before:
        path.write_text(after, encoding="utf-8")
        return True
    return False


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__.strip())
        return 2
    changed_any = False
    for arg in argv[1:]:
        p = Path(arg)
        if not p.exists():
            print(f"warning: {p} not found", file=sys.stderr)
            continue
        if p.is_dir():
            for f in p.rglob("*.json"):
                changed = migrate_file(f)
                changed_any = changed_any or changed
                if changed:
                    print(f"migrated: {f}")
        else:
            changed = migrate_file(p)
            changed_any = changed_any or changed
            if changed:
                print(f"migrated: {p}")
    return 0 if changed_any else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

