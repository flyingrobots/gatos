use std::path::PathBuf;

use base64::engine::general_purpose::STANDARD as BASE64_STD;
use base64::Engine;
use gatos_message_plane::{
    validate_ulid_str, CheckpointStore, GitCheckpointStore, GitMessagePublisher,
    GitMessageSubscriber, MessagePlaneError, MessagePublisher, MessageRecord, MessageSubscriber,
    TopicRef,
};

use gatos_message_plane::{MessageEnvelope, PublishReceipt};

const MAX_PAGE_SIZE: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagesReadEntry {
    pub ulid: String,
    pub commit: String,
    pub content_id: String,
    pub envelope_path: String,
    pub canonical_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagesReadResponse {
    pub messages: Vec<MessagesReadEntry>,
    pub next_since: Option<String>,
}

/// Git-backed service wrapper used by the daemon and CLI.
#[allow(dead_code)]
pub struct MessagePlaneService {
    repo: PathBuf,
    publisher: GitMessagePublisher,
    subscriber: GitMessageSubscriber,
    checkpoints: GitCheckpointStore,
}

impl MessagePlaneService {
    pub fn open<P: Into<PathBuf>>(repo: P) -> Result<Self, MessagePlaneError> {
        let repo_path = repo.into();
        Ok(Self {
            publisher: GitMessagePublisher::open(&repo_path)?,
            subscriber: GitMessageSubscriber::open(&repo_path)?,
            checkpoints: GitCheckpointStore::open(&repo_path)?,
            repo: repo_path,
        })
    }

    pub fn max_page_size(&self) -> usize {
        MAX_PAGE_SIZE
    }

    /// Stub entry point for the upcoming RPC server integration.
    pub fn messages_read(
        &self,
        topic: &TopicRef,
        since_ulid: Option<&str>,
        limit: usize,
        checkpoint_group: Option<&str>,
    ) -> Result<MessagesReadResponse, MessagePlaneError> {
        let records = self.read(topic, since_ulid, limit)?;
        let next_since = records.last().map(|r| r.ulid.clone());
        let messages: Vec<MessagesReadEntry> = records
            .into_iter()
            .map(|rec| MessagesReadEntry {
                ulid: rec.ulid,
                commit: rec.commit_id,
                content_id: rec.content_id,
                envelope_path: rec.envelope_path,
                canonical_json: BASE64_STD.encode(rec.canonical_envelope),
            })
            .collect();
        if let (Some(group), Some(last)) = (checkpoint_group, messages.last()) {
            self.checkpoints
                .persist_checkpoint(group, topic, &last.ulid, &last.commit)?;
        }
        Ok(MessagesReadResponse {
            messages,
            next_since,
        })
    }
}

impl MessagePublisher for MessagePlaneService {
    fn publish(
        &self,
        _topic: &TopicRef,
        _envelope: MessageEnvelope,
    ) -> Result<PublishReceipt, MessagePlaneError> {
        Err(MessagePlaneError::Repo(
            "Message Plane publish not implemented (see ADR-0005)".into(),
        ))
    }
}

impl MessageSubscriber for MessagePlaneService {
    fn read(
        &self,
        _topic: &TopicRef,
        since_ulid: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MessageRecord>, MessagePlaneError> {
        if let Some(cursor) = since_ulid {
            validate_ulid_str(cursor)?;
        }
        self.subscriber.read(_topic, since_ulid, limit)
    }
}

impl CheckpointStore for MessagePlaneService {
    fn persist_checkpoint(
        &self,
        _group: &str,
        _topic: &TopicRef,
        ulid: &str,
        _commit: &str,
    ) -> Result<(), MessagePlaneError> {
        validate_ulid_str(ulid)?;
        let _ = &_commit;
        Err(MessagePlaneError::Checkpoint(
            "checkpoint persistence not implemented (see ADR-0005)".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gatos_message_plane::{GitCheckpointStore, GitMessagePublisher, MessageEnvelope};
    use git2::Repository;
    use tempfile::tempdir;

    const ULIDS: [&str; 3] = [
        "01ARZ3NDEKTSV4RRFFQ69G5FCA",
        "01ARZ3NDEKTSV4RRFFQ69G5FCB",
        "01ARZ3NDEKTSV4RRFFQ69G5FCC",
    ];

    #[test]
    fn messages_read_returns_base64_and_next_since() {
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let publisher = GitMessagePublisher::open(dir.path()).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        let env_a = make_envelope(ULIDS[0]);
        let env_b = make_envelope(ULIDS[1]);
        publisher.publish(&topic, env_a.clone()).unwrap();
        publisher.publish(&topic, env_b.clone()).unwrap();

        let service = MessagePlaneService::open(dir.path()).unwrap();
        let resp = service
            .messages_read(&topic, None, 2, None)
            .expect("messages.read");

        assert_eq!(resp.messages.len(), 2);
        assert_eq!(resp.messages[0].ulid, env_a.ulid);
        let decoded = BASE64_STD.decode(&resp.messages[0].canonical_json).unwrap();
        assert_eq!(decoded, env_a.canonical_bytes);
        assert_eq!(resp.next_since, Some(env_b.ulid));
    }

    #[test]
    fn messages_read_respects_since_and_limit() {
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let publisher = GitMessagePublisher::open(dir.path()).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        for ulid in ULIDS {
            publisher.publish(&topic, make_envelope(ulid)).unwrap();
        }

        let service = MessagePlaneService::open(dir.path()).unwrap();
        let resp = service
            .messages_read(&topic, Some(ULIDS[0]), 1, None)
            .expect("messages.read");
        assert_eq!(resp.messages.len(), 1);
        assert_eq!(resp.messages[0].ulid, ULIDS[1]);
        assert_eq!(resp.next_since, Some(ULIDS[1].to_string()));
    }

    #[test]
    fn messages_read_persists_checkpoint_when_group_supplied() {
        let dir = tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let publisher = GitMessagePublisher::open(dir.path()).unwrap();
        let topic = TopicRef::new(dir.path(), "jobs/pending");
        let env_a = make_envelope(ULIDS[0]);
        let env_b = make_envelope(ULIDS[1]);
        publisher.publish(&topic, env_a).unwrap();
        publisher.publish(&topic, env_b.clone()).unwrap();

        let service = MessagePlaneService::open(dir.path()).unwrap();
        let group = "workers";
        let resp = service
            .messages_read(&topic, None, 2, Some(group))
            .expect("messages.read");

        assert_eq!(resp.next_since, Some(env_b.ulid.clone()));
        let store = GitCheckpointStore::open(dir.path()).unwrap();
        let checkpoint = store
            .load_checkpoint(group, &topic)
            .unwrap()
            .expect("checkpoint recorded");
        assert_eq!(checkpoint.ulid, env_b.ulid);
        let last_commit = resp.messages.last().unwrap().commit.clone();
        assert_eq!(checkpoint.commit.as_deref(), Some(last_commit.as_str()));
    }

    fn make_envelope(ulid: &str) -> MessageEnvelope {
        let raw = format!(
            "{{\"ulid\":\"{}\",\"ns\":\"tests\",\"type\":\"demo\",\"payload\":{{}}}}",
            ulid
        );
        MessageEnvelope::from_json_str(&raw).expect("valid envelope")
    }
}
