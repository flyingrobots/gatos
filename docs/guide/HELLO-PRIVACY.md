# Hello, GATOS — Data Privacy Path

This walkthrough demonstrates the hybrid privacy model: creating an opaque pointer for a private blob, folding deterministically, and rekeying the blob.

> Model recap
>
> - Public state: pushable materialized state.
> - Private overlay: encrypted blobs addressed via opaque pointers.
> - Public pointers MUST NOT reveal a plaintext hash; store it inside encrypted meta (or use a hiding commitment).

## 0. Prepare a Private Blob

```bash
echo "Very secret string" > secret.txt
```

## 1. Add Encrypted Blob and Create Opaque Pointer

Use the CLI (or JSONL) to store an encrypted blob and emit an opaque pointer that references it.

```bash
# Example CLI (shape may evolve)
git gatos blob add --encrypt --file secret.txt --out pointers/secret.ptr.json
cat pointers/secret.ptr.json | jq .
```

A pointer contains at least:

```json
{
  "kind": "opaque",
  "algo": "blake3",
  "ciphertext_hash": "blake3:<hex>",
  "encrypted_meta": "base64:..."   // contains plaintext commitment, KMS refs, cipher params
}
```

Commit the pointer on an event, then fold:

```bash
git gatos event add --ns privacy --type demo.pointer --payload @pointers/secret.ptr.json
git gatos fold --ns privacy
```

`State-Root` is computed deterministically from the public shape. Authorized workers can decrypt and verify the plaintext commitment from `encrypted_meta` outside the repository as needed.

## 2. Rekey the Blob

Rotate keys without changing the underlying plaintext.

```bash
git gatos blob rekey --ptr pointers/secret.ptr.json --to kms://key/new \
  --out pointers/secret.rekeyed.ptr.json
```

Fold again; the public state’s `State-Root` remains stable if the high‑level state hasn’t changed:

```bash
git gatos fold --ns privacy
```

## 3. Determinism Notes

- The public `State-Root` for views that include only pointers is stable across rekeys.
- Any materialized computations over decrypted plaintext must use deterministic pipelines and should record proofs (e.g., PoE/ZK) if audited.

---

Troubleshooting
- Ensure storage backends provide a Blob Availability Attestation (BAA): `{ blob, store, retain_until, sig }`.
- Policy can require a valid BAA before accepting pointers.
