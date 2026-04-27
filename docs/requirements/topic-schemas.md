# Topic Schema Requirements

Kerald topics are partitionless, but they are not schema-less. A topic that
accepts Arrow payloads must reference a Lance table schema before message
ingress.

## Requirements

- Topic creation MUST establish the Lance table schema reference before the topic accepts Arrow messages.
- Brokers MUST validate message payloads against the Arrow schema recorded for the Lance table version being written.
- Brokers MUST reject messages whose Arrow schema does not match the topic's Lance table schema reference.
- Kerald MUST NOT maintain a separate topic schema version sequence that duplicates Lance table versioning.
- Topic schemas MUST NOT introduce partition ownership or offset-based progress semantics.
- Timestamp cursor fields in payload or metadata schemas SHOULD use Arrow timestamp nanoseconds where client-visible progress is represented.
- Schema evolution MUST follow Lance table versioning semantics and constraints.

## Testing Expectations

- Unit tests cover Lance table schema-reference validation and payload-schema mismatch rejection.
- Integration tests cover topic creation followed by accepted and rejected Arrow message ingress against the topic's Lance table schema reference.
- Behavior tests cover the client-visible requirement that a topic rejects Arrow messages without a compatible declared schema.
