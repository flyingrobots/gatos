# Hello, GATOS — Operations Path

This quick, end‑to‑end walkthrough gets you from an empty repo to a folded state and an attested job result. It assumes you have `git` and the `git gatos` CLI installed, and (optionally) a local Stargate.

## 0. Initialize and Configure Remotes

```bash
mkdir hello-gatos && cd hello-gatos
git init
git gatos init

# Optional: Dual remotes for read (GitHub) and write (Stargate)
git remote add origin git@github.com:org/repo.git
git remote add stargate ssh://git@stargate.local/org/repo.git
```

## 1. Append a Governed Event

Append an event; the Policy Gate will decide ALLOW/DENY.

```bash
git gatos event add --ns demo --type demo.hello --payload '{"msg":"hello gatos"}'
```

If denied, you’ll see a deterministic error frame; the denial is recorded under `refs/gatos/audit/policy`:

```json
{"ok":false, "code":"POLICY_DENY", "reason":"missing cap: demo:append"}
```

Grant a capability (example governance policy) and try again, or continue if allowed.

## 2. Fold to State and Inspect

```bash
git gatos fold --ns demo
```

This writes a checkpoint under `refs/gatos/state/demo`. Verify trailers on the checkpoint commit:

```
State-Root: blake3:<hex>
Ledger-Head: <commit-oid>
Policy-Root: <commit-oid>
Fold-Engine: echo@<semver>
Fold-Version: <schema-version>
```

Show the materialized state (example):

```bash
git show refs/gatos/state/demo:state/demo.json | jq .
```

## 3. Enqueue a Job and Observe PoE

```bash
git gatos jobs enqueue --ns demo \
  --command '["/usr/bin/env","bash","-lc"]' \
  --args    '["echo","ok"]' \
  --timeout 30

# (Optional) Subscribe to the bus to watch for job messages
git gatos bus subscribe --topic gatos.jobs.pending
```

When the worker completes, GATOS records a result commit under `refs/gatos/jobs/<job-id>/result` with trailers:

```
Job-Id: blake3:<hex>
Proof-Of-Execution: blake3:<hex>
Worker-Id: ed25519:<pubkey>
```

The PoE envelope proves provenance/authenticity (who ran what, where, on which inputs). Query it:

```bash
git show refs/gatos/jobs/<job-id>/result:poe.json | jq .
```

## 4. Push via Stargate (Optional)

```bash
git push stargate --all
```

The Stargate enforces pre‑receive policy and mirrors to `origin` with `--prune`. Fetch from `origin` for reads.

---

Tips
- Determinism: same ledger + same `policy_root` ⇒ same `state_root`.
- At‑least‑once: the bus delivers at‑least‑once; use message ULIDs for idempotency.
