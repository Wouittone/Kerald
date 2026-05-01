# Storage Architecture

Kerald's payload persistence boundary stores Arrow payload batches in Lance datasets through an OpenDAL operator. The current implementation is local filesystem storage via OpenDAL `Fs`; direct filesystem persistence paths are avoided so future S3, R2, GCS, and Azure Blob support can use the same abstraction.

## Payload datasets

Each partitionless topic maps to a Lance dataset under the storage root. Kerald prepends a reserved internal `__kerald_cursor_ns` timestamp-nanosecond column to persisted batches. Client payload schemas remain Arrow schemas owned by `TopicDefinition`; the reserved column is internal storage metadata and payload schemas using that field name are rejected.

Polling is cursor-based:

- callers pass a `TimestampCursor` measured in nanoseconds since the Unix epoch;
- storage returns payload batches whose stored cursor is strictly greater than the supplied cursor;
- no offset, partition id, partition count, or ownership hint is required.

## Boundary limits

This storage layer is not a broker-side query engine. It provides Lance read/write behavior needed for durable messaging payloads and must not grow analytical query responsibilities.

Storage append and polling are also independent from notification tracking. Later broker write admission must combine coordination and durability evidence before acknowledging writes; a local append alone is not a multi-node delivery guarantee.

## Operational notes

The current slice does not change container layout or broker runtime wiring. Future production integration should emit OpenTelemetry logs, metrics, and traces for storage initialization, append/poll latency, failures, and storage health signals that influence admission.
