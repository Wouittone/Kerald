use arrow_array::{RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use cucumber::{World, given, then, when};
use kerald::{OpenDalStorage, PolledPayloadBatch, StorageConfig, TimestampCursor, TopicDefinition};
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug, Default, World)]
struct PayloadStorageWorld {
    temp_dir: Option<TempDir>,
    storage: Option<OpenDalStorage>,
    topic: Option<TopicDefinition>,
    polled_batches: Vec<PolledPayloadBatch>,
}

#[given("local payload storage is initialized")]
async fn local_payload_storage_is_initialized(world: &mut PayloadStorageWorld) {
    let temp_dir = TempDir::new().expect("scenario storage root should be created");
    let storage = OpenDalStorage::local(&StorageConfig::local(temp_dir.path()))
        .await
        .expect("local storage should initialize");

    world.temp_dir = Some(temp_dir);
    world.storage = Some(storage);
}

#[given(expr = "partitionless topic {string} has payloads at cursors {int} and {int}")]
async fn partitionless_topic_has_payloads_at_cursors(
    world: &mut PayloadStorageWorld,
    topic_name: String,
    first_cursor: i64,
    second_cursor: i64,
) {
    let topic = TopicDefinition::new(topic_name, order_schema()).expect("scenario topic should be valid");
    let storage = world.storage.as_ref().expect("storage should be initialized");

    storage
        .append_payload(&topic, cursor(first_cursor), order_batch(["first"]))
        .await
        .expect("first payload should append");
    storage
        .append_payload(&topic, cursor(second_cursor), order_batch(["second"]))
        .await
        .expect("second payload should append");

    world.topic = Some(topic);
}

#[given(expr = "partitionless topic {string} has no stored payloads")]
async fn partitionless_topic_has_no_stored_payloads(world: &mut PayloadStorageWorld, topic_name: String) {
    world.topic = Some(TopicDefinition::new(topic_name, order_schema()).expect("scenario topic should be valid"));
}

#[when(expr = "payloads are polled after cursor {int}")]
async fn payloads_are_polled_after_cursor(world: &mut PayloadStorageWorld, cursor: i64) {
    let storage = world.storage.as_ref().expect("storage should be initialized");
    let topic = world.topic.as_ref().expect("topic should be configured");

    world.polled_batches = storage
        .poll_payloads(topic, cursor(cursor))
        .await
        .expect("payload polling should succeed");
}

#[then(expr = "only payload cursor {int} is returned")]
async fn only_payload_cursor_is_returned(world: &mut PayloadStorageWorld, cursor: i64) {
    assert_eq!(world.polled_batches.len(), 1);
    assert_eq!(world.polled_batches[0].cursor(), cursor(cursor));
}

#[then("no partition or offset input is required for payload polling")]
async fn no_partition_or_offset_input_is_required(world: &mut PayloadStorageWorld) {
    assert!(world.topic.is_some());
    assert_eq!(world.polled_batches.len(), 1);
}

#[then("no payload batches are returned")]
async fn no_payload_batches_are_returned(world: &mut PayloadStorageWorld) {
    assert!(world.polled_batches.is_empty());
}

fn cursor(nanoseconds_since_epoch: i64) -> TimestampCursor {
    TimestampCursor::try_new(nanoseconds_since_epoch).expect("scenario cursor should be valid")
}

fn order_batch<const N: usize>(ids: [&str; N]) -> RecordBatch {
    RecordBatch::try_new(order_schema(), vec![Arc::new(StringArray::from(Vec::from(ids)))]).expect("scenario batch should match schema")
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("order_id", DataType::Utf8, false)]))
}

#[tokio::main]
async fn main() {
    PayloadStorageWorld::run("tests/cucumber/features/payload_storage.feature").await;
}
