use crate::TimestampCursor;
use arrow_array::RecordBatch;

/// Successful write admission and durable payload append result.
#[derive(Debug, Clone)]
pub struct PublishReceipt {
    cursor: TimestampCursor,
    rows: usize,
}

impl PublishReceipt {
    pub fn new(cursor: TimestampCursor, rows: usize) -> Self {
        Self { cursor, rows }
    }

    pub fn cursor(&self) -> TimestampCursor {
        self.cursor
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
}

/// Payload batch returned by a timestamp-cursor poll.
#[derive(Debug, Clone)]
pub struct PolledPayloadBatch {
    cursor: TimestampCursor,
    payload: RecordBatch,
}

impl PolledPayloadBatch {
    pub fn new(cursor: TimestampCursor, payload: RecordBatch) -> Self {
        Self { cursor, payload }
    }

    pub fn cursor(&self) -> TimestampCursor {
        self.cursor
    }

    pub fn payload(&self) -> &RecordBatch {
        &self.payload
    }
}
