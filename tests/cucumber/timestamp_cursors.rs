use cucumber::{World, given, then, when};
use kerald::{TimestampCursor, TimestampCursorRange, timestamp_cursor_from_epoch_nanos, timestamp_cursor_range};

#[derive(Debug, Default, World)]
struct CursorWorld {
    earliest: Option<TimestampCursor>,
    latest: Option<TimestampCursor>,
    candidate: Option<TimestampCursor>,
    range: Option<TimestampCursorRange>,
}

#[given(expr = "the earliest payload timestamp cursor is {int} nanoseconds")]
async fn earliest_cursor(world: &mut CursorWorld, nanos: i64) {
    world.earliest = Some(timestamp_cursor_from_epoch_nanos(nanos).expect("scenario timestamp should be valid"));
}

#[given(expr = "the latest payload timestamp cursor is {int} nanoseconds")]
async fn latest_cursor(world: &mut CursorWorld, nanos: i64) {
    world.latest = Some(timestamp_cursor_from_epoch_nanos(nanos).expect("scenario timestamp should be valid"));
}

#[given(expr = "a payload timestamp cursor is {int} nanoseconds")]
async fn candidate_cursor(world: &mut CursorWorld, nanos: i64) {
    world.candidate = Some(timestamp_cursor_from_epoch_nanos(nanos).expect("scenario timestamp should be valid"));
}

#[when("a client opens an inclusive timestamp cursor range")]
async fn inclusive_range(world: &mut CursorWorld) {
    let earliest = world.earliest.expect("scenario should define earliest cursor");
    let latest = world.latest.expect("scenario should define latest cursor");
    world.range = Some(timestamp_cursor_range(earliest, latest).expect("scenario range should be valid"));
}

#[then("the payload is visible in the polling range")]
async fn payload_visible(world: &mut CursorWorld) {
    let range = world.range.as_ref().expect("scenario should create a range");
    let candidate = world.candidate.expect("scenario should define candidate cursor");
    assert!(range.contains(&candidate));
}

#[then(expr = "the client sees nanosecond timestamp value {int}")]
async fn client_sees_nanos(world: &mut CursorWorld, nanos: i64) {
    let candidate = world.candidate.expect("scenario should define candidate cursor");
    assert_eq!(candidate, nanos);
}

#[tokio::main]
async fn main() {
    CursorWorld::run("tests/cucumber/features/timestamp_cursors.feature").await;
}
