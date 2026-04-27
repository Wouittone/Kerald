# Broker Runtime

Kerald's broker runtime is async-first for lifecycle and I/O-facing behavior.

Synchronous APIs are appropriate for pure value construction, deterministic validation, and in-memory configuration helpers. Runtime APIs that may touch transport, storage, coordination, timers, telemetry export, subscriber polling, or graceful shutdown should be async and fallible.

The broker server process owns the Tokio runtime. Library callers embed Kerald by awaiting broker lifecycle APIs inside their own runtime.

Operator CLI commands are distinct from the broker server process. They should perform finite control-plane operations by communicating with a running broker process to start, stop, inspect, or modify behavior.

Runtime boundaries must preserve the mandatory architecture constraints:

- Topics are partitionless, and runtime task layout must not introduce partition ownership assumptions.
- Client progress uses nanosecond timestamp cursors, not offsets.
- Notification tracking remains independent from payload delivery tracking.
- Write admission is safety-first and rejects ingress when delivery guarantees cannot be proven.
- Persistence remains Lance read/write only behind OpenDAL-backed storage.

As transport and persistence arrive, runtime components should use bounded queues, explicit backpressure, clear task ownership, OTel logs/metrics/traces, protected control-plane execution, and graceful shutdown hooks. Arrow is the broker's in-memory exchange format; compute-heavy query, compaction, optimization, and analytics work belongs outside the broker process.
