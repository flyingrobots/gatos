//! GATOS Message Plane — commit-backed message bus primitives.
//!
//! The real transport lives inside `gatosd`, but this crate defines the
//! public-facing types and traits that publishers/subscribers will use once
//! ADR-0005 is fully implemented. Keeping these definitions here lets other
//! crates depend on the semantics without needing the daemon.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use blake3::Hasher;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use git2::{Commit, Oid, Repository, Signature, Tree};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Placeholder export so downstream builds keep working while the real API
/// is filled in. Remove once the Message Plane lands.
#[allow(clippy::must_use_candidate)]
pub const fn hello_message_plane() -> &'static str {
    "Hello from gatos-message-plane!"
}

/// Maximum page size enforced by `messages.read` and `MessageSubscriber` implementations.
pub const MAX_PAGE_SIZE: usize = 512;
/// Default path to the canonical envelope blob within a message commit.
pub const DEFAULT_ENVELOPE_PATH: &str = "message/envelope.json";
/// Ref prefixes used across the Message Plane.
pub const REFS_MESSAGES_PREFIX: &str = "refs/gatos/messages";
pub const REFS_CONSUMERS_PREFIX: &str = "refs/gatos/consumers";

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
        Ok(format!(
            "{}/{}/head",
            REFS_MESSAGES_PREFIX,
            self.sanitized_name()?
        ))
    }

    fn segment_prefix(&self, ts: &SegmentTime) -> Result<String, MessagePlaneError> {
        Ok(format!(
            "{}/{:04}/{:02}/{:02}/{:02}",
            self.sanitized_name()?,
            ts.year,
            ts.month,
            ts.day,
            ts.hour
        ))
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
        let ulid = ulid.to_string();
        let namespace = value
            .get("ns")
            .and_then(Value::as_str)
            .ok_or_else(|| MessagePlaneError::InvalidEnvelope("missing 'ns'".into()))?
            .to_string();
        let event_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| MessagePlaneError::InvalidEnvelope("missing 'type'".into()))?
            .to_string();
        if !value.get("payload").is_some() {
            return Err(MessagePlaneError::InvalidEnvelope(
                "missing 'payload'".into(),
            ));
        }
        let canonical = canonicalize_json(value);
        let canonical_bytes = serde_json::to_vec(&canonical)
            .map_err(|e| MessagePlaneError::InvalidEnvelope(format!("serialize error: {e}")))?;
        Ok(Self {
            ulid,
            namespace,
            event_type,
            canonical_bytes,
        })
    }

    /// Returns `blake3:<hex>` digest of the canonical bytes.
    pub fn content_id(&self) -> String {
        compute_content_id(&self.canonical_bytes)
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
    /// Requested topic ref was not found.
    TopicNotFound,
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
            Self::TopicNotFound => write!(f, "topic not found"),
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
    fn publish(
        &self,
        topic: &TopicRef,
        envelope: MessageEnvelope,
    ) -> Result<PublishReceipt, MessagePlaneError>;
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

/// Git-backed implementation for reading messages from a topic.
pub struct GitMessageSubscriber {
    repo: Repository,
}

impl GitMessageSubscriber {
    /// Open a repository at `path` for reads.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MessagePlaneError> {
        let repo = Repository::open(path).map_err(map_git_err)?;
        Ok(Self { repo })
    }

    fn repo(&self) -> &Repository {
        &self.repo
    }

    fn head_commit(&self, topic: &TopicRef) -> Result<Commit<'_>, MessagePlaneError> {
        let head_ref = topic.head_ref()?;
        let oid = self.repo().refname_to_id(&head_ref).map_err(|err| {
            if err.code() == git2::ErrorCode::NotFound {
                MessagePlaneError::TopicNotFound
            } else {
                map_git_err(err)
            }
        })?;
        self.repo().find_commit(oid).map_err(map_git_err)
    }

    fn record_from_commit(&self, commit: &Commit) -> Result<MessageRecord, MessagePlaneError> {
        read_message_record(self.repo(), commit)
    }
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

/// Git-backed implementation storing checkpoints under `refs/gatos/consumers/<group>/<topic>`.
pub struct GitCheckpointStore {
    repo: Repository,
}

