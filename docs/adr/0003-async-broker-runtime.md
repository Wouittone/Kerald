# ADR 0003: Async Broker Runtime Boundary

Status: Proposed

## Context

Kerald's broker will serve many concurrent client, inter-broker, persistence, and telemetry operations while preserving safety-first admission and partitionless topic semantics. The runtime boundary must support efficient QUIC transport, dynamic voter discovery, VSR-style coordination, OpenDAL-backed storage, Lance read/write paths, subscriber polling, and OTel telemetry without dedicating one operating-system thread to each concurrent activity.

The first broker startup slice only evaluated static configuration and local admission state, so synchronous startup was sufficient for that limited behavior. As soon as broker lifecycle touches networking, storage, coordination tasks, timers, or graceful shutdown, synchronous APIs would force inefficient blocking or require a later public API break.

This decision does not change Kerald's client-facing invariants: topics remain partitionless, progress uses nanosecond timestamp cursors, notification tracking remains separate from payload delivery tracking, and brokers reject ingress when eventual delivery guarantees cannot be proven.

## Decision

Kerald broker lifecycle APIs are async-first. `Broker::start` is an async, fallible boundary owned by the broker runtime. The CLI binary owns a Tokio multi-thread runtime through `#[tokio::main]`.

Pure value construction and deterministic configuration parsing remain synchronous unless they perform I/O that benefits from async execution. Examples include `BrokerConfig::single_node`, quorum calculation, and validation of in-memory values.

Runtime implementation guidance:

- Network transport, inter-broker discovery, coordination, storage, subscriber polling, timers, telemetry export, and graceful shutdown paths should use async APIs.
- Broker startup should fail explicitly when required runtime resources cannot be initialized.
- Background tasks must have explicit ownership and shutdown behavior rather than being detached without lifecycle control.
- CPU-heavy Arrow work should be batched and isolated from async I/O progress when needed, for example through dedicated compute or blocking pools.
- Backpressure should be explicit through bounded queues, admission state, and resource limits.

## Alternatives Considered

Keeping broker lifecycle synchronous until the first concrete async transport was rejected because it would let synchronous public APIs harden before the messaging hot path exists.

Adding sync and async startup variants was rejected because it creates two lifecycle surfaces for a broker that is fundamentally I/O-oriented.

Using one thread per connection or operation was rejected because it conflicts with Kerald's efficiency-first mission and would not scale cleanly for QUIC streams, subscriber polling, storage calls, and coordination traffic.

## Consequences

Positive consequences:

- The public broker lifecycle matches the expected I/O-heavy implementation model.
- QUIC, OpenDAL, coordination, polling, timers, and telemetry can compose without blocking runtime worker threads.
- Runtime startup failures can be propagated before a broker is considered running.
- Future task ownership and graceful shutdown work has a clear API boundary.

Negative consequences:

- Tests and embedding code must run inside an async runtime.
- The binary depends on Tokio before the first concrete transport task lands.
- Purely synchronous bootstrap code now uses an async call even when no await point is currently needed internally.

## Rollout or Migration Notes

The initial rollout converts `Broker::start` to `async fn start(self) -> Result<RunningBroker, BrokerError>` and updates the CLI, integration tests, and behavior tests to await startup.

Follow-up work should add explicit task supervision and graceful shutdown once transport, discovery, coordination, or storage tasks are introduced. Behavior and integration suites should cover startup failure, shutdown, admission rejection under runtime failures, and preservation of partitionless timestamp-cursor semantics.
