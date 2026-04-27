use arrow_array::types::{ArrowPrimitiveType, DurationNanosecondType, TimestampNanosecondType};
use arrow_schema::{DataType, TimeUnit};
use thiserror::Error;

pub type TimestampCursor = <TimestampNanosecondType as ArrowPrimitiveType>::Native;
pub type TimestampCursorRange = <DurationNanosecondType as ArrowPrimitiveType>::Native;

pub const TIMESTAMP_BEFORE_UNIX_EPOCH: &str = "timestamp cursor cannot be before the Unix epoch";
pub const CURSOR_RANGE_IS_REVERSED: &str = "timestamp cursor range start is after end";

pub fn timestamp_cursor_from_epoch_nanos(unix_epoch_nanos: i64) -> Result<TimestampCursor, CursorError> {
    if unix_epoch_nanos < 0 {
        return Err(CursorError::InvalidTimestamp(TIMESTAMP_BEFORE_UNIX_EPOCH));
    }

    Ok(unix_epoch_nanos)
}

pub fn timestamp_cursor_data_type() -> DataType {
    DataType::Timestamp(TimeUnit::Nanosecond, None)
}

pub fn timestamp_cursor_range(start: TimestampCursor, end: TimestampCursor) -> Result<TimestampCursorRange, CursorError> {
    if start > end {
        return Err(CursorError::InvalidRange(CURSOR_RANGE_IS_REVERSED));
    }

    Ok(end - start)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CursorError {
    #[error("invalid timestamp cursor: {0}")]
    InvalidTimestamp(&'static str),

    #[error("invalid timestamp cursor range: {0}")]
    InvalidRange(&'static str),
}