impl GitCheckpointStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MessagePlaneError> {
        let repo = Repository::open(path).map_err(map_git_err)?;
        Ok(Self { repo })
    }

    fn ref_name(&self, group: &str, topic: &TopicRef) -> Result<String, MessagePlaneError> {
        Ok(format!(
            "{}/{}/{}",
            REFS_CONSUMERS_PREFIX,
            sanitize_identifier(group)?,
            topic.sanitized_name()?
        ))
    }

    pub fn load_checkpoint(
        &self,
        group: &str,
        topic: &TopicRef,
    ) -> Result<Option<ConsumerCheckpoint>, MessagePlaneError> {
        let refname = self.ref_name(group, topic)?;
        let topic_name = topic.sanitized_name()?;
        match self.repo.find_reference(&refname) {
            Ok(reference) => self.read_checkpoint(reference, group.to_string(), topic_name),
            Err(err) if err.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(err) => Err(map_git_err(err)),
        }
    }

    pub fn list_checkpoints(
        &self,
        topic: &TopicRef,
    ) -> Result<Vec<ConsumerCheckpoint>, MessagePlaneError> {
        let target_topic = topic.sanitized_name()?;
        let mut entries = Vec::new();
        let mut iter = self.repo.references().map_err(map_git_err)?;
        while let Some(reference) = iter.next().transpose().map_err(map_git_err)? {
            let name = match reference.name() {
                Some(n) => n.to_string(),
                None => continue,
            };
            if !name.starts_with("refs/gatos/consumers/") {
                continue;
            }
            if let Some(rest) = name.strip_prefix("refs/gatos/consumers/") {
                let mut parts = rest.splitn(2, '/');
                let group = match parts.next() {
                    Some(g) => g.to_string(),
                    None => continue,
                };
                let topic_part = match parts.next() {
                    Some(t) => t.to_string(),
                    None => continue,
                };
                if topic_part != target_topic {
                    continue;
                }
                if let Some(cp) = self.read_checkpoint(reference, group, topic_part)? {
                    entries.push(cp);
                }
            }
        }
        Ok(entries)
    }

    fn read_checkpoint(
        &self,
        reference: git2::Reference<'_>,
        group: String,
        topic: String,
    ) -> Result<Option<ConsumerCheckpoint>, MessagePlaneError> {
        let oid = match reference.target() {
            Some(oid) => oid,
            None => return Ok(None),
        };
        let blob = self.repo.find_blob(oid).map_err(map_git_err)?;
        let payload: CheckpointPayload = serde_json::from_slice(blob.content())
            .map_err(|e| MessagePlaneError::Checkpoint(e.to_string()))?;
        validate_ulid_str(&payload.ulid)?;
        Ok(Some(ConsumerCheckpoint {
            group,
            topic,
            ulid: payload.ulid,
            commit: payload.commit,
        }))
    }
}

impl CheckpointStore for GitCheckpointStore {
    fn persist_checkpoint(
        &self,
        group: &str,
        topic: &TopicRef,
        ulid: &str,
        commit: &str,
    ) -> Result<(), MessagePlaneError> {
        validate_ulid_str(ulid)?;
        let checkpoint = CheckpointPayload {
            ulid: ulid.to_string(),
            commit: Some(commit.to_string()),
        };
        let json = serde_json::to_string(&checkpoint)
            .map_err(|e| MessagePlaneError::Checkpoint(e.to_string()))?;
        let blob_oid = self.repo.blob(json.as_bytes()).map_err(map_git_err)?;
        let refname = self.ref_name(group, topic)?;
        self.repo
            .reference(&refname, blob_oid, true, "checkpoint update")
            .map(|_| ())
            .map_err(map_git_err)
    }
}

/// Git-backed implementation of [`MessagePublisher`].
pub struct GitMessagePublisher {
    repo: Repository,
    signature: Signature<'static>,
    max_messages_per_segment: u64,
    max_bytes_per_segment: u64,
    clock: Arc<dyn SegmentClock>,
}

