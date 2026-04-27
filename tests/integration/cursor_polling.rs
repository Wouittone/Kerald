use kerald::{timestamp_cursor_from_epoch_nanos, timestamp_cursor_range};

#[test]
fn polling_windows_are_bound_by_timestamp_cursors() {
    let first_payload = timestamp_cursor_from_epoch_nanos(100).expect("timestamp should be valid");
    let second_payload = timestamp_cursor_from_epoch_nanos(200).expect("timestamp should be valid");
    let third_payload = timestamp_cursor_from_epoch_nanos(300).expect("timestamp should be valid");
    let window = timestamp_cursor_range(first_payload, second_payload).expect("timestamp window should be valid");

    let visible_payloads = [first_payload, second_payload, third_payload]
        .into_iter()
        .filter(|cursor| window.contains(cursor))
        .collect::<Vec<_>>();

    assert_eq!(visible_payloads, vec![first_payload, second_payload]);
}
