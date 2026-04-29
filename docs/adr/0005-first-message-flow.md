# ADR 0005: First Message Flow

Status: Proposed

## Context

Kerald needs a first end-to-end broker path that can accept a message without
introducing partition concepts, offset cursors, broker-side blocking subscriber
semantics, or premature protocol and storage abstractions.

This milestone must preserve safety-first admission: a single-node broker may
accept writes after startup, while a multi-node broker must continue rejecting
writes until coordination can prove quorum safety.

## Decision

Add a bounded volatile single-node message flow to `RunningBroker`:

- Declare topics with `TopicDefinition`.
- Publish Arrow `RecordBatch` payloads directly through the `MessagePayload`
  type alias.
- Require each accepted message timestamp to advance beyond the topic's last
  timestamp.
- Return `MessageNotification` for accepted writes.
- Let subscribers poll notifications and payload batches independently with
  nanosecond timestamp cursors.
- Keep the volatile topic catalog and per-topic message window explicitly
  bounded.

The broker rejects publishes when write admission is disabled, the topic is
unknown, the payload schema does not match the topic schema, or the timestamp
does not advance. It also rejects ingress when the volatile per-topic message
window is full. It rejects new topic declarations when the volatile topic
catalog is full.

This decision does not introduce durable Lance storage, OpenDAL object storage,
QUIC transport, replication, or multi-node write routing.

## Alternatives Considered

- Offset-based cursors: rejected because Kerald progress semantics are
  timestamp-based.
- Partition-indexed message logs: rejected because topics are partitionless.
- A custom payload wrapper around Arrow batches: rejected because `RecordBatch`
  already represents the payload shape and extra single-field wrappers add
  indirection.
- Unbounded in-memory message retention: rejected because it violates Kerald's
  lightweight and efficiency-first mission and can silently convert broker
  memory into the message store.
- Dropping older accepted messages from a ring buffer: rejected because that
  would silently degrade delivery guarantees.
- Blocking subscriber delivery calls: rejected because notification tracking
  must remain independent from payload delivery tracking.

## Consequences

The first message can flow through the broker API while preserving the core
topic invariants and avoiding unbounded memory growth. Tests can now exercise
publish, bounded ingress rejection, notification polling, and payload polling
before protocol and persistence work begins.

The implementation is intentionally a tiny volatile window. Restart durability,
retention, TTL enforcement, Lance persistence, OpenDAL-backed storage, and
replicated delivery guarantees remain future work. This path must be replaced
before any production claim of durable delivery.

## Rollout or Migration Notes

No migration is required. This is the first message-flow API in the repository.
Future durable implementations must preserve the externally observable
partitionless, timestamp-cursor, and separated notification/payload semantics.