impl GitMessagePublisher {
    /// Open a repository at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MessagePlaneError> {
        Self::with_config(path, GitMessagePublisherConfig::default())
    }

    pub fn with_config<P: AsRef<Path>>(
        path: P,
        config: GitMessagePublisherConfig,
    ) -> Result<Self, MessagePlaneError> {
        let repo = Repository::open(path).map_err(map_git_err)?;
        let signature = Signature::now("gatos-message-plane", "message-plane@gatos.local")
            .map_err(map_git_err)?;
        Ok(Self {
            repo,
            signature,
            max_messages_per_segment: config.max_messages_per_segment,
            max_bytes_per_segment: config.max_bytes_per_segment,
            clock: config.clock,
        })
    }

    fn repo(&self) -> &Repository {
        &self.repo
    }

    fn read_head(
        &self,
        topic: &TopicRef,
    ) -> Result<Option<git2::Reference<'_>>, MessagePlaneError> {
        let head_ref = topic.head_ref()?;
        match self.repo().find_reference(&head_ref) {
            Ok(reference) => Ok(Some(reference)),
            Err(err) if err.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(err) => Err(map_git_err(err)),
        }
    }

    fn build_tree(
        &self,
        envelope: &MessageEnvelope,
        meta: &SegmentMeta,
    ) -> Result<Tree<'_>, MessagePlaneError> {
        let repo = self.repo();
        let meta_bytes =
            serde_json::to_vec(meta).map_err(|e| MessagePlaneError::Repo(e.to_string()))?;
        let blob_oid = repo.blob(&envelope.canonical_bytes).map_err(map_git_err)?;
        let mut message_dir = repo.treebuilder(None).map_err(map_git_err)?;
        message_dir
            .insert("envelope.json", blob_oid, 0o100644)
            .map_err(map_git_err)?;
        let message_tree_oid = message_dir.write().map_err(map_git_err)?;
        let meta_blob = repo.blob(&meta_bytes).map_err(map_git_err)?;
        let mut meta_dir = repo.treebuilder(None).map_err(map_git_err)?;
        meta_dir
            .insert("meta.json", meta_blob, 0o100644)
            .map_err(map_git_err)?;
        let meta_tree_oid = meta_dir.write().map_err(map_git_err)?;
        let mut root_builder = repo.treebuilder(None).map_err(map_git_err)?;
        const TREE_MODE: i32 = 0o040000;
        root_builder
            .insert("message", message_tree_oid, TREE_MODE)
            .map_err(map_git_err)?;
        root_builder
            .insert("meta", meta_tree_oid, TREE_MODE)
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
        segment_desc: &str,
    ) -> Result<(), MessagePlaneError> {
        let head_ref = topic.head_ref()?;
        match expected_old {
            Some(old) => self
                .repo()
                .reference_matching(&head_ref, new_oid, true, old, segment_desc)
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
                .reference(&head_ref, new_oid, true, segment_desc)
                .map(|_| ())
                .map_err(map_git_err),
        }
    }

    fn update_segment_ref(
        &self,
        segment_ref: &str,
        new_oid: Oid,
        expected_old: Option<Oid>,
    ) -> Result<(), MessagePlaneError> {
        match expected_old {
            Some(old) => self
                .repo()
                .reference_matching(segment_ref, new_oid, true, old, "segment append")
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
                .reference(segment_ref, new_oid, true, "segment init")
                .map(|_| ())
                .map_err(map_git_err),
        }
    }

    fn read_segment_meta(&self, commit: &Commit) -> Option<SegmentMeta> {
        read_segment_meta(self.repo(), commit)
    }

    fn derive_segment_meta(
        &self,
        topic: &TopicRef,
        now: DateTime<Utc>,
        payload_len: usize,
        previous: Option<&SegmentMeta>,
        envelope_ulid: &str,
    ) -> Result<(SegmentMeta, bool), MessagePlaneError> {
        let prefix = topic.segment_prefix(&SegmentTime::from_datetime(&now))?;
        if let Some(prev) = previous {
            if !self.should_rotate(prev, &prefix, payload_len) {
                let mut updated = prev.clone();
                updated.message_count += 1;
                updated.approximate_bytes += payload_len as u64;
                return Ok((updated, true));
            }
        }
        Ok((
            SegmentMeta::new(
                prefix,
                envelope_ulid.to_string(),
                now.timestamp(),
                payload_len as u64,
            ),
            false,
        ))
    }

    fn should_rotate(&self, existing: &SegmentMeta, new_prefix: &str, payload_len: usize) -> bool {
        if existing.segment_prefix != new_prefix {
            return true;
        }
        if existing.message_count >= self.max_messages_per_segment {
            return true;
        }
        existing.approximate_bytes + payload_len as u64 > self.max_bytes_per_segment
    }
}

