//! GATOS Message Plane — commit-backed message bus primitives.
//!
//! The real transport lives inside `gatosd`, but this crate defines the
//! public-facing types and traits that publishers/subscribers will use once
//! ADR-0005 is fully implemented. Keeping these definitions here lets other
//! crates depend on the semantics without needing the daemon.

use std::path::{Path, PathBuf};

use blake3::Hasher;
use git2::{Commit, FileMode, Oid, Repository, Signature, Tree};
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

    fn sanitized_name(&self) -> Result<String, MessagePlaneError> {
        sanitize_topic(&self.name)
    }

    pub fn head_ref(&self) -> Result<String, MessagePlaneError> {
        Ok(format!("refs/gatos/messages/{}/head", self.sanitized_name()?))
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
    /// ULID supplied in the envelope.
    pub ulid: String,
}

/// Full record returned by `MessageSubscriber::read`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    /// Commit hash containing the message.
    pub commit_id: String,
    /// Canonical `content_id` (BLAKE3 digest of the envelope).
    pub content_id: String,
    /// Path to `message/envelope.json` inside the commit tree.
    pub envelope_path: String,
    /// Canonical JSON bytes (base64 when serialized for RPC).
    pub canonical_envelope: Vec<u8>,
    /// ULID used for ordering/dedupe.
    pub ulid: String,
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
    /// Client supplied an invalid range/limit.
    InvalidLimit,
    /// Topic name used invalid characters.
    InvalidTopic(String),
}

impl std::fmt::Display for MessagePlaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Repo(e) => write!(f, "repository error: {}", e),
            Self::InvalidEnvelope(e) => write!(f, "invalid envelope: {}", e),
            Self::HeadConflict => write!(f, "topic head moved while publishing"),
            Self::Checkpoint(e) => write!(f, "checkpoint error: {}", e),
            Self::InvalidLimit => write!(f, "invalid range/limit"),
            Self::InvalidTopic(t) => write!(f, "invalid topic name: {t}"),
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
    ) -> Result<Vec<MessageRecord>, MessagePlaneError>;
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

/// Git-backed implementation of [`MessagePublisher`].
pub struct GitMessagePublisher {
    repo: Repository,
    signature: Signature<'static>,
}

impl GitMessagePublisher {
    /// Open a repository at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MessagePlaneError> {
        let repo = Repository::open(path).map_err(map_git_err)?;
        let signature = Signature::now("gatos-message-plane", "message-plane@gatos.local")
            .map_err(map_git_err)?;
        Ok(Self { repo, signature })
    }

    fn repo(&self) -> &Repository {
        &self.repo
    }

    fn read_head_oid(&self, topic: &TopicRef) -> Result<Option<Oid>, MessagePlaneError> {
        match topic.head_ref().and_then(|head| {
            match self.repo().find_reference(&head) {
                Ok(reference) => Ok(reference.target()),
                Err(err) if err.code() == git2::ErrorCode::NotFound => Ok(None),
                Err(err) => Err(map_git_err(err)),
            }
        }) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    fn build_tree(&self, envelope: &MessageEnvelope) -> Result<Tree<'_>, MessagePlaneError> {
        let repo = self.repo();
        let blob_oid = repo
            .blob(&envelope.canonical_bytes)
            .map_err(map_git_err)?;
        let mut message_dir = repo.treebuilder(None).map_err(map_git_err)?;
        message_dir
            .insert("envelope.json", blob_oid, 0o100644)
            .map_err(map_git_err)?;
        let message_tree_oid = message_dir.write().map_err(map_git_err)?;
        let mut root_builder = repo.treebuilder(None).map_err(map_git_err)?;
        root_builder
            .insert("message", message_tree_oid, git2::FileMode::Tree as u32)
            .map_err(map_git_err)?;
        let tree_oid = root_builder.write().map_err(map_git_err)?;
        repo.find_tree(tree_oid).map_err(map_git_err)
    }

    fn build_commit_message(envelope: &MessageEnvelope) -> String {
        format!(
            "{}\n\nEvent-Id: ulid:{}\nContent-Id: {}\n",
            envelope.event_type,
            envelope.ulid,
            envelope.content_id()
        )
    }

    fn update_head(
        &self,
        topic: &TopicRef,
        new_oid: Oid,
        expected_old: Option<Oid>,
    ) -> Result<(), MessagePlaneError> {
        let head_ref = topic.head_ref()?;
        match expected_old {
            Some(old) => self
                .repo()
                .reference_matching(&head_ref, new_oid, true, old)
                .map(|_| ())
                .map_err(|err| {
                    if err.code() == git2::ErrorCode::NotFound {
                        MessagePlaneError::HeadConflict
                    } else {
                        map_git_err(err)
                    }
                }),
            None => self
                .repo()
                .reference(&head_ref, new_oid, true, "init message topic")
                .map(|_| ())
                .map_err(map_git_err),
        }
    }
}

