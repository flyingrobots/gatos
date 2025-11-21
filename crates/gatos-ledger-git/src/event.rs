use ed25519_dalek::{SignatureError, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Minimal error type for ledger operations.
#[derive(Debug)]
pub enum LedgerError {
    NotImplemented,
    Signature(SignatureError),
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "not implemented"),
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
        Err(LedgerError::NotImplemented)
    }

    /// Compute CID (dag-cbor + blake3) of canonical bytes.
    pub fn event_cid(&self) -> LedgerResult<String> {
        Err(LedgerError::NotImplemented)
    }
}

/// Sign an event envelope (over canonical bytes).
pub fn sign_event(env: &EventEnvelope, key: &SigningKey) -> LedgerResult<ed25519_dalek::Signature> {
    Err(LedgerError::NotImplemented)
}

/// Verify an event envelope signature.
pub fn verify_event(
    env: &EventEnvelope,
    key: &VerifyingKey,
    sig: &ed25519_dalek::Signature,
) -> LedgerResult<bool> {
    Err(LedgerError::NotImplemented)
}
