# gatos watch / lock CLI

This is a planning stub for ADR-0006. The implementation is in progress; this doc captures the intent so downstream teams can plan integrations.

## `gatos watch`

```
git gatos watch [--once] [--state <path>]
```

- Starts the local enforcement daemon. Observes the working tree plus `.git/refs/gatos/**`.
- By default runs continuously; `--once` performs a single scan (useful in CI or troubleshooting).
- Emits structured JSONL events to stdout (see `schemas/v1/watch/events.schema.json`). Use `--state` to override the default state directory (`~/.config/gatos/watch`).

## `gatos lock acquire`

```
git gatos lock acquire <path...> [--reason <text>] [--no-wait]
```

- Computes canonical lock ids for each path/glob.
- Creates governance proposals referencing the rule declared in `.gatos/policy.yaml`.
- Waits for Grants unless `--no-wait` is provided. The output lists `path`, `proposal`, `grant`, and `status` columns.

## `gatos lock release`

```
git gatos lock release <path...> [--reason <text>]
```

- Revokes or supersedes existing grants, making the files writable again once the watcher processes the change.

## `gatos install-hooks`

```
git gatos install-hooks [--force]
```

- Writes managed `pre-commit`, `pre-push`, and `post-checkout`/`post-merge` hooks.
- Hooks call back into `gatos hook run <name>` so logic stays centralized.
- `--force` reinstalls even if hooks already exist.

> **Status:** CLI surface is being implemented. This document is intentionally aspirational so docs/spec stay aligned with ADR-0006.
