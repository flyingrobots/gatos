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
