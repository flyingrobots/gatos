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
    pub digest: String,
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

