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
- `parse_topic_name` validates and trims topic names at construction boundaries.
- `TopicDefinition` stores topic name and Arrow `SchemaRef` metadata directly.
- Local payload storage can append and poll Arrow batches in Lance through
  OpenDAL using nanosecond `TimestampCursor` values.
- Broker write routing, durable topic catalog storage, notification tracking,
  and protocol-facing payload delivery are future slices.

The topic model intentionally keeps indirection low. Thin wrappers are avoided
when a standard Rust or Arrow type already describes the data accurately.

Operational implications:

- A single-node broker can reason about a topic without assigning ownership to a
  partition.
- A multi-node broker may receive a write on any node, but acknowledgement may
  route internally through the current VSR primary and must wait until
  coordination can prove quorum safety.
- Adding topic metadata must preserve the partitionless surface and avoid
  offset-based progress concepts.
