use arrow_array::{RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use kerald::{PolledPayloadBatch, PublishReceipt, TimestampCursor, TimestampCursorError};
use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

const NEGATIVE_TIMESTAMP_CURSOR: &str = "timestamp cursor must not be negative";
const SYSTEM_TIME_BEFORE_EPOCH: &str = "system time is before the Unix epoch";

#[test]
fn timestamp_cursor_converts_system_time_to_exact_nanoseconds() {
    let time = UNIX_EPOCH + Duration::new(2, 300);
    let cursor = TimestampCursor::from_system_time(time).expect("time after epoch should convert");

    assert_eq!(cursor.as_nanos(), 2_000_000_300);
}

#[test]
fn timestamp_cursor_rejects_times_before_unix_epoch() {
    let error =
        TimestampCursor::from_system_time(UNIX_EPOCH - Duration::from_nanos(100)).expect_err("time before epoch should be rejected");

    assert_eq!(error, TimestampCursorError::InvalidSystemTime(SYSTEM_TIME_BEFORE_EPOCH));
}

#[test]
fn timestamp_cursor_rejects_negative_nanoseconds() {
    let error = TimestampCursor::try_new(-1).expect_err("negative cursor should be rejected");

    assert_eq!(error, TimestampCursorError::InvalidNanoseconds(NEGATIVE_TIMESTAMP_CURSOR));
}

#[test]
fn timestamp_cursor_orders_by_nanoseconds_without_offsets() {
    let earlier = cursor(100);
    let later = cursor(200);

    assert!(earlier < later);
    assert_eq!(TimestampCursor::unix_epoch().as_nanos(), 0);
}

#[test]
fn publish_receipt_reports_cursor_and_rows() {
    let receipt = PublishReceipt::new(cursor(42), 3);

    assert_eq!(receipt.cursor().as_nanos(), 42);
    assert_eq!(receipt.rows(), 3);
}

#[test]
fn polled_payload_batch_exposes_cursor_and_arrow_payload() {
    let payload = order_batch(["a", "b"]);
    let polled = PolledPayloadBatch::new(cursor(99), payload);

    assert_eq!(polled.cursor().as_nanos(), 99);
    assert_eq!(polled.payload().num_rows(), 2);
    assert_eq!(polled.payload().schema(), order_schema());
}

fn cursor(nanoseconds_since_epoch: i64) -> TimestampCursor {
    TimestampCursor::try_new(nanoseconds_since_epoch).expect("test cursor should be valid")
}

fn order_batch<const N: usize>(ids: [&str; N]) -> RecordBatch {
    RecordBatch::try_new(order_schema(), vec![Arc::new(StringArray::from(Vec::from(ids)))]).expect("test batch should match schema")
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("order_id", DataType::Utf8, false)]))
}
