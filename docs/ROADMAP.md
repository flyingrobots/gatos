# GATOS — Roadmap (v0.3 → GA)

## Phase 0 — Echo Determinism Lock (Week 1)

- [ ] Cross-arch golden vectors green
- [ ]Canonical serializer frozen

## Phase 1 — Core Ledger + Policy (Weeks 2–3)

- [ ] Journals (FF-only), 
- [ ] CAS, 
- [ ] policy VM (pure), 
- [ ] audit decisions
- [ ] Tests: 
  - [ ] F1, 
  - [ ] F2, 
  - [ ] F3

## Phase 2 — Sessions + Bus QoS (Weeks 4–5)

- [ ] session start/undo/fork/merge
- [ ] mbus QoS (`at_least_once`, `exactly_once`), 
- [ ] acks/commitments
- [ ] Tests: F5

## Phase 3 — Epochs + Doctor + Observability (Week 6)

- [ ] epoch anchors, 
- [ ] compactor
- [ ] `/healthz`, 
- [ ] `/readyz`, 
- [ ] `/metrics`; 
- [ ] `gatos doctor`

## Phase 4 — Opaque Pointers + Proof Envelopes v1 (Weeks 7–8)

- [ ] opaque registry + rekey
- [ ] commitment proofs + verifier

## Phase 5 — Wesley Target (Weeks 9–10) 

- [ ] `wesley build --target gatos` emits schemas
- [ ] RLS
- [ ] folds
- [ ] Demo app (invoices)

## Phase 6 — Push-Gate Profile (Weeks 11–12)

- [ ] Optional gateway with RYW, HA, split-brain repair
- [ ] SaaS-hosted profile defaults

**Exit to GA**: all golden tests pass on CI matrix; docs + examples complete.
