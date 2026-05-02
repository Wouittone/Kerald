# ADR 0005: Local Payload Storage Boundary

## Status
Accepted

## Context
Kerald needs a durable payload foundation for partitionless Arrow topics before broker write admission and protocol front doors can safely depend on stored data. The first persistence slice is local-only: it validates the storage boundary without making distributed delivery, notification tracking, or multi-node admission decisions.

Mandatory constraints still apply: topics remain partitionless, progress uses nanosecond timestamp cursors rather than offsets, notification tracking is separate from payload polling, and brokers must not embed LanceDB-style query responsibilities.

## Decision
Kerald stores payload `RecordBatch` values as Lance datasets addressed through an OpenDAL operator. The initial implementation supports local filesystem storage through OpenDAL `Fs`; cloud OpenDAL services remain a future support path.

Lance integration may use Lance/object-store compatibility shims, including URI handles required by Lance APIs, but all object reads, writes, listings, and commit operations must flow through the OpenDAL operator. A Lance URI used to select an object-store path is an internal compatibility handle, not permission to bypass OpenDAL with direct broker filesystem persistence.

Stored batches prepend a reserved internal Arrow column named `__kerald_cursor_ns` with timestamp-nanosecond semantics. Payload polling filters for rows whose cursor is strictly greater than the supplied `TimestampCursor` value and returns batches ordered by ascending cursor value. This preserves timestamp cursor semantics and avoids offset-like progress.

The storage API is a persistence boundary only. It may append and poll payload batches for a known `TopicDefinition`, but it must not become a broker query engine, expose partition placement, or acknowledge producer writes independently of future coordination and durability admission checks.

## Alternatives considered
- Direct filesystem writes: rejected because storage must use OpenDAL so local, S3, R2, GCS, and Azure Blob paths can share one abstraction.
- Passing Lance a direct local filesystem URI while treating OpenDAL as configuration only: rejected because it would weaken the storage abstraction and make future cloud backends a separate persistence path.
- Embedding LanceDB/query-engine behavior: rejected because brokers own messaging durability, not analytical query execution.
- Offset-like sequence positions: rejected because Kerald progress/cursors use nanosecond timestamps.
- Partitioned storage paths or partition ids: rejected because topics are partitionless and any node may accept writes once admission is safe.
- Implementing cloud OpenDAL services in this slice: deferred to keep the first boundary small and locally testable.

## Consequences
Positive consequences:
- Establishes a concrete Arrow/Lance persistence foundation.
- Keeps payload polling independent from notification tracking.
- Makes cursor filtering deterministic, timestamp-based, and ordered by cursor.
- Keeps the broker API free of partitions and offsets.
- Preserves the OpenDAL abstraction even while adapting to Lance/object-store API requirements.

Negative consequences:
- Introduces a reserved internal column that payload schemas must not own.
- Requires schema validation before durable append.
- Does not yet solve replication, quorum durability, write admission, or protocol exposure.
- Lance/OpenDAL dependencies must continue to be validated against the musl Alpine container target.
- The OpenDAL-backed Lance adapter must track Lance/object-store API changes without weakening the no-direct-filesystem boundary.

## Rollout and migration notes
No data migration is expected because Kerald has no prior public payload dataset format. Validation for this slice must cover create, append, reopen, empty-topic polling, strict cursor filtering, schema mismatch rejection, and reserved cursor-field rejection through the OpenDAL-backed path. Future broker write admission must gate acknowledgements on coordination and storage durability instead of assuming that a local append alone is sufficient. Future OpenTelemetry work should add logs, metrics, and traces for storage initialization, append latency/failure, poll latency/failure, and durability health used by admission decisions.
