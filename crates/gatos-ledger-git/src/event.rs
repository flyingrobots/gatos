//! Event envelope types and validation for the git-backed ledger.
//!
//! ## Security
//!
//! Event envelopes are validated before being committed to prevent injection attacks:
//!
//! - **ULID validation**: Must be exactly 26 uppercase Crockford base32 characters
//!   to prevent newline injection in git commit messages
//! - **Event type validation**: Only allows alphanumeric, '.', '-', '_' characters
//!   to prevent newline and control character injection in git commit messages
//!
//! See [`EventEnvelope::validate`] for details.

use cid::Cid;
use ed25519_dalek::{SignatureError, Signer, SigningKey, VerifyingKey};
use multihash::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};
use serde_ipld_dagcbor::to_vec;
use serde_json::Value;

/// Minimal error type for ledger operations.
#[derive(Debug)]
pub enum LedgerError {
    NotImplemented,
    Encode(String),
    Signature(SignatureError),
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "not implemented"),
            Self::Encode(e) => write!(f, "encode error: {e}"),
            Self::Signature(e) => write!(f, "signature error: {e}"),
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<SignatureError> for LedgerError {
    fn from(err: SignatureError) -> Self {
        LedgerError::Signature(err)
    }
}

pub type LedgerResult<T> = Result<T, LedgerError>;

/// Canonical event envelope per SPEC §4.1 (fields subset; extended as needed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    pub event_type: String,
    pub ulid: String,
    pub actor: String,
    #[serde(default)]
    pub caps: Vec<String>,
    pub payload: Value,
    pub policy_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig_alg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
}

impl EventEnvelope {
    /// Serialize to canonical DAG-CBOR bytes.
    pub fn canonical_bytes(&self) -> LedgerResult<Vec<u8>> {
        to_vec(self).map_err(|e| LedgerError::Encode(e.to_string()))
    }

    /// Compute CID (dag-cbor + blake3) of canonical bytes.
    pub fn event_cid(&self) -> LedgerResult<String> {
        let bytes = self.canonical_bytes()?;
        let mh = Code::Blake3_256.digest(&bytes);
        const DAG_CBOR_CODEC: u64 = 0x71;
        let cid = Cid::new_v1(DAG_CBOR_CODEC, mh);
        Ok(cid.to_string())
    }

    /// Validate envelope fields to prevent injection attacks.
    pub fn validate(&self) -> Result<(), String> {
        validate_ulid(&self.ulid)?;
        validate_event_type(&self.event_type)?;
        Ok(())
    }
}

/// Validate ULID format to prevent commit message injection.
///
/// ULIDs must be exactly 26 characters of uppercase Crockford base32 (0-9A-HJKMNP-TV-Z).
/// This prevents newline injection in git commit messages.
fn validate_ulid(ulid: &str) -> Result<(), String> {
    const ULID_LEN: usize = 26;
    const CROCKFORD_BASE32: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

    if ulid.len() != ULID_LEN {
        return Err(format!(
            "invalid ulid '{}': must be exactly {} characters",
            ulid, ULID_LEN
        ));
    }

    if !ulid.chars().all(|c| CROCKFORD_BASE32.contains(c)) {
        return Err(format!(
            "invalid ulid '{}': must be uppercase Crockford base32 (0-9A-HJKMNP-TV-Z)",
            ulid
        ));
    }

    Ok(())
}

/// Validate event type to prevent commit message injection.
///
/// Event types must:
/// - Allow only alphanumeric, '.', '-', '_'
/// - Max length: 64 chars
/// - Reject newlines and control characters
fn validate_event_type(event_type: &str) -> Result<(), String> {
    const MAX_EVENT_TYPE_LEN: usize = 64;

    if event_type.is_empty() {
        return Err("event_type cannot be empty".into());
    }

    if event_type.len() > MAX_EVENT_TYPE_LEN {
        return Err(format!(
            "event_type exceeds max length {}",
            MAX_EVENT_TYPE_LEN
        ));
    }

    // Explicitly reject newlines and control characters
    if event_type.chars().any(|c| c == '\n' || c == '\r' || c.is_control()) {
        return Err(format!(
            "invalid event_type '{}': contains newlines or control characters",
            event_type
        ));
    }

    // Only allow alphanumeric, '.', '-', '_'
    if !event_type
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err(format!(
            "invalid event_type '{}': only alphanumeric, '.', '-', '_' allowed",
            event_type
        ));
    }

    Ok(())
}

/// Sign an event envelope (over canonical bytes).
pub fn sign_event(env: &EventEnvelope, key: &SigningKey) -> LedgerResult<ed25519_dalek::Signature> {
    let bytes = env.canonical_bytes()?;
    Ok(key.sign(&bytes))
}

/// Verify an event envelope signature.
pub fn verify_event(
    env: &EventEnvelope,
    key: &VerifyingKey,
    sig: &ed25519_dalek::Signature,
) -> LedgerResult<bool> {
    let bytes = env.canonical_bytes()?;
    Ok(key.verify_strict(&bytes, sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn envelope(ulid: &str) -> EventEnvelope {
        EventEnvelope {
            event_type: "event.test".into(),
            ulid: ulid.into(),
            actor: "user:alice".into(),
            caps: vec![],
            payload: json!({"x": 1}),
            policy_root: "deadbeef".into(),
            sig_alg: None,
            ts: None,
        }
    }

    #[test]
    fn validate_ulid_rejects_newline_injection() {
        let env = envelope("01ARZ3\nMalicious: evil");
        assert!(env.validate().is_err());
    }

    #[test]
    fn validate_ulid_rejects_invalid_chars() {
        let env = envelope("01ARZ3NDEKTSV4RRFFQ69G5F@V");
        assert!(env.validate().is_err());

        let env2 = envelope("01ARZ3NDEKTSV4RRFFQ69G5F+V");
        assert!(env2.validate().is_err());
    }

    #[test]
    fn validate_ulid_rejects_wrong_length() {
        let env = envelope("01ARZ3");
        assert!(env.validate().is_err());

        let env2 = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAVEXTRA");
        assert!(env2.validate().is_err());

        let env3 = envelope("");
        assert!(env3.validate().is_err());
    }

    #[test]
    fn validate_ulid_accepts_valid_ulid() {
        let env = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert!(env.validate().is_ok());
    }

    #[test]
    fn validate_event_type_rejects_newline_injection() {
        let mut env = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        env.event_type = "event.append\nSigned-off-by: evil".into();
        assert!(env.validate().is_err());
    }

    #[test]
    fn validate_event_type_rejects_control_chars() {
        let mut env = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        env.event_type = "event\x00null".into();
        assert!(env.validate().is_err());

        let mut env2 = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        env2.event_type = "event\ttype".into();
        assert!(env2.validate().is_err());
    }

    #[test]
    fn validate_event_type_accepts_valid_type() {
        let mut env = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        env.event_type = "event.append".into();
        assert!(env.validate().is_ok());

        let mut env2 = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        env2.event_type = "user.login-v2".into();
        assert!(env2.validate().is_ok());

        let mut env3 = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        env3.event_type = "app_event_123".into();
        assert!(env3.validate().is_ok());
    }

    #[test]
    fn validate_rejects_oversized_payload() {
        let mut env = envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        // Create a payload > 1MB
        env.payload = json!({"data": "x".repeat(2_000_000)});
        assert!(env.validate().is_err());
    }
}
