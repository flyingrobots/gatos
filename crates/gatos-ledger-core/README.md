# GATOS Ledger Core

This crate provides the `no_std`-compatible core logic for the GATOS ledger. It defines the pure, portable data structures and semantics for the commit graph, hashing, and proofs.

It defines the `ObjectStore` trait, which acts as a "port" for storage backends to implement.

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).

## Example: Content ID and Detached Signatures (ADR‑0001)

```rust
use blake3;
use gatos_ledger_core::{compute_content_id, CommitCore, Signature, PubKey};

fn main() {
    // Build the unsigned logical commit core
    let core = CommitCore {
        parent: None,
        tree: blake3::hash(b"tree-bytes").into(),
        message: "initial import".to_string(),
        timestamp: 1_725_000_000,
    };

    // Canonical identifier depends only on the core
    let content_id = compute_content_id(&core).expect("hash");
    println!("content-id blake3: {}", hex::encode(content_id));

    // Signatures are detached attestations over the content id
    let _sig = Signature { signer: PubKey::from([0u8; 32]), sig: [0u8; 64] };
    // Attaching, removing, or reordering signatures does not change `content_id`.
}
```