impl MessageSubscriber for GitMessageSubscriber {
    fn read(
        &self,
        topic: &TopicRef,
        since_ulid: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MessageRecord>, MessagePlaneError> {
        if let Some(cursor) = since_ulid {
            validate_ulid_str(cursor)?;
        }
        if limit == 0 {
            return Err(MessagePlaneError::InvalidLimit);
        }
        let limit = limit.min(MAX_PAGE_SIZE);

        let mut commit = self.head_commit(topic)?;
        let mut records_desc = Vec::new();
        loop {
            records_desc.push(self.record_from_commit(&commit)?);
            if commit.parent_count() == 0 {
                break;
            }
            commit = commit.parent(0).map_err(map_git_err)?;
        }

        let slice = if let Some(cursor) = since_ulid {
            if let Some(pos) = records_desc.iter().position(|r| r.ulid == cursor) {
                &records_desc[..pos]
            } else {
                &records_desc[..]
            }
        } else {
            &records_desc[..]
        };

        let mut ordered: Vec<MessageRecord> = slice.iter().rev().cloned().collect();
        if ordered.len() > limit {
            ordered.truncate(limit);
        }
        Ok(ordered)
    }
}

/// Determines which segments are eligible for TTL pruning.
pub struct SegmentPruner {
    repo: Repository,
}

impl SegmentPruner {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MessagePlaneError> {
        let repo = Repository::open(path).map_err(map_git_err)?;
        Ok(Self { repo })
    }

    pub fn find_prunable_segments(
        &self,
        topic: &TopicRef,
        now: DateTime<Utc>,
        retention: Duration,
        checkpoints: &[ConsumerCheckpoint],
    ) -> Result<Vec<String>, MessagePlaneError> {
        let topic_name = topic.sanitized_name()?;
        let prefix = format!("refs/gatos/messages/{}/", topic_name);
        let mut refs = self.repo.references().map_err(map_git_err)?;
        let mut prunable = Vec::new();
        while let Some(reference) = refs.next().transpose().map_err(map_git_err)? {
            let name = match reference.name() {
                Some(n) => n.to_string(),
                None => continue,
            };
            if !name.starts_with(&prefix) || name.ends_with("/head") {
                continue;
            }
            let oid = match reference.target() {
                Some(oid) => oid,
                None => continue,
            };
            let commit = self.repo.find_commit(oid).map_err(map_git_err)?;
            let meta = match read_segment_meta(&self.repo, &commit) {
                Some(m) => m,
                None => continue,
            };
            let age = now.timestamp() - meta.started_at_epoch;
            if age < retention.num_seconds() {
                continue;
            }
            let record = read_message_record(&self.repo, &commit)?;
            if checkpoints
                .iter()
                .filter(|cp| cp.topic == topic_name)
                .any(|cp| cp.ulid < record.ulid)
            {
                continue;
            }
            prunable.push(name);
        }
        Ok(prunable)
    }

