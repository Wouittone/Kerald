# Cursor Semantics Requirements

Kerald progress cursors are nanosecond timestamps. They are not broker log
positions, partition positions, or client-visible sequence numbers.

## Requirements

- Public progress APIs MUST represent cursors as Arrow timestamp nanoseconds since the Unix epoch.
- Cursor timestamp representation MUST use Arrow date/time types directly rather than converting through a separate timestamp library.
- Cursor ordering MUST be timestamp ordering.
- Cursor creation MUST reject Arrow timestamp values before the Unix epoch.
- Bounded polling MUST use timestamp cursor ranges, with explicit validation that the range start is not after the range end.
- Cursor APIs MUST NOT expose partition or offset concepts.

## Testing Expectations

- Unit tests cover cursor creation, Arrow timestamp typing, ordering, and range validation.
- Integration tests cover polling windows bounded by timestamp cursors.
- Behavior tests cover the client-visible expectation that progress is expressed as nanosecond timestamps.
