# ADR 0003: Async Broker Runtime Boundary

Status: Proposed

## Context

Kerald's broker will serve many concurrent client, inter-broker, persistence, and telemetry operations while preserving safety-first admission and partitionless topic semantics. The runtime boundary must support efficient QUIC transport, deterministic broker discovery and VSR voter-set validation, VSR-style coordination, OpenDAL-backed storage, Lance read/write paths, subscriber polling, and OTel telemetry without dedicating one operating-system thread to each concurrent activity.

The first broker startup slice only evaluated static configuration and local admission state, so synchronous startup was sufficient for that limited behavior. As soon as broker lifecycle touches networking, storage, coordination tasks, timers, or graceful shutdown, synchronous APIs would force inefficient blocking or require a later public API break.

This decision does not change Kerald's client-facing invariants: topics remain partitionless, progress uses nanosecond timestamp cursors, notification tracking remains separate from payload delivery tracking, and brokers reject ingress when eventual delivery guarantees cannot be proven.

## Decision

Kerald broker lifecycle APIs are async-first. `Broker::start` is an async, fallible boundary owned by the broker server runtime. The long-running broker process owns a Tokio runtime through its server entrypoint.

The operator CLI is a separate control surface. CLI operations should be finite commands that communicate with a running broker process to start, stop, inspect, or modify behavior. They must not become the long-running broker runtime merely because they perform control-plane work.

Pure value construction and deterministic configuration parsing remain synchronous unless they perform I/O that benefits from async execution. Examples include `BrokerConfig::single_node`, quorum calculation, and validation of in-memory values.

Runtime implementation guidance:

- Network transport, inter-broker discovery, coordination, storage, subscriber polling, timers, telemetry export, and graceful shutdown paths should use async APIs.
- Broker startup should fail explicitly when required runtime resources cannot be initialized.
- Background tasks must have explicit ownership and shutdown behavior rather than being detached without lifecycle control.
- Brokers must treat Arrow as the in-memory exchange format, not as permission to perform compute-heavy query, compaction, optimization, or analytics work inside the broker process. Those workloads belong in dedicated modules or external systems.
- Backpressure should be explicit through bounded queues, admission state, and resource limits.
- High-priority control paths, including shutdown hooks, should have protected execution capacity through distinct task supervisors, queues, or Tokio execution pools so they remain responsive under data-plane load.

## Alternatives Considered

Keeping broker lifecycle synchronous until the first concrete async transport was rejected because it would let synchronous public APIs harden before the messaging hot path exists.

Adding sync and async startup variants was rejected because it creates two lifecycle surfaces for a broker that is fundamentally I/O-oriented.

Using one thread per connection or operation was rejected because it conflicts with Kerald's efficiency-first mission and would not scale cleanly for QUIC streams, subscriber polling, storage calls, and coordination traffic. This rejection does not preclude distinct async execution pools for control-plane priority or shutdown responsiveness.

## Consequences

Positive consequences:

- The public broker lifecycle matches the expected I/O-heavy implementation model.
- QUIC, OpenDAL, coordination, polling, timers, and telemetry can compose without blocking runtime worker threads.
- Runtime startup failures can be propagated before a broker is considered running.
- Future task ownership and graceful shutdown work has a clear API boundary.
- Control-plane operations can be designed as finite CLI commands against a running process instead of being conflated with the long-running broker runtime.

Negative consequences:

- Tests and embedding code must run inside an async runtime.
- The broker server binary depends on Tokio before the first concrete transport task lands.
- Purely synchronous bootstrap code now uses an async call even when no await point is currently needed internally.

## Rollout or Migration Notes

The initial rollout converts `Broker::start` to `async fn start(self) -> Result<RunningBroker, BrokerError>` and updates the broker server entrypoint, integration tests, and behavior tests to await startup.

Follow-up work should add explicit task supervision, protected control-plane execution, and graceful shutdown once transport, discovery, coordination, or storage tasks are introduced. Runtime rollout for VSR must include task supervision and shutdown behavior for primary election/view-change tasks, replication streams, quorum-health timers, and admission rejection paths. Behavior and integration suites should cover startup failure, shutdown, admission rejection under runtime failures, and preservation of partitionless timestamp-cursor semantics.