impl MessagePublisher for GitMessagePublisher {
    fn publish(
        &self,
        topic: &TopicRef,
        envelope: MessageEnvelope,
    ) -> Result<PublishReceipt, MessagePlaneError> {
        let tree = self.build_tree(&envelope)?;
        let head_oid = self.read_head_oid(topic)?;
        let mut parent_commits = Vec::new();
        if let Some(oid) = head_oid {
            parent_commits.push(
                self.repo()
                    .find_commit(oid)
                    .map_err(map_git_err)?,
            );
        }
        let parent_refs: Vec<&Commit<'_>> = parent_commits.iter().collect();
        let message = Self::build_commit_message(&envelope);
        let commit_oid = self
            .repo()
            .commit(
                None,
                &self.signature,
                &self.signature,
                &message,
                &tree,
                &parent_refs,
            )
            .map_err(map_git_err)?;
        self.update_head(topic, commit_oid, head_oid)?;
        Ok(PublishReceipt {
            commit_id: commit_oid.to_string(),
            content_id: envelope.content_id(),
            ulid: envelope.ulid.clone(),
        })
    }
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

fn sanitize_topic(input: &str) -> Result<String, MessagePlaneError> {
    if input.is_empty() {
        return Err(MessagePlaneError::InvalidTopic("empty".into()));
    }
    let mut normalized = Vec::new();
    for segment in input.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || !segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Err(MessagePlaneError::InvalidTopic(input.into()));
        }
        normalized.push(segment);
    }
    Ok(normalized.join("/"))
}

fn map_git_err(err: git2::Error) -> MessagePlaneError {
    MessagePlaneError::Repo(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const GOOD_ULID: &str = "01HAF6ZZZ8Q1EXAMPLEFUNAAA";

    #[test]
    fn ulid_validation_accepts_uppercase_crockford() {
        assert!(validate_ulid_str(GOOD_ULID).is_ok());
    }

    #[test]
    fn ulid_validation_rejects_bad_values() {
        assert!(matches!(
            validate_ulid_str("short"),
            Err(MessagePlaneError::InvalidEnvelope(_))
        ));
        assert!(matches!(
            validate_ulid_str("01HAF6zzzzzzzzzzzzzzzzzzz"),
            Err(MessagePlaneError::InvalidEnvelope(_))
        ));
    }

    #[test]
    fn envelope_canonicalization_sorts_keys() {
        let envelope_json = json!({
            "payload": {"b": 1, "a": 2},
            "type": "demo",
            "ns": "tests",
            "ulid": GOOD_ULID,
            "refs": {"x": "blake3:1234"}
        });
        let envelope = MessageEnvelope::from_value(envelope_json).expect("valid envelope");
        let canonical_str = String::from_utf8(envelope.canonical_bytes.clone()).unwrap();
        assert!(canonical_str.find("\"ns\"") < canonical_str.find("\"payload\""));
        assert_eq!(
            envelope.content_id(),
            envelope.content_id(),
            "content id should be deterministic"
        );
    }

    #[test]
    fn envelope_requires_payload() {
        let broken = json!({
            "type": "demo",
            "ns": "tests",
            "ulid": GOOD_ULID
        });
        assert!(matches!(
            MessageEnvelope::from_value(broken),
            Err(MessagePlaneError::InvalidEnvelope(_))
        ));
    }

    #[test]
    fn sanitize_topic_checks_segments() {
        assert_eq!(sanitize_topic("jobs/pending").unwrap(), "jobs/pending");
        assert!(sanitize_topic("../evil").is_err());
        assert!(sanitize_topic("foo//bar").is_err());
        assert!(sanitize_topic("bad*segment").is_err());
    }
}
