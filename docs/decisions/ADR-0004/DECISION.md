---
Status: Accepted
Date: 2025-11-10
ADR: ADR-0004
Authors: [flyingrobots, gemini-agent]
Requires: [ADR-0001]
Related: [ADR-0002, ADR-0003]
Tags: [Privacy, Projection, Opaque Pointers, Morphology Calculus]
Schemas:
  - schemas/v1/privacy/opaque_pointer.schema.json
---

# ADR‑0004: Hybrid Privacy Model (Public Projection + Private Overlay)

## Scope

This ADR defines a **hybrid privacy model** for the GATOS operating surface. It formalizes the separation of state into a public, verifiable component and a private, actor-anchored overlay. This is achieved by introducing a **Projection Functor** that transforms a unified state into a public projection, leaving sensitive data in a private store referenced by **Opaque Pointers**.

## Rationale

GATOS's core value proposition is its verifiable, deterministic public ledger. However, many real-world applications require storing sensitive or large data (PII, secrets, large binaries) without committing it to the public history. The previous ad-hoc approach of using local, out-of-repo storage lacks the formal guarantees required by the GATOS Morphology Calculus.

This ADR makes the hybrid model **normative, deterministic, and provable**. It ensures that public state remains globally verifiable while private data is securely addressable, auditable, and tied to the GATOS identity and policy model.

## Mathematical Foundation (Morphology Calculus)

This model is a direct application of the GATOS Morphology Calculus.

1.  **Shape Categories**: We define three categories of shapes:
    *   `Sh_Unified`: The category of shapes containing both public and private data.
    *   `Sh_Public`: The category of shapes containing only public data and opaque pointers.
    *   `Sh_Private`: The category of shapes containing only the private data blobs.

2.  **Projection as a Functor**: The privacy model is implemented as a functor, `Proj`, which maps shapes and morphisms from the unified category to the public category.
    `Proj: Sh_Unified -> Sh_Public`

    This functor applies the privacy policy rules (`redact`, `pointerize`) to transform a unified shape into its public projection. The private data is extracted into `Sh_Private` during this process.

    ```mermaid
    graph TD
        subgraph Sh_Unified
            U1("Unified Shape 1")
            U2("Unified Shape 2")
            U1 -- "Commit c" --> U2
        end

        subgraph Sh_Public
            P1("Public Shape 1")
            P2("Public Shape 2")
            P1 -- "Proj(c)" --> P2
        end

        subgraph Sh_Private
            B1("Private Blobs 1")
            B2("Private Blobs 2")
        end

        U1 -- "Proj" --> P1
        U2 -- "Proj" --> P2

        U1 -- "Extract" --> B1
        U2 -- "Extract" --> B2

        style P1 fill:#cde,stroke:#333
        style P2 fill:#cde,stroke:#333
    ```

This ensures that the transformation is structure-preserving and that the public history remains a valid, deterministic projection of the complete history.

## Decision

### 1. Actor-Anchored Private Namespace (Normative)

Private data overlays are fundamentally tied to an actor's identity, not an ephemeral session. This anchors private data within the GATOS trust graph.

-   **Actor ID:** The canonical identifier for an actor, e.g., `ed25519:<pubkey>`.
-   **Private Refs:** Private data is stored under refs namespaced by the actor ID.
    ```
    refs/gatos/private/<actor-id>/<ns>/<channel>
    ```
-   **Public Refs:** The corresponding public projection lives in the main state namespace.
    ```
    refs/gatos/state/public/<ns>/<channel>
    ```

### 2. Opaque Pointers (Normative)

When private data is elided from the `PublicState`, a canonical JSON **Opaque Pointer** envelope is inserted in its place.

```mermaid
classDiagram
    class OpaquePointer {
        +string kind: "opaque_pointer"
        +string algo: "blake3"
        +string digest: "blake3:<hex>"            // plaintext digest
        +string ciphertext_digest "blake3:<hex>"  // MAY be present
        +int    size                                // SHOULD be present (bytes)
        +string location
        +string capability                          // MUST NOT embed secrets
        +object extensions                          // forward-compatible
    }
```

-   **`digest`**: The content-address of the private plaintext (`blake3(plaintext_bytes)`). This is the immutable link between the public and private worlds.
-   **`ciphertext_digest`**: The content-address of the stored ciphertext (`blake3(ciphertext_bytes)`). For low‑entropy privacy classes (see Policy Hooks), the public pointer **MUST** include `ciphertext_digest` and policy **MUST NOT** expose the plaintext digest publicly.
-   **`location`**: A URI indicating where to resolve the blob. Supported schemes include:
    -   `gatos-node://ed25519:<pubkey>`: Resolve via the GATOS trust graph.
    -   `https://...`, `s3://...`, `ipfs://...`: Standard distributed storage.
    -   `file:///...`: For local development and testing.
