# GATOS — Task Breakdown

## EPIC-1: Ledger Core

- [ ] Implement GitLedger.append (CAS update-ref)
- [ ] Canonical JSON (sorted keys) serializer
- [ ] CLI: `gatos event add`
- [ ] Tests:
  - [ ] concurrent append,
  - [ ] deny audit

## EPIC-2: Policy VM

- [ ] RGS→RGC compiler
- [ ] Pure interpreter (no I/O)
- [ ] `gatos policy check`
- [ ] Tests: deterministic verdicts

## EPIC-3: Fold Engine Integration

- [ ] Echo FoldEngine adapter
- [ ] `gatos fold` + checkpoints
- [ ] Golden vectors across OS/arch

## EPIC-4: Message Bus

- [ ] Topics/shards;
- [ ] publish/subscribe
- [ ] Acks & commitments;
- [ ] dedupe
- [ ] Tests:
  - [ ] exactly-once torture

## EPIC-5: Sessions

- [ ] start
- [ ] undo
- [ ] fork
- [ ] merge
- [ ] lattice/DPO joins for conflicts

## EPIC-6: CAS & Opaque Pointers

- [ ] FastCDC integration;
- [ ] manifests
- [ ] Rekey;
- [ ] export policy controls

## EPIC-7: Epochs & Compaction

- [ ] epoch anchors;
- [ ] compact & verify
- [ ] `gatos doctor` invariants

## EPIC-8: Observability

- [ ] `/healthz`,
- [ ] `/readyz`,
- [ ] `/metrics`
- [ ] Prometheus exporter

## EPIC-9: Proof Envelopes

- [ ] commitment prover/verifier
- [ ] ZK backend trait

## EPIC-10: Wesley Target

- [ ] schema emission;
- [ ] RLS bundles
- [ ] example demo repo

## EPIC-11: Push-Gate Profile

- [ ] Gateway service;
- [ ] RYW waiters
- [ ] Split-brain repair tools
