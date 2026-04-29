use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;
use thiserror::Error;

const EMPTY_TOPIC_NAME: &str = "topic name must not be empty";
const INVALID_TOPIC_NAME_CHARACTER: &str = "topic name must contain only ASCII letters, numbers, dots, underscores, or hyphens";
const TOPIC_NAME_TOO_LONG: &str = "topic name must be at most 255 bytes";

/// User-visible topic identifier.
///
/// Topic names identify a single partitionless stream. They deliberately carry
/// no partition count, partition id, shard id, or ownership hint.
pub type TopicName = String;

/// Nanosecond timestamp cursor used for message progress.
pub type TimestampNs = i64;

/// Arrow payload accepted by Kerald's broker path.
pub type MessagePayload = RecordBatch;

pub const TOPIC_NAME_MAX_LEN_BYTES: usize = 255;

pub fn parse_topic_name(value: impl AsRef<str>) -> Result<TopicName, TopicError> {
    let value = value.as_ref().trim();

    if value.is_empty() {
        return Err(TopicError::InvalidName(EMPTY_TOPIC_NAME));
    }

    if value.len() > TOPIC_NAME_MAX_LEN_BYTES {
        return Err(TopicError::InvalidName(TOPIC_NAME_TOO_LONG));
    }

    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(TopicError::InvalidName(INVALID_TOPIC_NAME_CHARACTER));
    }

    Ok(value.to_owned())
}

/// Partitionless topic metadata exposed by the broker model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicDefinition {
    name: TopicName,
    schema: SchemaRef,
}

impl TopicDefinition {
    pub fn new(name: impl AsRef<str>, schema: SchemaRef) -> Result<Self, TopicError> {
        parse_topic_name(name).map(|name| Self { name, schema })
    }

    pub fn name(&self) -> &TopicName {
        &self.name
    }

    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

/// Topic model errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TopicError {
    #[error("invalid topic name: {0}")]
    InvalidName(&'static str),
}

/// Notification progress metadata for an accepted message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageNotification {
    topic: TopicName,
    timestamp_ns: TimestampNs,
    row_count: usize,
}

impl MessageNotification {
    pub fn new(topic: TopicName, timestamp_ns: TimestampNs, row_count: usize) -> Self {
        Self {
            topic,
            timestamp_ns,
            row_count,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn timestamp_ns(&self) -> TimestampNs {
        self.timestamp_ns
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }
}
