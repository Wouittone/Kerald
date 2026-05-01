use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const NEGATIVE_TIMESTAMP_CURSOR: &str = "timestamp cursor must not be negative";
const SYSTEM_TIME_BEFORE_EPOCH: &str = "system time is before the Unix epoch";
const SYSTEM_TIME_OVERFLOW: &str = "system time exceeds supported nanosecond cursor range";

/// Client progress cursor measured as nanoseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimestampCursor(i64);

impl TimestampCursor {
    pub const fn try_new(nanoseconds_since_epoch: i64) -> Result<Self, TimestampCursorError> {
        if nanoseconds_since_epoch < 0 {
            return Err(TimestampCursorError::InvalidNanoseconds(NEGATIVE_TIMESTAMP_CURSOR));
        }

        Ok(Self(nanoseconds_since_epoch))
    }

    pub const fn unix_epoch() -> Self {
        Self(0)
    }

    pub fn now() -> Result<Self, TimestampCursorError> {
        Self::from_system_time(SystemTime::now())
    }

    pub fn from_system_time(value: SystemTime) -> Result<Self, TimestampCursorError> {
        let nanos = value
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TimestampCursorError::InvalidSystemTime(SYSTEM_TIME_BEFORE_EPOCH))?
            .as_nanos();
        let nanos = i64::try_from(nanos).map_err(|_| TimestampCursorError::InvalidSystemTime(SYSTEM_TIME_OVERFLOW))?;

        Ok(Self(nanos))
    }

    pub const fn as_nanos(&self) -> i64 {
        self.0
    }
}

/// Timestamp cursor construction errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimestampCursorError {
    #[error("invalid timestamp cursor: {0}")]
    InvalidNanoseconds(&'static str),
    #[error("invalid timestamp cursor: {0}")]
    InvalidSystemTime(&'static str),
}
