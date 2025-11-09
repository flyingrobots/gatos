#![no_std]

//! GATOS Ledger Core — `no_std` primitives.
//!
//! This crate defines portable types and traits for the GATOS ledger.
//! It intentionally avoids `std` to run in constrained environments
//! (embedded, WASM-without-std).
//!
//! All serialization uses bincode v2 with `config::standard()`, ensuring
//! deterministic byte representations: identical structs produce identical
//! bytes across platforms and architectures (given a fixed bincode version).
//! Caveats: determinism assumes a fixed type definition and serialization
//! config across crate versions; floats are serialized bitwise; endianness is
//! normalized by the format; changing field order or enum variants will change
//! bytes.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use bincode::{config, encode_to_vec, Decode, Encode};
use serde_with::serde_as;
use smallvec::SmallVec;

/// 256-bit BLAKE3 content hash digest.
///
/// - Size: 32 bytes (verbatim byte array as produced by `blake3` — no
///   endianness reinterpretation).
/// - Usage: primary identifier for content-addressed objects and commits.
pub type Hash = [u8; 32];

/// Public key bytes for signature verification.
///
/// The concrete scheme (e.g., ed25519) is defined at the policy/enforcement
/// layer. We use a fixed 32-byte array here to keep the core portable and
/// deterministic across platforms and backends.
pub type PubKey = [u8; 32];

/// Errors produced by storage backends implementing [`ObjectStore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// Underlying backend I/O or system error.
    Io,
    /// Data corruption detected (e.g., hash mismatch).
    Corruption,
    /// Operation unsupported by this backend.
    Unsupported,
    /// Internal invariant violation.
    Invariant,
}

/// Abstraction for content-addressed object storage.
///
/// Backends MUST ensure that `id` is the BLAKE3 hash of `data` when storing
/// content. Implementations SHOULD be idempotent: storing the same `(id, data)`
/// pair multiple times is not an error.
pub trait ObjectStore {
    /// Persist bytes under the given content `id`.
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    /// Returns a [`StoreError`] if the backend fails to persist the bytes or
    /// if invariants (such as id/content mismatch) are violated.
    fn put_object(&mut self, id: &Hash, data: &[u8]) -> Result<(), StoreError>;

    /// Retrieve bytes by content `id`.
    ///
    /// Returns `Ok(Some(Vec<u8>))` if present, `Ok(None)` if not found.
    ///
    /// # Errors
    /// Returns a [`StoreError`] if the backend fails to access the underlying
    /// storage or detects corruption.
    fn get_object(&self, id: &Hash) -> Result<Option<Vec<u8>>, StoreError>;
}

/// Immutable core content of a commit (unsigned).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Encode, Decode)]
pub struct CommitCore {
    pub parent: Option<Hash>,
    pub tree: Hash,
    /// Human-readable message describing the change (canonicalized bytes).
    pub message: String,
    /// Seconds since Unix epoch (UTC). Treated as data; determinism is
    /// preserved because the canonical id is a function of this value.
    pub timestamp: u64,
}

/// A detached signature over a `CommitCore` content id, with signer metadata.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Encode, Decode)]
pub struct Signature {
    /// Signer public key bytes (scheme defined by policy layer).
    #[serde_as(as = "[_; 32]")]
    pub signer: PubKey,
    /// 64-byte signature over the `CommitCore` content id, optionally bound to
    /// additional metadata at higher layers (e.g., policy_root).
    #[serde_as(as = "[_; 64]")]
    pub sig: [u8; 64],
}

/// A commit container: the logical core plus zero or more signatures.
///
/// IMPORTANT: The canonical commit identifier is derived solely from the
/// serialized `CommitCore`. Signatures do not affect the identifier.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Commit {
    pub core: CommitCore,
    #[serde(skip_serializing_if = "SmallVec::is_empty", default)]
    pub sigs: SmallVec<[Signature; 2]>,
}

/// Compute the canonical commit identifier for a `Commit`.
///
/// This is defined as the BLAKE3 hash of the canonical bincode encoding of the
/// unsigned `CommitCore` only. It is therefore invariant under the set/order of
/// signatures present in [`Commit::sigs`].
///
/// # Errors
/// Returns an error if serialization fails.
pub fn compute_commit_id(commit: &Commit) -> Result<Hash, bincode::error::EncodeError> {
    compute_content_id(&commit.core)
}

/// Compute the deterministic content id for unsigned commit content.
///
/// Uses bincode with standard configuration for canonical serialization,
/// followed by BLAKE3 hashing. Deterministic across platforms for identical
/// inputs and a fixed serialization config.
///
/// # Errors
/// Returns an error if serialization fails under the canonical configuration.
pub fn compute_content_id(core: &CommitCore) -> Result<Hash, bincode::error::EncodeError> {
    let bytes = encode_to_vec(core, config::standard())?;
    Ok(blake3::hash(&bytes).into())
}

// Test support: enable std for unit tests.
#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;
    use std::string::ToString;

    fn fixed_core() -> CommitCore {
        CommitCore {
            parent: Some([0x11; 32]),
            tree: [0x22; 32],
            message: "hello".to_string(),
            timestamp: 1_725_000_000,
        }
    }

    // Note: Commit is encoded/decoded via serde formats externally; the
    // canonical id is derived from the core only, tested below.

    #[test]
    fn test_compute_commit_id_invariant_under_signatures() {
        let core = fixed_core();
        let id_core = compute_content_id(&core).unwrap();

        // No signatures
        let commit0 = Commit {
            core: core.clone(),
            sigs: SmallVec::new(),
        };
        let id0 = compute_commit_id(&commit0).unwrap();

        // One signature
        let sig1 = Signature {
            signer: [0xAA; 32],
            sig: [0xBB; 64],
        };
        let commit1 = Commit {
            core: core.clone(),
            sigs: smallvec![sig1.clone()],
        };
        let id1 = compute_commit_id(&commit1).unwrap();

        // Two signatures (different order)
        let sig2 = Signature {
            signer: [0xCC; 32],
            sig: [0xDD; 64],
        };
        let commit2a = Commit {
            core: core.clone(),
            sigs: smallvec![sig1, sig2.clone()],
        };
        let commit2b = Commit {
            core: core.clone(),
            sigs: smallvec![
                sig2,
                Signature {
                    signer: [0xAA; 32],
                    sig: [0xBB; 64]
                }
            ],
        };
        let id2a = compute_commit_id(&commit2a).unwrap();
        let id2b = compute_commit_id(&commit2b).unwrap();

        assert_eq!(id_core, id0);
        assert_eq!(id0, id1);
        assert_eq!(id1, id2a);
        assert_eq!(id2a, id2b);
    }

    #[test]
    fn test_compute_content_id_stability() {
        let core = fixed_core();
        let id1 = compute_content_id(&core).unwrap();
        let id2 = compute_content_id(&core).unwrap();
        assert_eq!(id1, id2);
    }
}
