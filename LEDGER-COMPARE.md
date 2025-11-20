# Ledger Kernel vs. gatos-ledger Requirements

This note contrasts the upstream **ledger-kernel** specification (`/Users/james/git/ledger-kernel/SPEC.md`, v0.1.0, 2025-10-27) with the GATOS ledger requirements defined in **SPEC §4** and **ADR-0001**. It highlights alignments, gaps, and the concrete work needed to resurrect `crates/gatos-ledger-git`.

## 1. Where They Already Align

| Theme | Ledger-Kernel | GATOS Requirements | Notes |
| --- | --- | --- | --- |
| Append-only refs | §4 “Core Invariants” mandate append-only, FF-only commits per ledger | SPEC §4.2 journals (`refs/gatos/journal/<ns>/<actor>`) are FF-only with CAS enforcement | Same high-level invariant; naming/layout differs. |
| Policy gating | `evaluate(R,L,E,P)` + invariant 4.5 require deterministic policy checks before append | Every EventEnvelope carries `policy_root`; pushes run through `gatos-policy` before journals | Both insist on deterministic policies, but GATOS binds the specific policy commit to each event. |
| Attestations | Canonical attestation schema (signer, algorithm, signature, scope) | Event envelopes + commit trailers capture signatures (`ed25519` at minimum) | Scope semantics differ but both rely on cryptographic signatures. |
| Replay/determinism | §3 `replay` + invariant 4.3 (“Replay operations must be deterministic”) | SPEC §5 folds + ADR-0014 PoF enforce deterministic replay | Replay conceptually identical; GATOS layers fold engines + proofs on top. |

## 2. Key Divergences & Required Adjustments

| Area | Ledger-Kernel Spec | GATOS Expectation | Work Needed for `gatos-ledger-git` |
| --- | --- | --- | --- |
| **Ref namespace** | `refs/_ledger/<namespace>/current`, `/attest`, `/policy`, `/meta` (§7) | `refs/gatos/journal/<ns>/<actor>` for events; audit/policy refs elsewhere | Implement adapter translating GATOS namespaces; keep `_ledger` layout only if compatibility mode requested. |
| **Entry / Envelope format** | Abstract entry schema (id, parent, timestamp, payload, attestations). Serialization left to implementation (§5.1). | EventEnvelope defined in SPEC §4.1 — DAG-CBOR canonical bytes, ULIDs, `actor`, `caps`, `policy_root`, `payload`, optional `sig_alg` & `ts`. | Implement DAG-CBOR serializer + ULID idempotency per SPEC; extend ledger-kernel payload model with `caps` + `policy_root` fields. |
| **Capability & trust integration** | No concept of capability tokens or trust graph. | EventEnvelope requires `caps` array + `actor` resolution and audit of denies (FEATURES F4, ADR-0003). | `gatos-ledger-git` must call trust graph before append; ledger-kernel compliance alone is insufficient. |
| **Audit refs** | Suggests attestation refs (`refs/_ledger/<namespace>/attest`), but no deny log. | GATOS logs denies at `refs/gatos/audit/policy/deny/<ulid>` and sessions/watcher logs elsewhere. | Append helper must emit audit commits on failure paths. |
| **Metadata binding** | Kernel schema lacks `policy_root`, `fold_root`, `session_id`. | GATOS requires `policy_root` per event, fold/pointer metadata elsewhere. | Extend ledger core structs to carry extra trailers/payload fields; ensure `gatos-ledger-git` writes them into commit metadata. |
| **Transport/API** | No RPC layer defined. | GATOS CLI/daemon expose JSONL `event.append`, watchers, Stargate push-gate integrations. | Build Rust APIs that integrate with `gatosd` JSONL protocol and Stargate. |
| **Profiles / Research mode** | Not addressed. | Research profile enforces PoF, PoE, PoX, GC anchors (§12). | Ledger backend must expose hooks so push-gate can enforce profile-specific invariants (e.g., verifying PoF before accepting state refs). |
| **`no_std` core vs std backend** | Kernel spec assumes standard environment. | ADR-0001 splits `gatos-ledger-core` (no_std) from std backends (git). | Rebuild `gatos-ledger-core` traits that wrap ledger-kernel concepts, then implement the std backend (`gatos-ledger-git`) on top of libgit2. |
| **Namespace isolation** | Strict: “each ledger’s validity depend solely on its own references.” | GATOS journals often interact with other refs (policy refs, mounts, message plane) through policy metadata. | Document how GATOS reintroduces cross-ref dependencies yet maintains verifiability via `policy_root` & audit refs. |

## 3. Implications for `gatos-ledger-git`

1. **Adopt ledger-kernel invariants but map them to GATOS ref layout.** Implement wrapper traits so we can eventually run ledger-kernel’s compliance suite while still writing to `refs/gatos/journal/**`.
2. **Implement DAG-CBOR serialization + ULID idempotency.** Reuse the EventEnvelope spec; ensure hashes/IDs comply with ledger-kernel’s `Entry.id` invariant.
3. **Bind policy + capability context.** Each append must:
   - Resolve the actor via the trust graph (ADR-0003).
   - Verify capability grants/caps before writing.
   - Store `policy_root` in both the envelope payload and commit trailers.
4. **Emit audit artifacts.** Success path writes the journal commit; failure path writes deny/audit entries for policy/watchers.
5. **Expose ergonomic APIs.** `gatos-ledger-core` should define traits like `EventStore`/`Journal`, with the git backend implementing them via libgit2. Keep `no_std` core free of git dependencies as ADR-0001 requires.
6. **Testing Strategy.**
   - Port ledger-kernel compliance tests where possible (append-only, deterministic replay).
   - Add CAS contention tests specific to `refs/gatos/journal/**`.
   - Add DAG-CBOR canonicalization golden vectors (SPEC §4.1 references ADR-0001).
   - Verify policy/audit hooks (deny logs, capability enforcement).

## 4. Next Steps

1. **Documented comparison (this file).** ✅
2. **Update `docs/decisions/ADR-0001`** to note ledger backends are missing and reference this comparison in the “Implementation Notes”.
3. **Recreate `crates/gatos-ledger-core` + `crates/gatos-ledger-git`.**
   - `gatos-ledger-core`: traits for EventEnvelope serialization, CAS operations, policy callbacks (no_std friendly).
   - `gatos-ledger-git`: libgit2 backend providing CAS append/read/replay + audit hooks, and optionally adapters for ledger-kernel namespaces.
4. **Add integration tests** under `tests/ledger/` covering append, deny, replay, and cross-profile behavior.
5. **Re-run workspace builds** so downstream crates (policy, message plane, jobs) can link against the restored ledger backend.
