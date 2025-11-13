---
title: git gatos foldc — Compile EchoLua to ELC
---

# git gatos foldc — Compile EchoLua to ELC

Compiles EchoLua source (`.lua`) into Echo Lua IR (ELC) bytes serialized as DAG‑CBOR.

## Synopsis

```bash
git gatos foldc <src.lua> -o <out.elc>
```

## Behavior

- Parses + normalizes Lua, lowers to EchoLua IR, serializes to DAG‑CBOR.
- Emits the engine id in diagnostics: `echo@<semver>+elc@<semver>+num=q32.32+rng=pcg32@<ver>`.
- Prints `fold_root = sha256:<hex>` on success.

## Notes

- EchoLua follows the Deterministic Lua profile (see docs/deterministic-lua.md).
- Forbidden constructs cause compilation failure (see `git gatos lint`).

