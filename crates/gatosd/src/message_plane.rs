use gatos_message_plane::{
    validate_ulid_str, CheckpointStore, MessagePlaneError, MessagePublisher, MessageSubscriber,
    TopicRef,
};

use gatos_message_plane::{MessageEnvelope, PublishReceipt};

const MAX_PAGE_SIZE: usize = 512;

/// Placeholder service that will eventually wrap the real Git-backed Message Plane.
pub struct MessagePlaneService;

impl MessagePlaneService {
    pub fn new() -> Self {
        Self
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
    ) -> Result<Vec<PublishReceipt>, MessagePlaneError> {
        self.read(topic, since_ulid, limit)
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
    ) -> Result<Vec<PublishReceipt>, MessagePlaneError> {
        if let Some(cursor) = since_ulid {
            validate_ulid_str(cursor)?;
        }
        let _clamped = limit.clamp(1, MAX_PAGE_SIZE);
        Ok(Vec::new())
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
        Err(MessagePlaneError::Checkpoint(
            "checkpoint persistence not implemented (see ADR-0005)".into(),
        ))
    }
}
