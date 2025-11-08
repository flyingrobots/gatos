# GATOS — STAKEHOLDERS & GOALS

> [!NOTE]
> We define primary stakeholders and their core goals. 
> (User stories are generated per feature in `FEATURES.md`.)

## Stakeholders

### App Developer (DEV)

— build features, 
- test locally, 
- commit artifacts, 
- script automations.
  
### Platform Engineer (PENG)

- operate repos at scale, 
- enforce policy, 
- ensure latency/throughput, 
- maintain health.
  
### Security/Compliance (SEC) 

— enforce least privilege, 
- attest changes, 
- audit trails, 
- key rotation.

### SRE / Ops (SRE) 

— deploy safely, 
- debug incidents, 
- maintain SLAs, 
- post‑mortems.
  
### Data/ML Engineer (DML) 

— manage large artifacts, 
- track lineage, 
- reproduce results across air‑gapped nodes.
  
### Auditor / GRC (AUD) 

— independently verify history and policy decisions; 
- ensure evidence sufficiency.
  
### LLM/Agent (AGENT) 

— interact programmatically via JSONL; 
- publish/consume messages under caps/policy.

### Product/Program Manager (PM)

— visibility across environments, 
- approvals, 
- governance gates.

## Goals Synopsis

- `DEV`: determinism, fast local loop, simple APIs.  
- `PENG`: predictable ops (GC, epochs), observability, policy profiles.  
- `SEC`: artifacts-as-law, explainable denies, signatures everywhere.  
- `SRE`: exactly-once buses, replayable state, clean audit of deploys.  
- `DML`: chunking/opaque pointers, RLS export, provenance.  
- `AUD`: portable verification, no SaaS dependency, proof envelopes.  
- `AGENT`: streaming JSONL, idempotent semantics, backpressure.  
- `PM`: approvals/quorum, evidence dashboards, governance diffs.
