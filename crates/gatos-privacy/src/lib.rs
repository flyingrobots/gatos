//! gatos-privacy — Opaque Pointer types and helpers
//!
//! This crate defines the JSON-facing pointer envelope used by the
//! hybrid privacy model (ADR-0004). The struct mirrors the v1 schema
//! in `schemas/v1/privacy/opaque_pointer.schema.json`.
//!
//! Canonicalization: when computing content IDs or digests, callers
//! MUST serialize JSON using RFC 8785 JCS. This crate intentionally
//! does not take a dependency on a specific JCS implementation to
//! keep the workspace lean; higher layers may provide one.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpaquePointer {
    pub kind: Kind,
    pub algo: Algo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ciphertext_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    pub location: String,
    pub capability: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    OpaquePointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Algo {
    Blake3,
}

impl OpaquePointer {
    /// Validate invariants beyond serde schema mapping.
    pub fn validate(&self) -> Result<(), PointerError> {
        let has_plain = self.digest.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let has_cipher = self
            .ciphertext_digest
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !(has_plain || has_cipher) {
            return Err(PointerError::MissingDigest);
        }
        let low_entropy = self
            .extensions
            .as_ref()
            .and_then(|v| v.get("class"))
            .and_then(|c| c.as_str())
            .map(|s| s == "low-entropy")
            .unwrap_or(false);
        if low_entropy {
            if !has_cipher {
                return Err(PointerError::LowEntropyNeedsCiphertextDigest);
            }
            if has_plain {
                return Err(PointerError::LowEntropyForbidsPlainDigest);
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PointerError {
    #[error("at least one of digest or ciphertext_digest is required")]
    MissingDigest,
    #[error("low-entropy class requires ciphertext_digest")]
    LowEntropyNeedsCiphertextDigest,
    #[error("low-entropy class forbids plaintext digest")]
    LowEntropyForbidsPlainDigest,
}
