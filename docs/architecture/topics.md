# Topics

Kerald represents a topic as partitionless metadata keyed by a validated topic
name and an Arrow payload schema. The core topic definition does not include
partition counts, partition ids, partition ownership, shard ownership, or
routing keys.

This keeps the public model simpler than Kafka-class partitioned systems while
leaving room for internal coordination, replication, and placement strategies.
Those internal strategies must not leak into client-visible topic APIs or
progress semantics.

Current implementation boundary:

- `TopicName` is a `String` alias for the user-visible topic identifier.
- `TimestampNs` is an `i64` alias for nanosecond timestamp cursors.
- `MessagePayload` is an Arrow `RecordBatch` alias; the broker accepts Arrow
  batches directly instead of wrapping payloads in a one-field type.
- `parse_topic_name` validates and trims topic names at construction boundaries.
- `TopicDefinition` stores topic name and Arrow `SchemaRef` metadata directly.
- A running single-node broker can declare a topic, accept a schema-matching
  Arrow payload, return a notification timestamp, and let subscribers poll
  notifications and payload batches independently by timestamp cursor.
- Multi-node brokers continue to reject writes until coordination can prove
  quorum safety.
- Durable topic catalog storage, Lance persistence, OpenDAL-backed object
  storage, protocol front doors, and replicated write routing are future slices.

The topic model intentionally keeps indirection low. Thin wrappers are avoided
when a standard Rust or Arrow type already describes the data accurately.

Operational implications:

- A single-node broker can reason about a topic without assigning ownership to a
  partition.
- A multi-node broker may still reject write admission until coordination can
  prove quorum safety.
- Adding topic metadata must preserve the partitionless surface and avoid
  offset-based progress concepts.
