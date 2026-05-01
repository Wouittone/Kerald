# Storage Requirements

Kerald persistence must remain lightweight, explicit, and aligned with the broker invariants.

## Current local payload storage slice

- Store Arrow `RecordBatch` payloads in Lance datasets.
- Access Lance through OpenDAL; local filesystem support uses OpenDAL `Fs`.
- Keep topics partitionless: storage APIs must not require partition ids, partition counts, offsets, shard ids, or ownership hints.
- Use `TimestampCursor` nanoseconds for progress. Polling returns payloads with stored cursor values strictly greater than the supplied cursor.
- Reject payload batches whose Arrow schema does not exactly match the topic schema.
- Reject topic payload schemas that use the reserved internal `__kerald_cursor_ns` field name.
- Keep notification tracking separate from payload polling.
- Do not expose broker-side query-engine responsibilities.

## Future requirements

- Add OpenDAL-backed S3, R2, GCS, and Azure Blob support paths without bypassing the storage abstraction.
- Integrate storage durability with safety-first write admission; reject ingress when durability and eventual delivery guarantees cannot be upheld.
- Add OpenTelemetry logs, metrics, and traces for storage initialization, append/poll operations, failures, and admission-relevant health.
- Validate Lance/OpenDAL dependencies in musl Alpine multi-stage container builds.
