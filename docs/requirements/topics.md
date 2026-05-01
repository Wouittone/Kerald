# Partitionless Topics

Kerald topics are named message streams with Arrow payload schemas and without
partition-facing API concepts. Clients identify a topic by name and schema; they
do not choose partition counts, partition ids, partition keys, or partition
owners.

The topic model must preserve Kerald's core guarantees:

- Any broker node may receive producer writes for a topic. If the broker is not
  the current VSR primary for the relevant control-plane view, it must route or
  forward internally; no broker may acknowledge/admit the write until
  safety-first admission can prove VSR quorum health and the required durability
  path.
- Multi-node routing, primary forwarding, and replication must remain
  broker-internal concerns.
- Payloads for a topic must conform to the topic's Arrow schema before they are
  accepted into durable broker paths.
- Client progress must be modeled with nanosecond timestamp cursors, not
  offset-like partition positions.
- Subscriber notification tracking remains independent from payload delivery
  tracking.

Topic names are operator and client visible identifiers. The initial name rules
are intentionally simple and portable:

- Names are trimmed before validation.
- Names must not be empty.
- Names must be at most 255 bytes.
- Names may contain ASCII letters, numbers, dots, underscores, and hyphens.

Later topic metadata such as TTL overrides may be added without introducing
partition fields. TTL behavior must continue to follow the fixed precedence:
cluster-level default, then topic-level override, then message-level override.