    pub fn prune(
        &self,
        topic: &TopicRef,
        now: DateTime<Utc>,
        retention: Duration,
        checkpoints: &[ConsumerCheckpoint],
    ) -> Result<Vec<String>, MessagePlaneError> {
        let candidates = self.find_prunable_segments(topic, now, retention, checkpoints)?;
        for refname in &candidates {
            if let Ok(mut r) = self.repo.find_reference(refname) {
                r.delete().map_err(map_git_err)?;
            }
        }
        Ok(candidates)
    }
}

impl MessagePublisher for GitMessagePublisher {
    fn publish(
        &self,
        topic: &TopicRef,
        envelope: MessageEnvelope,
    ) -> Result<PublishReceipt, MessagePlaneError> {
        let now = self.clock.now();
        let payload_len = envelope.canonical_bytes.len();
        let head_ref = self.read_head(topic)?;
        let head_oid = head_ref.as_ref().and_then(|r| r.target());
        let parent_commit = if let Some(oid) = head_oid {
            Some(self.repo().find_commit(oid).map_err(map_git_err)?)
        } else {
            None
        };
        let previous_meta = parent_commit
            .as_ref()
            .and_then(|commit| self.read_segment_meta(commit));
        let (segment_meta, continuing) = self.derive_segment_meta(
            topic,
            now,
            payload_len,
            previous_meta.as_ref(),
            &envelope.ulid,
        )?;
        let tree = self.build_tree(&envelope, &segment_meta)?;
        let parent_refs: Vec<&Commit<'_>> = parent_commit.iter().collect();
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
        let segment_path = segment_meta.segment_path();
        let segment_ref = format!("refs/gatos/messages/{}", segment_path);
        let old_segment_oid = if continuing { head_oid } else { None };
        self.update_segment_ref(&segment_ref, commit_oid, old_segment_oid)?;
        self.update_head(topic, commit_oid, head_oid, &segment_path)?;
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
        return Err(MessagePlaneError::InvalidEnvelope(
            "ulid must be 26 chars".into(),
        ));
    }
    if !input
        .chars()
        .all(|c| matches!(c, '0'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='T' | 'V'..='Z'))
    {
        return Err(MessagePlaneError::InvalidEnvelope(
            "ulid must be uppercase Crockford base32".into(),
        ));
    }
    Ok(())
}

fn compute_content_id(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    format!("blake3:{}", hex::encode(hasher.finalize().as_bytes()))
}

fn read_message_record(
    repo: &Repository,
    commit: &Commit,
) -> Result<MessageRecord, MessagePlaneError> {
    let tree = commit.tree().map_err(map_git_err)?;
    let message_tree = tree
        .get_name("message")
        .ok_or_else(|| MessagePlaneError::Repo("message tree missing".into()))?;
    let message_tree = repo.find_tree(message_tree.id()).map_err(map_git_err)?;
    let envelope = message_tree
        .get_name("envelope.json")
        .ok_or_else(|| MessagePlaneError::Repo("envelope.json missing".into()))?;
    let blob = repo.find_blob(envelope.id()).map_err(map_git_err)?;
    let canonical = blob.content().to_vec();
    let envelope_json: Value = serde_json::from_slice(&canonical)
        .map_err(|e| MessagePlaneError::Repo(format!("parse envelope: {e}")))?;
    let ulid = envelope_json
        .get("ulid")
        .and_then(Value::as_str)
        .ok_or_else(|| MessagePlaneError::Repo("envelope missing ulid".into()))?
        .to_string();
    validate_ulid_str(&ulid)?;

    Ok(MessageRecord {
        commit_id: commit.id().to_string(),
        content_id: compute_content_id(&canonical),
        envelope_path: DEFAULT_ENVELOPE_PATH.to_string(),
        canonical_envelope: canonical,
        ulid,
    })
}

fn read_segment_meta(repo: &Repository, commit: &Commit) -> Option<SegmentMeta> {
    let tree = commit.tree().ok()?;
    let meta_tree_entry = tree.get_name("meta")?;
    let meta_tree = repo.find_tree(meta_tree_entry.id()).ok()?;
    let meta_blob_entry = meta_tree.get_name("meta.json")?;
    let blob = repo.find_blob(meta_blob_entry.id()).ok()?;
    serde_json::from_slice(blob.content()).ok()
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

fn sanitize_identifier(input: &str) -> Result<String, MessagePlaneError> {
    if input.is_empty()
        || !input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/'))
    {
        return Err(MessagePlaneError::InvalidTopic(input.into()));
    }
    Ok(input.to_string())
}

fn map_git_err(err: git2::Error) -> MessagePlaneError {
    MessagePlaneError::Repo(err.to_string())
}

pub(crate) trait SegmentClock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

struct SystemClock;

impl SegmentClock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Clone)]
pub struct GitMessagePublisherConfig {
    pub max_messages_per_segment: u64,
    pub max_bytes_per_segment: u64,
    pub(crate) clock: Arc<dyn SegmentClock>,
}

