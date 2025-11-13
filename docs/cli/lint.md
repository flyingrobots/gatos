---
title: git gatos lint — Deterministic Lua Linter
---

# git gatos lint — Deterministic Lua Linter

Static lints to enforce the EchoLua deterministic profile.

## Synopsis

```bash
git gatos lint folds/**/*.lua
```

## Forbidden patterns (fail)

- `pairs`, `__pairs`, `coroutine.*`, `__gc`
- `math.random`, `os.*`, `io.*`, `debug.*`, `package.*`
- Non‑canonical numeric literals; reliance on unspecified table iteration order

## Warnings (configurable)

- Using RNG in folds (recommend jobs instead)
- Large constant tables without dsort/dpairs use

See: docs/deterministic-lua.md for full profile.

