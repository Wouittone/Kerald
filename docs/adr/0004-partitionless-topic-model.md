# ADR 0004: Partitionless Topic Model

Status: Proposed

## Context

Kerald's product mission requires topics to be partitionless and payloads to be
represented with Arrow. Clients should not need to understand partition counts,
partition ids, partition keys, or partition ownership to create, write to, or
read from a topic. They do need a schema contract so producers, consumers,
storage, and bindings agree on the Arrow payload shape. This is a long-term
public API decision because topic identity and schema metadata will be visible
through Rust, Python, Java, QUIC, and future protocol front doors.

The topic model must also leave safety boundaries intact. Any broker node may
receive writes, but acknowledgement/admission must wait until VSR-backed
safety-first admission proves quorum health and the required durability path.
This must remain internal and must not expose partition, shard, primary
ownership, or offset concepts to clients. Multi-node coordination, storage
placement, replication, and recovery may need internal structures, but those
structures must not become client-visible partition semantics.

This decision does not define write routing, durable topic catalog storage,
timestamp cursor persistence, subscriber notification tracking, payload polling,
or TTL override storage. Those are separate decision surfaces.

## Decision

Kerald's initial public topic model is name-plus-schema and partitionless:

- `TopicName` is represented as a `String` alias, with validation performed by
  the `parse_topic_name` and `TopicDefinition::new` construction boundaries.
- `TopicDefinition` stores the topic name and Arrow `SchemaRef` directly, with
  no partition metadata.
- Public topic APIs must not require or return partition counts, partition ids,
  partition owners, shard owners, or offset-like partition positions.

The implementation should keep levels of indirection minimal. New wrappers
around standard Rust or Arrow types should be added only when they enforce
meaningful invariants or remove real complexity.

Topic names are trimmed before validation, must be non-empty, must be at most
255 bytes, and may contain ASCII letters, numbers, dots, underscores, and
hyphens.

Future internal routing and replication work may introduce private data
structures, but they must preserve the client-facing partitionless topic model.
Write admission and durable payload paths must validate Arrow payloads against
the topic schema once those paths exist. Any future topic-level metadata,
including TTL overrides, must be added without turning partitions into public
API concepts.

## Alternatives Considered

Exposing Kafka-style partitions was rejected because it directly conflicts with
Kerald's partitionless topic requirement and makes the system harder to operate.

Accepting arbitrary topic names was rejected for the initial slice because names
will be used across protocol front doors, configuration, storage paths, and
language bindings. A small portable character set reduces early compatibility
risk.

Inventing a Kerald-specific schema format was rejected because the product
baseline already requires Arrow payload representation. Reusing Arrow schema
metadata keeps the broker boundary aligned with payload validation, storage, and
bindings.

Wrapping topic names and schemas in single-field newtypes was rejected because
the current invariants can be enforced at parsing/construction boundaries while
using standard `String` and Arrow `SchemaRef` values directly.

Adding internal routing metadata to `TopicDefinition` was rejected for now
because routing, quorum-backed admission, durable storage, and protocol behavior
are not implemented yet. Adding speculative fields would create compatibility
surface without proven need.

## Consequences

Positive consequences:

- The public topic API starts from Kerald's core partitionless invariant.
- Clients can identify topics and payload shape without learning broker
  placement details.
- Future protocol and language bindings have a small, portable topic identity
  and schema surface.
- The model avoids offset-style partition positions before timestamp cursor
  semantics are introduced.

Negative consequences:

- Name validation may need a deliberate compatibility migration if later
  protocols require a wider character set.
- Durable topic catalog behavior is not implemented by this decision.
- Internal routing and replication work must map from a public name-plus-schema
  model to private coordination state without leaking that state to clients.
- Adding the Arrow schema crate creates an explicit dependency before payload
  ingestion exists.

## Rollout or Migration Notes

No persisted topic catalog exists yet, so no data migration is required. The
initial rollout adds the core Rust topic model with Arrow schema metadata, unit
coverage, integration coverage showing cluster-size independence, and Cucumber
behavior coverage for the partitionless client-facing contract.