impl Default for GitMessagePublisherConfig {
    fn default() -> Self {
        Self {
            max_messages_per_segment: 100_000,
            max_bytes_per_segment: 192 * 1024 * 1024,
            clock: Arc::new(SystemClock),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SegmentMeta {
    version: u32,
    segment_prefix: String,
    segment_ulid: String,
    started_at_epoch: i64,
    message_count: u64,
    approximate_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CheckpointPayload {
    ulid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerCheckpoint {
    pub group: String,
    pub topic: String,
    pub ulid: String,
    pub commit: Option<String>,
}

impl SegmentMeta {
    fn new(
        segment_prefix: String,
        segment_ulid: String,
        started_at_epoch: i64,
        first_bytes: u64,
    ) -> Self {
        Self {
            version: 1,
            segment_prefix,
            segment_ulid,
            started_at_epoch,
            message_count: 1,
            approximate_bytes: first_bytes,
        }
    }

    fn segment_path(&self) -> String {
        format!("{}/{}", self.segment_prefix, self.segment_ulid)
    }
}

#[derive(Debug, Clone)]
struct SegmentTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
}

impl SegmentTime {
    fn from_datetime(dt: &DateTime<Utc>) -> Self {
        Self {
            year: dt.year(),
            month: dt.month(),
            day: dt.day(),
            hour: dt.hour(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    const GOOD_ULID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

    fn require_docker() {
        assert_eq!(
            std::env::var("GATOS_TEST_IN_DOCKER").as_deref(),
            Ok("1"),
            "Tests must run inside the Docker harness (set GATOS_TEST_IN_DOCKER=1)",
        );
    }

    #[test]
    fn ulid_validation_accepts_uppercase_crockford() {
        require_docker();
        assert!(validate_ulid_str(GOOD_ULID).is_ok());
    }

    #[test]
    fn ulid_validation_rejects_bad_values() {
        require_docker();
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
        require_docker();
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
        require_docker();
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
        require_docker();
        assert_eq!(sanitize_topic("jobs/pending").unwrap(), "jobs/pending");
        assert!(sanitize_topic("../evil").is_err());
        assert!(sanitize_topic("foo//bar").is_err());
        assert!(sanitize_topic("bad*segment").is_err());
    }

    #[test]
    fn checkpoint_store_writes_blob_ref() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let store = GitCheckpointStore::open(dir.path()).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        store
            .persist_checkpoint("workers", &topic, GOOD_ULID, "abc123")
            .unwrap();

        let repo = Repository::open(dir.path()).unwrap();
        let refname = "refs/gatos/consumers/workers/jobs/pending";
        let blob_oid = repo.refname_to_id(refname).unwrap();
        let blob = repo.find_blob(blob_oid).unwrap();
        let checkpoint: serde_json::Value = serde_json::from_slice(blob.content()).unwrap();
        assert_eq!(checkpoint.get("ulid").unwrap().as_str().unwrap(), GOOD_ULID);
        assert_eq!(
            checkpoint.get("commit").unwrap().as_str().unwrap(),
            "abc123"
        );
        assert_eq!(checkpoint.as_object().unwrap().len(), 2);

        let loaded = store
            .load_checkpoint("workers", &topic)
            .unwrap()
            .expect("checkpoint exists");
        assert_eq!(loaded.group, "workers");
        assert_eq!(loaded.topic, "jobs/pending");
        assert_eq!(loaded.ulid, GOOD_ULID);
        assert_eq!(loaded.commit.as_deref(), Some("abc123"));
    }

    #[test]
    fn rotates_when_hour_changes() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let clock = Arc::new(MockClock::new(vec![
            dt(2025, 11, 20, 13),
            dt(2025, 11, 20, 13),
            dt(2025, 11, 20, 14),
        ]));
        let config = GitMessagePublisherConfig {
            clock,
            ..Default::default()
        };
        let publisher = GitMessagePublisher::with_config(dir.path(), config).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        publish(&publisher, &topic, "01ARZ3NDEKTSV4RRFFQ69G5FBA");
        publish(&publisher, &topic, "01ARZ3NDEKTSV4RRFFQ69G5FBB");
        publish(&publisher, &topic, "01ARZ3NDEKTSV4RRFFQ69G5FBC");

        let repo = Repository::open(dir.path()).unwrap();
        let head_oid = repo
            .refname_to_id("refs/gatos/messages/jobs/pending/head")
            .unwrap();
        let head_commit = repo.find_commit(head_oid).unwrap();
        let latest_meta = read_meta(&repo, &head_commit);
        assert_eq!(latest_meta.message_count, 1);
        assert!(latest_meta.segment_prefix.ends_with("/14"));

        let second_commit = head_commit.parent(0).unwrap();
        let second_meta = read_meta(&repo, &second_commit);
        assert_eq!(second_meta.message_count, 2);
        assert!(second_meta.segment_prefix.ends_with("/13"));
        assert_ne!(second_meta.segment_prefix, latest_meta.segment_prefix);

        assert_ref_points_to(
            &repo,
            format!("refs/gatos/messages/{}", second_meta.segment_path()),
            second_commit.id(),
        );
        assert_ref_points_to(
            &repo,
            format!("refs/gatos/messages/{}", latest_meta.segment_path()),
            head_commit.id(),
        );
    }

    #[test]
    fn rotates_when_message_limit_exceeded() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let clock = Arc::new(MockClock::new(vec![
            dt(2025, 11, 20, 10),
            dt(2025, 11, 20, 10),
            dt(2025, 11, 20, 10),
        ]));
        let config = GitMessagePublisherConfig {
            max_messages_per_segment: 2,
            clock,
            ..Default::default()
        };
        let publisher = GitMessagePublisher::with_config(dir.path(), config).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        publish(&publisher, &topic, "01ARZ3NDEKTSV4RRFFQ69G5FBD");
        publish(&publisher, &topic, "01ARZ3NDEKTSV4RRFFQ69G5FBE");
        publish(&publisher, &topic, "01ARZ3NDEKTSV4RRFFQ69G5FBF");

        let repo = Repository::open(dir.path()).unwrap();
        let head_oid = repo
            .refname_to_id("refs/gatos/messages/jobs/pending/head")
            .unwrap();
        let head_commit = repo.find_commit(head_oid).unwrap();
        let latest_meta = read_meta(&repo, &head_commit);
        assert_eq!(latest_meta.message_count, 1);
        let second_commit = head_commit.parent(0).unwrap();
        let second_meta = read_meta(&repo, &second_commit);
        assert_eq!(second_meta.message_count, 2);
        assert_eq!(second_meta.segment_prefix, latest_meta.segment_prefix);
        assert_ne!(second_meta.segment_ulid, latest_meta.segment_ulid);
    }

    #[test]
    fn subscriber_reads_canonical_json_in_order() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let publisher = GitMessagePublisher::open(dir.path()).unwrap();
        let subscriber = GitMessageSubscriber::open(dir.path()).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");

        let ulids = [
            "01ARZ3NDEKTSV4RRFFQ69G5FBG",
            "01ARZ3NDEKTSV4RRFFQ69G5FBH",
            "01ARZ3NDEKTSV4RRFFQ69G5FBJ",
        ];
        let envelopes: Vec<_> = ulids.iter().map(|u| make_envelope(u)).collect();
        for env in &envelopes {
            publisher.publish(&topic, env.clone()).unwrap();
        }

        let records = subscriber.read(&topic, None, 10).unwrap();
        assert_eq!(records.len(), envelopes.len());
        let returned_ulids: Vec<_> = records.iter().map(|r| r.ulid.as_str()).collect();
        assert_eq!(returned_ulids, ulids);
        for (record, env) in records.iter().zip(envelopes.iter()) {
            assert_eq!(record.envelope_path, "message/envelope.json");
            assert_eq!(record.canonical_envelope, env.canonical_bytes);
            assert_eq!(record.content_id, env.content_id());
        }
    }

    #[test]
    fn subscriber_respects_since_ulid_and_limit() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let publisher = GitMessagePublisher::open(dir.path()).unwrap();
        let subscriber = GitMessageSubscriber::open(dir.path()).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");

        let ulids = [
            "01ARZ3NDEKTSV4RRFFQ69G5FBK",
            "01ARZ3NDEKTSV4RRFFQ69G5FBL",
            "01ARZ3NDEKTSV4RRFFQ69G5FBM",
            "01ARZ3NDEKTSV4RRFFQ69G5FBN",
        ];
        for u in &ulids {
            publish(&publisher, &topic, u);
        }

        let records = subscriber
            .read(&topic, Some(ulids[1]), 2)
            .expect("read succeeds");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].ulid, ulids[2]);
        assert_eq!(records[1].ulid, ulids[3]);

        let limited = subscriber.read(&topic, None, 1).expect("limit clamped");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].ulid, ulids[0]);
    }

    #[test]
    fn pruner_skips_when_checkpoint_lags() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let clock = Arc::new(MockClock::new(vec![
            dt(2025, 11, 20, 9),
            dt(2025, 11, 20, 9),
            dt(2025, 11, 20, 11),
        ]));
        let config = GitMessagePublisherConfig {
            clock,
            ..Default::default()
        };
        let publisher = GitMessagePublisher::with_config(dir.path(), config).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        let ulids = [
            "01ARZ3NDEKTSV4RRFFQ69G5FBD",
            "01ARZ3NDEKTSV4RRFFQ69G5FBE",
            "01ARZ3NDEKTSV4RRFFQ69G5FBF",
        ];
        for u in &ulids {
            publish(&publisher, &topic, u);
        }

        let store = GitCheckpointStore::open(dir.path()).unwrap();
        // Checkpoint behind the last ULID in the first segment -> should block pruning.
        store
            .persist_checkpoint("workers", &topic, ulids[0], "deadbeef")
            .unwrap();
        let checkpoints = store.list_checkpoints(&topic).unwrap();
        let pruner = SegmentPruner::open(dir.path()).unwrap();
        let candidates = pruner
            .find_prunable_segments(
                &topic,
                dt(2025, 11, 20, 13),
                Duration::hours(3),
                &checkpoints,
            )
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn pruner_allows_old_segment_once_checkpoints_advance() {
        require_docker();
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let clock = Arc::new(MockClock::new(vec![
            dt(2025, 11, 20, 9),
            dt(2025, 11, 20, 9),
            dt(2025, 11, 20, 11),
        ]));
        let config = GitMessagePublisherConfig {
            clock,
            ..Default::default()
        };
        let publisher = GitMessagePublisher::with_config(dir.path(), config).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        let ulids = [
            "01ARZ3NDEKTSV4RRFFQ69G5FBG",
            "01ARZ3NDEKTSV4RRFFQ69G5FBH",
            "01ARZ3NDEKTSV4RRFFQ69G5FBJ",
        ];
        for u in &ulids {
            publish(&publisher, &topic, u);
        }

        let store = GitCheckpointStore::open(dir.path()).unwrap();
        // Advance checkpoint past the first segment's last ULID.
        store
            .persist_checkpoint("workers", &topic, ulids[2], "cafebabe")
            .unwrap();
        let checkpoints = store.list_checkpoints(&topic).unwrap();
        let pruner = SegmentPruner::open(dir.path()).unwrap();
        let candidates = pruner
            .find_prunable_segments(
                &topic,
                dt(2025, 11, 20, 13),
                Duration::hours(3),
                &checkpoints,
            )
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].contains(ulids[0]));
        // Perform the prune and ensure the ref disappears.
        let deleted = pruner
            .prune(
                &topic,
                dt(2025, 11, 20, 13),
                Duration::hours(3),
                &checkpoints,
            )
            .unwrap();
        assert_eq!(deleted, candidates);
        let repo = Repository::open(dir.path()).unwrap();
        assert!(repo.find_reference(&deleted[0]).is_err());
    }

    fn publish(publisher: &GitMessagePublisher, topic: &TopicRef, ulid: &str) {
        publisher
            .publish(topic, make_envelope(ulid))
            .expect("publish");
    }

    fn make_envelope(ulid: &str) -> MessageEnvelope {
        let raw = format!(
            "{{\"ulid\":\"{}\",\"ns\":\"tests\",\"type\":\"demo\",\"payload\":{{}}}}",
            ulid
        );
        MessageEnvelope::from_json_str(&raw).expect("valid envelope")
    }

    fn dt(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
    }

    fn read_meta(repo: &Repository, commit: &Commit) -> SegmentMeta {
        let tree = commit.tree().unwrap();
        let meta_tree = tree.get_name("meta").unwrap();
        let meta_tree = repo.find_tree(meta_tree.id()).unwrap();
        let meta_blob = meta_tree.get_name("meta.json").unwrap();
        let blob = repo.find_blob(meta_blob.id()).unwrap();
        serde_json::from_slice(blob.content()).unwrap()
    }

    fn assert_ref_points_to(repo: &Repository, refname: String, expected: Oid) {
        let oid = repo.refname_to_id(&refname).unwrap();
        assert_eq!(oid, expected);
    }

    struct MockClock {
        times: Mutex<Vec<DateTime<Utc>>>,
    }

    impl MockClock {
        fn new(times: Vec<DateTime<Utc>>) -> Self {
            Self {
                times: Mutex::new(times),
            }
        }
    }

    impl SegmentClock for MockClock {
        fn now(&self) -> DateTime<Utc> {
            let mut guard = self.times.lock().unwrap();
            guard.remove(0)
        }
    }
}
