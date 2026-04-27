# Topic Schemas

Kerald topics are partitionless and schema-declared. The topic metadata layer
references the Lance table schema that every accepted message for that topic
must satisfy. Any broker may accept writes for a topic only when it can validate
the payload against the Arrow schema recorded in the Lance table manifest for
the table version being written and preserve delivery guarantees.

The schema boundary belongs at topic creation and message ingress, not inside a
broker-side query engine. Brokers validate Arrow payload shape and metadata,
then persist and replicate messages through the storage and coordination paths.
There is a single schema reference between the topic and its Lance table:
Kerald follows Lance table versioning semantics, where each table version's
manifest carries the complete schema definition. Schema evolution is handled as
part of Lance's table capabilities rather than by a separate broker-maintained
schema version for topics.

Future persistence architecture assessments should evaluate whether the Lance
table and schema boundary is owned by dedicated LanceDB/Lance-backed
persistence nodes rather than broker nodes. Those assessments must reference
the official LanceDB documentation and storage architecture guidance documented
in `lancedb-persistence.md`.

Brokers must not add embedded analytics, schema inference, or LanceDB query
responsibilities as part of schema handling.

Cursor fields that are exposed to clients should align with Kerald cursor
semantics: Arrow timestamp nanoseconds, not offsets or partition positions.
Future schema evolution work must follow Lance table versioning semantics and
constraints before allowing writes under an evolved schema.
