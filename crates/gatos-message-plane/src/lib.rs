//! GATOS Message Plane — commit-backed message bus primitives.
//!
//! The real transport lives inside `gatosd`, but this crate defines the
//! public-facing types and traits that publishers/subscribers will use once
//! ADR-0005 is fully implemented. Keeping these definitions here lets other
//! crates depend on the semantics without needing the daemon.

use std::path::PathBuf;

use blake3::Hasher;
use serde_json::{Map, Value};

/// Placeholder export so downstream builds keep working while the real API
/// is filled in. Remove once the Message Plane lands.
#[allow(clippy::must_use_candidate)]
pub const fn hello_message_plane() -> &'static str {
    "Hello from gatos-message-plane!"
}

/// Canonical reference to a message topic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicRef {
    /// Repository path housing the `refs/gatos/messages/<topic>` namespace.
    pub repo: PathBuf,
    /// Logical topic name (e.g., `governance`, `jobs/pending`).
    pub name: String,
}

impl TopicRef {
    /// Creates a new topic reference rooted at the provided repository path.
    pub fn new<P: Into<PathBuf>, S: Into<String>>(repo: P, name: S) -> Self {
        Self {
            repo: repo.into(),
            name: name.into(),
        }
    }
}

/// Canonical envelope payload conforming to
/// `schemas/v1/message-plane/event_envelope.schema.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEnvelope {
    /// ULID from the envelope body (used for ordering + dedupe).
    pub ulid: String,
    /// Namespace string (e.g., `governance`).
    pub namespace: String,
    /// Type string (e.g., `proposal.created`).
    pub event_type: String,
    /// Canonical JSON bytes written into `message/envelope.json`.
    pub canonical_bytes: Vec<u8>,
}

impl MessageEnvelope {
    /// Convenience constructor for callers that already produced canonical JSON.
    pub fn new<U: Into<String>, N: Into<String>, T: Into<String>, B: Into<Vec<u8>>>(
        ulid: U,
        namespace: N,
        event_type: T,
        canonical_bytes: B,
    ) -> Self {
        Self {
            ulid: ulid.into(),
            namespace: namespace.into(),
            event_type: event_type.into(),
            canonical_bytes: canonical_bytes.into(),
        }
    }

    /// Build an envelope from raw JSON and canonicalize it.
    pub fn from_json_str(raw: &str) -> Result<Self, MessagePlaneError> {
        let value: Value = serde_json::from_str(raw)
            .map_err(|e| MessagePlaneError::InvalidEnvelope(format!("parse error: {e}")))?;
        Self::from_value(value)
    }

    /// Build an envelope from an already parsed JSON value.
    pub fn from_value(value: Value) -> Result<Self, MessagePlaneError> {
        let ulid = value
            .get("ulid")
            .and_then(Value::as_str)
            .ok_or_else(|| MessagePlaneError::InvalidEnvelope("missing 'ulid'".into()))?;
        validate_ulid_str(ulid)?;
        let namespace = value
            .get("ns")
            .and_then(Value::as_str)
            .ok_or_else(|| MessagePlaneError::InvalidEnvelope("missing 'ns'".into()))?;
        let event_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| MessagePlaneError::InvalidEnvelope("missing 'type'".into()))?;
        if !value.get("payload").is_some() {
            return Err(MessagePlaneError::InvalidEnvelope(
                "missing 'payload'".into(),
            ));
        }
        let canonical = canonicalize_json(value);
        let canonical_bytes = serde_json::to_vec(&canonical)
            .map_err(|e| MessagePlaneError::InvalidEnvelope(format!("serialize error: {e}")))?;
        Ok(Self {
            ulid: ulid.to_string(),
            namespace: namespace.to_string(),
            event_type: event_type.to_string(),
            canonical_bytes,
        })
    }

    /// Returns `blake3:<hex>` digest of the canonical bytes.
    pub fn content_id(&self) -> String {
        let mut hasher = Hasher::new();
        hasher.update(&self.canonical_bytes);
        format!("blake3:{}", hex::encode(hasher.finalize().as_bytes()))
    }
}

/// Result of writing a message commit to the ledger/repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReceipt {
    /// Git commit id (`oid`) of the message commit.
    pub commit_id: String,
    /// Canonical `content_id` (BLAKE3 hex digest of the envelope bytes).
    pub content_id: String,
}

/// Errors encountered during publish/subscribe workflows.
#[derive(Debug)]
pub enum MessagePlaneError {
    /// Repository IO or libgit2 failure.
    Repo(String),
    /// Provided envelope failed schema/canonical validation.
    InvalidEnvelope(String),
    /// CAS violation while appending to a topic.
    HeadConflict,
    /// Subscriber checkpoint could not be stored.
    Checkpoint(String),
}

impl std::fmt::Display for MessagePlaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Repo(e) => write!(f, "repository error: {}", e),
            Self::InvalidEnvelope(e) => write!(f, "invalid envelope: {}", e),
            Self::HeadConflict => write!(f, "topic head moved while publishing"),
            Self::Checkpoint(e) => write!(f, "checkpoint error: {}", e),
        }
    }
}

impl std::error::Error for MessagePlaneError {}

/// Publish interface implemented by the daemon.
pub trait MessagePublisher {
    /// Append a message to `topic`, returning the resulting commit + content ids.
    fn publish(&self, topic: &TopicRef, envelope: MessageEnvelope)
        -> Result<PublishReceipt, MessagePlaneError>;
}

/// Subscriber interface for streaming messages off a topic.
pub trait MessageSubscriber {
    /// Fetch up to `limit` messages newer than `since_ulid`.
    fn read(
        &self,
        topic: &TopicRef,
        since_ulid: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PublishReceipt>, MessagePlaneError>;
}

/// Persistence for consumer checkpoints (refs/gatos/consumers/**).
pub trait CheckpointStore {
    /// Record `ulid`/`commit` as the last-seen event for `topic` and `group`.
    fn persist_checkpoint(
        &self,
        group: &str,
        topic: &TopicRef,
        ulid: &str,
        commit: &str,
    ) -> Result<(), MessagePlaneError>;
}

/// Validates that `input` is a 26-char ULID using uppercase Crockford base32.
pub fn validate_ulid_str(input: &str) -> Result<(), MessagePlaneError> {
    if input.len() != 26 {
        return Err(MessagePlaneError::InvalidEnvelope("ulid must be 26 chars".into()));
    }
    if !input.chars().all(|c| matches!(c, '0'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='T' | 'V'..='Z'))
    {
        return Err(MessagePlaneError::InvalidEnvelope(
            "ulid must be uppercase Crockford base32".into(),
        ));
    }
    Ok(())
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut new_map = Map::with_capacity(entries.len());
            for (k, v) in entries {
                new_map.insert(k, canonicalize_json(v));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}
