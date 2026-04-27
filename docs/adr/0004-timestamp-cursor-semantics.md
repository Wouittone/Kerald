# ADR 0004: Timestamp Cursor Semantics

## Status

Accepted

## Context

Kerald progress tracking must use nanosecond timestamps rather than offsets.
This decision affects client APIs, subscriber progress, polling windows,
persistence lookup boundaries, and compatibility for language bindings. The
cursor model needs to be explicit before topic, subscriber, protocol, or storage
work grows APIs around a different progress representation.

## Decision

Represent client-visible progress with Arrow's native signed 64-bit nanosecond
timestamp value and Arrow's timestamp datatype:
`DataType::Timestamp(TimeUnit::Nanosecond, None)`. Kerald exposes a
`TimestampCursor` type alias for that scalar rather than a custom cursor
wrapper. Cursor ordering is timestamp ordering. Bounded polling uses validated
inclusive standard ranges.

Creating a cursor rejects Arrow timestamp values before the Unix epoch. The
cursor API does not expose partition or offset concepts, and it does not convert
through a separate date/time library.

## Alternatives Considered

- Broker log offsets: rejected because offsets conflict with Kerald's
  timestamp-cursor requirement and would leak internal storage layout into
  client progress.
- Partition-scoped sequence numbers: rejected because topics are partitionless
  and client progress must not depend on partition ownership.
- Signed integer timestamps: rejected for the initial API because pre-epoch
  progress has no product requirement and would add unnecessary states.
- Manual `SystemTime` arithmetic: rejected because cursor progress should use
  Arrow's native timestamp representation directly instead of converting
  through wall-clock types in the broker.
- Separate third-party timestamp libraries: rejected because Kerald already
  standardizes payload representation on Arrow and should avoid extra
  conversions in hot progress paths.

## Consequences

Positive:

- Client progress has one simple representation across broker modes and future
  language bindings.
- Cursor ordering is deterministic and independent from storage layout.
- Future polling APIs can accept timestamp ranges without partition assumptions.
- Cursor values align directly with Arrow payload and persistence boundaries.
- Topic schemas can represent client-visible progress fields with the same
  Arrow timestamp nanosecond type.

Negative:

- Physical storage implementations may need a separate internal index to map
  timestamp ranges to durable records efficiently.
- Multiple payloads can share the same nanosecond timestamp, so future delivery
  work must define tie handling without exposing offsets.
- Arrow timestamp nanoseconds use a signed 64-bit value, so the positive
  timestamp range is smaller than an unsigned 64-bit nanosecond counter.

## Rollout Or Migration Notes

This is the first cursor API, so no migration is required. Future subscriber,
protocol, and persistence changes should reuse the Arrow timestamp-nanosecond
cursor scalar for client-visible progress and keep any physical addressing internal. Topic
payload work should pair Arrow message ingress with topic-declared schemas, as
recorded in ADR 0005.
