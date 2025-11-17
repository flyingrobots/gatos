---
Status: Proposed
Date: 2025-11-09
ADR: ADR-0006
Authors: [flyingrobots]
Requires: [ADR-0003]
Related: [ADR-0002, ADR-0004]
Tags: [Watcher, Hooks, Locks]
Schemas:
  - ../../../../schemas/v1/policy/locks.schema.json   # TBD schema capturing `.gatos/policy.yaml` locks/watch config
  - ../../../../schemas/v1/watch/events.schema.json   # Structured event payloads emitted by the watcher (defined in implementation ADR)
Supersedes: []
Superseded-By: []
---

# ADR-0006: Local Enforcement — Watcher Daemon & Git Hooks

## Scope

Provide **local enforcement** of governance policy via (a) a cross-platform Watcher daemon (`gatos watch`) and (b) Git hooks installed by the CLI. The goal is to mirror the guarantees of ADR-0003 (Consensus Governance) on every workstation: Perforce-style read-only locks until a Grant exists, pre-commit/pre-push policy gates, and reactive automation tied to the Job Plane.

## Rationale

- **Problem:** Without local enforcement, contributors can edit locked assets, forget to acquire Grants, or push non-compliant history—only to be rejected later by server-side policy.
- **Context:** Artists and developers expect “read-only until lock” workflows, automatic tests on save/fold change, and immediate feedback instead of slow CI rebukes.
- **Outcome:** A consistent, deterministic local experience that reflects policy reality before changes ever leave the workstation.

## Decision

### 1. Watcher Daemon (`gatos watch`)

- Monitors the working tree plus `.git/refs/gatos/**` using cross-platform file notifications (inotify/FSEvents/ReadDirectoryChangesW). When unavailable, the daemon MUST fall back to polling.
- Enforces **read-only masks** for paths matched in `.gatos/policy.yaml` `locks` section until a valid **Grant** exists (ADR-0003). Default enforcement:
  - POSIX: `chmod -w`, escalating to `chflags uchg`/`chattr +i` when the user opts in.
  - Windows: set `FILE_ATTRIBUTE_READONLY`.
- Emits structured events (JSONL on stdout + desktop notification hooks) whenever policy denies a mutation attempt. Event schema (see `schemas/v1/watch/events.schema.json`):

```json
{ "ts": "2025-11-09T12:00:00Z", "rule": "governance.locks.assets", "actor": "user:alice", "path": "assets/hero.obj", "action": "deny.write", "remediation": "Acquire lock" }
```
- Watches `refs/gatos/grants/**` for updates so that newly approved locks are released immediately without requiring a restart.
- The daemon MUST persist state (e.g., last applied masks, pending lock requests) under `~/.config/gatos/watch/` to survive restarts. State files are advisory; corruption or tampering MUST trigger a full resync from Git policy data before enforcement resumes.

### 2. Git Hooks (managed surface)

`gatos install-hooks` installs managed hook scripts (POSIX shell + PowerShell). Hooks MUST be idempotent and re-runnable.

- `pre-commit`: rejects staged changes touching locked paths, consults the Watcher cache, and logs violations under `refs/gatos/audit/locks/<ulid>`.
- `pre-push`: verifies that every outbound reference has the required Grants (ADR-0003) and that Proof-of-Fold/Proof-of-Execution metadata (when mandated) is present. Failure MUST block the push.
- `post-merge` / `post-checkout`: re-apply read-only masks based on current grants.
- Hooks MUST fail closed if the policy engine cannot evaluate (missing cache, corrupt policy, etc.). Users can bypass only via the documented escape hatch (`GATOS_NO_HOOKS=1`), which emits a warning banner *and* records an audit trailer (`Bypass-Hooks: user:alice reason=env override`) on the next push so server-side policy can flag the session.

### 3. Lock Acquisition UX

- `gatos lock acquire <path>`:
  1. Computes the canonical lock id (path glob + repository root).
  2. Creates a **Proposal** under `refs/gatos/proposals/locks/<ulid>` referencing the governance rule declared in `.gatos/policy.yaml`.
  3. Waits (with progress feedback) for a **Grant** to materialize; once quorum is met, the Watcher daemon removes the read-only bit for the granted files.
- `gatos lock release <path>`: revokes or supersedes the Grant via ADR-0003’s revocation flow.
- CLI helpers MUST support batching (multiple paths). “Per-path best-effort” means the CLI issues one Proposal per path and continues processing remaining entries even if some fail; failures MUST be reported individually, and commands MUST exit non-zero if any path failed. A summary (“2/3 locks granted”) is shown and detailed status recorded under `~/.config/gatos/locks/`.

### 4. Reactive Automation

- Policies MAY declare `watcher.tasks[]` entries that run deterministic commands locally when a file is saved or when a fold finishes (e.g., run formatters, lint, or spawn a local Job Plane task in “loopback” mode per ADR-0002).
- Tasks run in a sandbox (`git worktree` or temp dir) and MUST publish their outputs as Job commits if the policy requires proof (e.g., `Proof-Of-Execution: local`). Implementations MUST enforce sane defaults: max concurrent tasks = 2, default timeout = 120s, configurable via `.gatos/policy.yaml`. Exceeding limits terminates the task and logs a warning.

### 5. Configuration (`.gatos/policy.yaml`)

```yaml
locks:
  - match: "assets/**"
    rule: "governance.locks.assets"   # ADR-0003 rule id
    read_only: true
  - match: "codegen/**"
    rule: "governance.locks.codegen"
watcher:
  poll_fallback_ms: 5000
  tasks:
    - when: "on_save"
      match: "**/*.proto"
      run_job: "format.proto"
```

- The `locks` array declares glob patterns and the governance rule controlling each.
- `watcher.tasks` describes optional automation hooks. Task definitions MUST reference existing Job Plane manifests when `run_job` is used.
- Users MAY opt out by setting `GATOS_NO_HOOKS=1` or `GATOS_NO_WATCH=1`, but the CLI MUST warn that doing so removes local guardrails. Opt-outs SHOULD be persisted to `refs/gatos/audit/locks/<ulid>` so reviewers know the session bypassed local enforcement.

## Consequences

**Pros**
- Prevents “foot-gun” edits to locked or policy-controlled files.
- Provides Perforce-style artist workflow inside Git + ADR governance.
- Gives immediate feedback (notifications + hook failures) instead of delayed CI surprises.

**Cons**
- Platform differences in file permissions; must handle FAT/NTFS quirks and network filesystems.
- Misconfiguration could temporarily lock users out; hooks must degrade gracefully when profiles change.
- Local enforcement can be bypassed; server-side policy remains the source of truth.

## Security Considerations

- Hooks and watcher run with user privileges and cannot be trusted for adversarial enforcement; the remote push-gate remains authoritative.
- The watcher MUST respect ADR-0004 privacy rules: never emit private overlay paths in logs/notifications unless the actor already has access, and avoid leaking pointer metadata.
- Daemon communication channels (e.g., JSONL socket) MUST authenticate local clients or restrict to loopback to prevent untrusted processes from spoofing policy events.
