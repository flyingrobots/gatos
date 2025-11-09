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

extern crate alloc;
use alloc::vec::Vec;

use bincode::{config, encode_to_vec, Decode, Encode};
use serde_with::serde_as;

/// 256-bit BLAKE3 content hash digest.
///
/// - Size: 32 bytes (verbatim byte array as produced by `blake3` — no
///   endianness reinterpretation).
/// - Usage: primary identifier for content-addressed objects and commits.
pub type Hash = [u8; 32];

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

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Encode, Decode)]
pub struct Commit {
    /// Optional parent commit id (None for roots).
    pub parent: Option<Hash>,
    /// Hash of the tree (content root) this commit points to.
    pub tree: Hash,
    /// 64-byte author/issuer signature over canonical commit bytes.
    /// The scheme is defined at the policy/enforcement layer.
    #[serde_as(as = "[_; 64]")]
    pub signature: [u8; 64],
}

/// Compute the deterministic content hash for a commit.
///
/// Uses bincode with standard configuration for canonical serialization,
/// followed by BLAKE3 hashing. Deterministic across platforms for identical
/// inputs and a fixed serialization config.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn compute_commit_id(commit: &Commit) -> Result<Hash, bincode::error::EncodeError> {
    let bytes = encode_to_vec(commit, config::standard())?;
    Ok(blake3::hash(&bytes).into())
}

// Test support: enable std for unit tests.
#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{config, decode_from_slice};

    fn fixed_commit() -> Commit {
        Commit {
            parent: Some([0x11; 32]),
            tree: [0x22; 32],
            signature: [0x33; 64],
        }
    }

    #[test]
    fn test_commit_roundtrip() {
        let c = fixed_commit();
        let bytes = encode_to_vec(&c, config::standard()).unwrap();
        let (decoded, consumed): (Commit, usize) =
            decode_from_slice(&bytes, config::standard()).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded.parent, c.parent);
        assert_eq!(decoded.tree, c.tree);
        assert_eq!(decoded.signature, c.signature);
    }

    #[test]
    fn test_commit_serialization_determinism() {
        let c = fixed_commit();
        let a = encode_to_vec(&c, config::standard()).unwrap();
        let b = encode_to_vec(&c, config::standard()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_compute_commit_id_stability() {
        let c = fixed_commit();
        let id1 = compute_commit_id(&c).unwrap();
        let id2 = compute_commit_id(&c).unwrap();
        assert_eq!(id1, id2);
    }
}
