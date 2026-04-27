# ADR 0005: Topic-Declared Arrow Schemas

## Status

Accepted

## Context

Kerald represents payloads with Arrow and uses Arrow timestamp nanoseconds for
client-visible progress cursors. Accepting arbitrary Arrow messages without a
topic-level schema reference would push schema inference or late payload
validation into hot broker paths and make behavior less deterministic for
clients.

Topic schemas also affect public API compatibility: clients need to know which
payload shape a topic accepts before writing, and brokers need a deterministic
admission rule for schema mismatches.

## Decision

Topics must reference a Lance table schema before accepting Arrow messages.
There is one schema reference for the topic: the Arrow schema recorded in the
Lance table manifest for the table version being written. Brokers validate
incoming message payloads against that Lance table schema at ingress and reject
incompatible payloads explicitly.

Accepted messages are stored in the Lance table and table version whose manifest
defines the schema used for admission. Schema evolution follows Lance table
versioning and schema evolution semantics; Kerald does not maintain an
independent topic schema version sequence separate from Lance.

Future persistence architecture work should assess whether LanceDB/Lance-backed
persistence runs on dedicated nodes so table versioning, schema evolution, and
storage backend concerns are handled outside lightweight broker nodes. That
assessment must reference the official LanceDB documentation and storage
architecture guide.

Topic schema declarations must preserve Kerald's partitionless model. They must
not introduce partition ownership, offset-based progress, or broker-side query
responsibilities. Client-visible progress fields should use Arrow timestamp
nanoseconds where represented in payload or metadata schemas.

## Alternatives Considered

- Schema inference from the first message: rejected because it makes topic
  behavior depend on write race order and weakens deterministic admission.
- Accepting any Arrow schema per message: rejected because it complicates
  subscribers, storage layout, and client compatibility.
- Maintaining a separate topic schema version alongside Lance table versions:
  rejected because Lance table manifests already carry the Arrow schema for each
  table version, and duplicating that versioning model would create drift.
- Broker-side query/schema engine responsibilities: rejected because brokers
  should validate Arrow payload boundaries without becoming embedded query
  engines.

## Consequences

Positive:

- Write admission has a clear payload compatibility rule.
- Clients can discover topic payload shape before producing messages.
- Arrow payload validation stays aligned with cursor and storage boundaries.
- Topic schema identity follows Lance table versioning rather than a separate
  broker-defined schema version.
- Persistence-heavy table management can be evaluated as a dedicated
  LanceDB/Lance node responsibility instead of becoming broker behavior.

Negative:

- Topic creation requires schema metadata before the first write.
- Schema evolution must respect Lance table versioning semantics and
  constraints.

## Rollout Or Migration Notes

This records the requirement before topic payload ingress exists, so no data
migration is required. Future topic, protocol, binding, and storage work should
carry the topic's Lance table schema reference through their APIs and tests.
Future persistence ADRs should reference `https://docs.lancedb.com/` and
assess storage architecture from that documentation, while keeping Rust crates
as Kerald's implementation path even when LanceDB examples are Python-first.
