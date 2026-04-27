use arrow_schema::{DataType, TimeUnit};
use kerald::{
    CURSOR_RANGE_IS_REVERSED, CursorError, timestamp_cursor_data_type, timestamp_cursor_from_epoch_nanos, timestamp_cursor_range,
};

#[test]
fn cursor_exposes_nanoseconds_since_unix_epoch() {
    let cursor = timestamp_cursor_from_epoch_nanos(1_700_000_000_123_456_789).expect("timestamp should be valid");

    assert_eq!(cursor, 1_700_000_000_123_456_789);
}

#[test]
fn cursor_uses_arrow_timestamp_nanosecond_type() {
    assert_eq!(timestamp_cursor_data_type(), DataType::Timestamp(TimeUnit::Nanosecond, None));
}

#[test]
fn cursor_rejects_negative_arrow_timestamps() {
    let error = timestamp_cursor_from_epoch_nanos(-1).expect_err("pre-epoch timestamps should be rejected");

    assert_eq!(error, CursorError::InvalidTimestamp(kerald::TIMESTAMP_BEFORE_UNIX_EPOCH));
}

#[test]
fn cursors_order_by_timestamp_value() {
    let earlier = timestamp_cursor_from_epoch_nanos(99).expect("timestamp should be valid");
    let later = timestamp_cursor_from_epoch_nanos(100).expect("timestamp should be valid");

    assert!(earlier < later);
}

#[test]
fn cursor_range_is_inclusive() {
    let start = timestamp_cursor_from_epoch_nanos(10).expect("timestamp should be valid");
    let middle = timestamp_cursor_from_epoch_nanos(15).expect("timestamp should be valid");
    let end = timestamp_cursor_from_epoch_nanos(20).expect("timestamp should be valid");

    let range = timestamp_cursor_range(start, end).expect("ascending range should be valid");

    assert_eq!(*range.start(), start);
    assert_eq!(*range.end(), end);
    assert!(range.contains(&start));
    assert!(range.contains(&middle));
    assert!(range.contains(&end));
    assert!(!range.contains(&timestamp_cursor_from_epoch_nanos(21).expect("timestamp should be valid")));
}

#[test]
fn cursor_range_rejects_reversed_bounds() {
    let start = timestamp_cursor_from_epoch_nanos(20).expect("timestamp should be valid");
    let end = timestamp_cursor_from_epoch_nanos(10).expect("timestamp should be valid");

    let error = timestamp_cursor_range(start, end).expect_err("reversed bounds should be rejected");

    assert_eq!(error, CursorError::InvalidRange(CURSOR_RANGE_IS_REVERSED));
}
