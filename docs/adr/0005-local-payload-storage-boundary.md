# ADR 0005: Local Payload Storage Boundary

## Status
Accepted

## Context
Kerald needs a durable payload foundation for partitionless Arrow topics before broker write admission and protocol front doors can safely depend on stored data. The first persistence slice is local-only: it validates the storage boundary without making distributed delivery, notification tracking, or multi-node admission decisions.

Mandatory constraints still apply: topics remain partitionless, progress uses nanosecond timestamp cursors rather than offsets, notification tracking is separate from payload polling, and brokers must not embed LanceDB-style query responsibilities.

## Decision
Kerald stores payload `RecordBatch` values as Lance datasets addressed through an OpenDAL operator. The initial implementation supports local filesystem storage through OpenDAL `Fs`; cloud OpenDAL services remain a future support path.

Stored batches prepend a reserved internal Arrow column named `__kerald_cursor_ns` with timestamp-nanosecond semantics. Payload polling filters for rows whose cursor is strictly greater than the supplied `TimestampCursor` value. This preserves timestamp cursor semantics and avoids offset-like progress.

The storage API is a persistence boundary only. It may append and poll payload batches for a known `TopicDefinition`, but it must not become a broker query engine, expose partition placement, or acknowledge producer writes independently of future coordination and durability admission checks.

## Alternatives considered
- Direct filesystem writes: rejected because storage must use OpenDAL so local, S3, R2, GCS, and Azure Blob paths can share one abstraction.
- Embedding LanceDB/query-engine behavior: rejected because brokers own messaging durability, not analytical query execution.
- Offset-like sequence positions: rejected because Kerald progress/cursors use nanosecond timestamps.
- Partitioned storage paths or partition ids: rejected because topics are partitionless and any node may accept writes once admission is safe.
- Implementing cloud OpenDAL services in this slice: deferred to keep the first boundary small and locally testable.

## Consequences
Positive consequences:
- Establishes a concrete Arrow/Lance persistence foundation.
- Keeps payload polling independent from notification tracking.
- Makes cursor filtering deterministic and timestamp-based.
- Keeps the broker API free of partitions and offsets.

Negative consequences:
- Introduces a reserved internal column that payload schemas must not own.
- Requires schema validation before durable append.
- Does not yet solve replication, quorum durability, write admission, or protocol exposure.
- Lance/OpenDAL dependencies must continue to be validated against the musl Alpine container target.

## Rollout and migration notes
No data migration is expected because Kerald has no prior public payload dataset format. Future broker write admission must gate acknowledgements on coordination and storage durability instead of assuming that a local append alone is sufficient. Future OpenTelemetry work should add logs, metrics, and traces for storage initialization, append latency/failure, poll latency/failure, and durability health used by admission decisions.