-   **`capability`**: A reference identifying the authorization and decryption mechanism required to access the blob. It **MUST NOT** embed secrets or pre‑signed tokens. It SHOULD be a stable identifier (e.g., `gatos-key://v1/aes-256-gcm/<key-id>` or `kms://...`) that can be resolved privately at the policy layer.
    -   Pointers MAY publish a non‑sensitive label and keep resolver details private via policy. Implementations MAY also place auxiliary hints inside `extensions`.

The canonical `content_id` of the pointer itself is `blake3(JCS(pointer_json))`, where `JCS(…)` denotes RFC 8785 JSON Canonicalization Scheme applied to UTF‑8 bytes. This rule is normative for all canonical JSON in GATOS (pointers, governance envelopes, any JSON state snapshots).

**Schema:** `schemas/v1/privacy/opaque_pointer.schema.json`

### 3. The Projection Function (Normative)

The State Engine (`gatos-echo`) is responsible for executing the projection.

1.  It computes a **UnifiedState** by folding the complete event history.
2.  It consults the **Privacy Policy** (`.gatos/policy.yaml`).
3.  It traverses the `UnifiedState` tree, applying `redact` or `pointerize` rules.
    -   `redact`: The field is removed from the public state.
    -   `pointerize`: The field's value is stored as a private blob, and an Opaque Pointer is substituted in the public state.
4.  The resulting `PublicState` is committed to the public refs, and the `Private Blobs` are persisted to their specified `location`.

Determinism Requirements:
- All JSON artifacts produced during projection (including Opaque Pointers) MUST be canonicalized with RFC 8785 JCS prior to hashing.
- When non‑JSON maps are materialized (e.g., Git tree entries), keys MUST be ordered lexicographically by their lowercase UTF‑8 bytes.

```mermaid
sequenceDiagram
    participant E as State Engine (gatos-echo)
    participant Pol as Policy Engine
    participant L as Ledger (Git)
    participant PS as Private Store

    E->>E: 1. Fold history into UnifiedState
    E->>Pol: 2. Fetch privacy rules
    Pol-->>E: 3. Return rules (redact/pointerize)
    E->>E: 4. Apply rules to create PublicState + PrivateBlobs
    E->>L: 5. Commit PublicState to public refs
    E->>PS: 6. Store PrivateBlobs by digest
```

### 4. Pointer Resolution Protocol (Normative)

Authentication semantics are aligned with HTTP. We adopt a simple, interoperable model (JWT default; HTTP Message Signatures optional):

-  **Endpoint**: `POST /gatos/private/blobs/resolve`
-  **Request Body (application/json; JCS canonical form)**:
   `{ "digest": "blake3:<hex>", "want": "plaintext"|"ciphertext" }`
-  **Authorization**: `Authorization: Bearer <JWT>`
   - Claims MUST include: `sub` (ed25519:<pubkey>), `aud` (node id or URL), `method` ("POST"), `path` ("/gatos/private/blobs/resolve"), `exp`, and `nbf`.
   - Clock skew tolerance: ±300 seconds.
   - On missing/invalid token: `401 Unauthorized`. On policy denial: `403 Forbidden`.

A client resolving an Opaque Pointer **MUST** follow this protocol:

1.  **Parse Pointer**: Extract `digest`, optional `ciphertext_digest`, `location`, and `capability`.
2.  **Fetch Blob**:
    -   If `gatos-node://<actor-id>`, resolve the actor's endpoint from the trust graph, then `POST /gatos/private/blobs/resolve` with the body above.
    -   The node **MUST** verify the bearer token and enforce policy before returning the blob.
3.  **Acquire Capability**:
    -   Resolve the `capability` reference via the configured key system (KMS, key server). Secrets MUST NOT be embedded in the pointer.
4.  **Decrypt and Verify**:
    -   Decrypt the fetched blob using the resolved key and AAD parameters (see Security Notes).
    -   Compute `blake3(plaintext)` and compare to `digest` if published; compute `blake3(ciphertext)` and compare to `ciphertext_digest` if published. A mismatch **MUST** produce `DigestMismatch`.

Response headers on success:
```
Content-Type: application/octet-stream
X-BLAKE3-Digest: blake3:<hex-of-ciphertext>
Digest: sha-256=<base64-of-ciphertext>
```

Optional HTTP Message Signatures profile (RFC 9421):
- Clients MAY authenticate by signing `@method`, `@target-uri`, `date`, `host`, `content-digest` (SHA‑256 of the JSON body) and sending `Signature-Input` and `Signature` headers.
- Servers SHOULD still return `Digest` and `X-BLAKE3-Digest` headers for response integrity.

