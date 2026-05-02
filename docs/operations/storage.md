# Payload Storage Operation

Kerald's current payload storage slice is local-only and is not yet wired to a
producer protocol acknowledgement path. A successful local append is durable
payload persistence for that broker storage root, not a multi-node delivery or
notification guarantee.

## Configuration and layout

Local storage is initialized with `StorageConfig::local(root)`. The root
directory is created when storage starts. Topic payloads are stored as Lance
datasets beneath `topics/<topic>.lance` and all Lance object access must flow
through the OpenDAL-backed storage boundary.

Operators and tests must not edit Lance dataset contents directly. Future cloud
storage support must add OpenDAL service configuration rather than bypassing the
storage abstraction.

## Failure modes

- Invalid or unusable storage roots fail storage initialization.
- Polling a topic that has no dataset returns an empty result.
- Payload schemas must exactly match the topic Arrow schema.
- Topic schemas using the reserved `__kerald_cursor_ns` field are rejected.
- Existing datasets whose stored schema does not match the topic schema fail
  safely instead of being read or appended.
- Lance/OpenDAL operation failures surface as storage operation errors and must
  not be converted into successful broker admission.

Payload polling is independent from notification tracking. Poll calls use
nanosecond `TimestampCursor` values and return payload batches with stored
cursors strictly greater than the supplied cursor, ordered by cursor.

## Rollout and observability

No data migration exists for this initial storage format. Before relying on this
path in a release, validation evidence must include create, append, reopen,
empty poll, schema mismatch, and reserved-field scenarios on the musl target.

Current tracing is not the full production telemetry target. Production rollout
must add OTel logs, metrics, and traces for storage initialization,
append/poll latency and failures, dataset schema mismatches, and storage health
signals that affect safety-first admission.
