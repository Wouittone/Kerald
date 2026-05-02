use arrow_array::{Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use kerald::{KERALD_CURSOR_FIELD, OpenDalStorage, StorageConfig, StorageError, TimestampCursor, TopicDefinition};
use std::sync::Arc;

const PAYLOAD_SCHEMA_MISMATCH: &str = "payload schema must match topic schema";

#[tokio::test]
async fn polling_undefined_topic_payloads_returns_empty_batches() {
    let temp_dir = tempfile::tempdir().expect("test storage root should be created");
    let storage = OpenDalStorage::local(&StorageConfig::local(temp_dir.path()))
        .await
        .expect("local storage should initialize");
    let topic = order_topic();

    let batches = storage
        .poll_payloads(&topic, TimestampCursor::unix_epoch())
        .await
        .expect("undefined topic dataset should poll as empty");

    assert!(batches.is_empty());
}

#[tokio::test]
async fn payload_append_and_poll_use_strict_nanosecond_timestamp_cursors() {
    let temp_dir = tempfile::tempdir().expect("test storage root should be created");
    let storage = OpenDalStorage::local(&StorageConfig::local(temp_dir.path()))
        .await
        .expect("local storage should initialize");
    let topic = order_topic();

    storage
        .append_payload(&topic, cursor(100), order_batch(["first"]))
        .await
        .expect("first payload should append");
    storage
        .append_payload(&topic, cursor(200), order_batch(["second"]))
        .await
        .expect("second payload should append");

    let all_batches = storage
        .poll_payloads(&topic, TimestampCursor::unix_epoch())
        .await
        .expect("poll from epoch should include all appended payloads");
    assert_eq!(all_batches.len(), 2);
    assert!(all_batches.iter().any(|batch| batch.cursor() == cursor(100)));
    assert!(all_batches.iter().any(|batch| batch.cursor() == cursor(200)));

    let later_batches = storage
        .poll_payloads(&topic, cursor(100))
        .await
        .expect("poll after first cursor should include only later payloads");
    assert_eq!(later_batches.len(), 1);
    assert_eq!(later_batches[0].cursor(), cursor(200));
    assert_eq!(string_value(later_batches[0].payload(), 0, 0), "second");
}

#[tokio::test]
async fn payload_storage_reopens_lance_dataset_through_opendal_path() {
    let temp_dir = tempfile::tempdir().expect("test storage root should be created");
    let config = StorageConfig::local(temp_dir.path());
    let topic = order_topic();

    let storage = OpenDalStorage::local(&config).await.expect("local storage should initialize");
    storage
        .append_payload(&topic, cursor(100), order_batch(["created"]))
        .await
        .expect("payload dataset should be created through OpenDAL");
    storage
        .append_payload(&topic, cursor(200), order_batch(["appended"]))
        .await
        .expect("payload dataset should append through OpenDAL");
    drop(storage);

    let reopened = OpenDalStorage::local(&config)
        .await
        .expect("local storage should reopen existing root");
    let reopened_batches = reopened
        .poll_payloads(&topic, cursor(100))
        .await
        .expect("reopened storage should poll existing Lance dataset");

    assert_eq!(reopened_batches.len(), 1);
    assert_eq!(reopened_batches[0].cursor(), cursor(200));
    assert_eq!(string_value(reopened_batches[0].payload(), 0, 0), "appended");

    reopened
        .append_payload(&topic, cursor(300), order_batch(["after-reopen"]))
        .await
        .expect("reopened storage should append through OpenDAL");
    let appended_after_reopen = reopened
        .poll_payloads(&topic, cursor(200))
        .await
        .expect("reopened append should be pollable");

    assert_eq!(appended_after_reopen.len(), 1);
    assert_eq!(appended_after_reopen[0].cursor(), cursor(300));
    assert_eq!(string_value(appended_after_reopen[0].payload(), 0, 0), "after-reopen");
}

#[tokio::test]
async fn append_rejects_payloads_that_do_not_match_topic_schema() {
    let temp_dir = tempfile::tempdir().expect("test storage root should be created");
    let storage = OpenDalStorage::local(&StorageConfig::local(temp_dir.path()))
        .await
        .expect("local storage should initialize");
    let topic = order_topic();

    let error = storage
        .append_payload(&topic, cursor(100), mismatched_batch())
        .await
        .expect_err("schema mismatch should be rejected");

    assert_eq!(error, StorageError::SchemaMismatch(PAYLOAD_SCHEMA_MISMATCH));
}

#[tokio::test]
async fn append_rejects_topic_schemas_that_use_reserved_cursor_field() {
    let temp_dir = tempfile::tempdir().expect("test storage root should be created");
    let storage = OpenDalStorage::local(&StorageConfig::local(temp_dir.path()))
        .await
        .expect("local storage should initialize");
    let topic = TopicDefinition::new("orders.received", reserved_cursor_schema()).expect("topic name should be valid");

    let error = storage
        .append_payload(&topic, cursor(100), reserved_cursor_batch())
        .await
        .expect_err("reserved cursor field should be rejected");

    assert_eq!(error, StorageError::ReservedFieldName(KERALD_CURSOR_FIELD));
}

fn cursor(nanoseconds_since_epoch: i64) -> TimestampCursor {
    TimestampCursor::try_new(nanoseconds_since_epoch).expect("test cursor should be valid")
}

fn string_value(batch: &RecordBatch, column: usize, row: usize) -> String {
    batch
        .column(column)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("column should contain strings")
        .value(row)
        .to_owned()
}

fn order_topic() -> TopicDefinition {
    TopicDefinition::new("orders.received", order_schema()).expect("topic should be valid")
}

fn order_batch<const N: usize>(ids: [&str; N]) -> RecordBatch {
    RecordBatch::try_new(order_schema(), vec![Arc::new(StringArray::from(Vec::from(ids)))]).expect("test batch should match schema")
}

fn mismatched_batch() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new("order_id", DataType::Int64, false)]));
    RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1]))]).expect("mismatched test batch should build")
}

fn reserved_cursor_batch() -> RecordBatch {
    RecordBatch::try_new(
        reserved_cursor_schema(),
        vec![Arc::new(StringArray::from(vec!["client-owned-cursor"]))],
    )
    .expect("reserved cursor test batch should build")
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("order_id", DataType::Utf8, false)]))
}

fn reserved_cursor_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new(KERALD_CURSOR_FIELD, DataType::Utf8, false)]))
}