```mermaid
sequenceDiagram
    participant C as Client
    participant PN as Private GATOS Node
    participant KMS as Key Management Service

    C->>C: 1. Read OpaquePointer
    C->>PN: 2. POST /gatos/private/blobs/resolve (Authorization: Bearer <JWT>)
    PN->>PN: 3. Check policy (is C allowed?)
    alt Authorized
        PN-->>C: 4. Return encrypted blob
        C->>KMS: 5. Request key for {capability}
        KMS-->>C: 6. Return decryption key
        C->>C: 7. Decrypt blob
        C->>C: 8. Verify blake3(decrypted) == digest
    else Unauthorized
        PN-->>C: 4. Return 401/403
    end
```

### 5. Policy Hooks (Normative)

The privacy policy is defined in `.gatos/policy.yaml` and extends the policy engine's domain.

```yaml
privacy:
  classes:
    pii_low_entropy:
      min_entropy_bits: 40
      publish_plaintext_digest: false
      require_ciphertext_digest: true
  rules:
    - select: "path.to.sensitive.data"
      action: "pointerize"
      class: "pii_low_entropy"
      capability: "gatos-key://v1/aes-256-gcm/ops-key-01"
      location: "gatos-node://ed25519:<owner-pubkey>"
    - select: "path.to.transient.data"
      action: "redact"
```

The `select` syntax will use a simple path-matching language (e.g., glob patterns) defined by the policy engine.

### 6. Auditability and Trailers (Normative)

To make privacy operations transparent and auditable, any commit that creates a `PublicState` from a projection **MUST** include the following trailers:

```
Privacy-Redactions: 3
Privacy-Pointers: 12
Privacy-Pointer-Rotations: 1
```

This provides a simple, top-level indicator that a projection has occurred, prompting auditors to look deeper if necessary.

## Consequences

### Pros

-   **Provable Privacy**: The model is grounded in the Morphology Calculus, making it verifiable.
-   **Decoupled Storage**: Private data can live in any storage system (S3, IPFS, local disk) without affecting the public ledger's logic.
-   **Integrated Auth/Authz**: By tying pointers to actor identities and capabilities, access to private data is governed by the existing GATOS trust and policy model.
-   **Preserves Verifiability**: The `PublicState` remains globally verifiable, as pointers are just content-addressed links.

### Cons

-   **Increased Complexity**: Resolution requires network requests and interaction with key management systems, adding latency and potential points of failure.
-   **Operational Overhead**: Operators must manage the private blob stores and ensure their availability and security.

## Feature Payoff

-   **Secure PII/Secret Storage**: Store sensitive data off-chain while retaining an auditable link to it.
-   **Large Artifact Management**: Handle large binaries (ML models, videos) without bloating the Git repository.
-   **Compliant Data Sharing**: Share a public, redacted dataset with third parties while retaining private access to the full, unified view.
-   **Federated Learning**: Different actors can hold private models locally, referenced by pointers in a public "training plan" shape.

---

## Namespacing and Storage (Normative)

-   Private overlays are actor‑anchored: `refs/gatos/private/<actor-id>/<ns>/<channel>` index metadata. The local workspace mirror is `gatos/private/<actor-id>/<ns>/<channel>`.
-   Private blobs themselves are NOT stored under Git refs. They live in pluggable blob stores and are addressed by their `ciphertext_digest`/`digest`.

## Security & Privacy Notes (Normative)

-   Capability references in pointers MUST NOT contain secrets or pre‑signed tokens. Use stable identifiers and resolve sensitive data via policy.
-   AES‑256‑GCM (if used) MUST include AAD composed of: actor id, pointer `content_id`, and policy version; nonces MUST be 96‑bit, randomly generated, and never reused per key.
-   Right‑to‑be‑forgotten: deleting private blobs breaks pointer resolution but does not remove the public pointer. Implement erasure as a tombstone event plus an audit record.

### Algorithm variants (experimental; private attestations only)

- Implementations MAY use a keyed BLAKE3 variant for private attestation envelopes (not for public Opaque Pointers): `algo = "blake3-keyed"` with parameters encoded in an envelope or pointer `extensions` field.
- Recommended KDF: `hkdf-sha256`; context string `"gatos:ptr:priv:<policy_id>"`; derive `key = HKDF(policy_key, salt = actor_pubkey, info = context)`.
- Public pointers MUST continue to use `algo = "blake3"` for third‑party verifiability.

## Error Taxonomy (Normative)

Implementations SHOULD use a stable set of error codes with JSON problem details:

- `Unauthorized` (401)
- `Forbidden` (403)
- `NotFound` (404)
- `DigestMismatch` (422)
- `CapabilityUnavailable` (503)
- `PolicyDenied` (403)
