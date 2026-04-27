# LanceDB Persistence Assessment

Kerald brokers should stay lightweight: transport, coordination, safety-first
admission, payload validation, and telemetry belong in the broker path, while
durable table semantics should remain in the Lance/LanceDB persistence boundary.

Future architecture assessments for persistence MUST reference the official
LanceDB documentation:

- `https://docs.lancedb.com/`

The documentation establishes LanceDB as built on Lance, with built-in table
versioning, schema evolution, and fast random access. It also includes storage
architecture guidance that future assessments should use when comparing
latency, scale, cost, and availability tradeoffs.

This strengthens the case for evaluating dedicated LanceDB/Lance-backed
persistence nodes instead of embedding persistence-heavy table management in
Kerald broker nodes. A deployment may still use embedded/local LanceDB during
development or small single-node operation, but multi-node architecture work
should assess whether separate persistence nodes better satisfy table
versioning, schema evolution, storage backend selection, and operational
availability requirements.

Kerald should continue to use Rust crates for implementation paths even when
LanceDB documentation presents examples in Python first. Documentation examples
do not change Kerald's language/runtime targets.

## LanceDB OSS And Tiered Storage

LanceDB OSS can be used as an embedded library with local filesystem paths or
direct object-storage URIs. That supports choosing a storage backend, but it is
not the same as built-in multi-tier storage inside OSS.

For Kerald, multi-tier storage with LanceDB OSS should be assessed as one of
these designs:

- Rely on the selected storage backend's own tiering or lifecycle features, for
  example object-store lifecycle policies or managed file-storage cold tiers.
- Run a Kerald-owned persistence service that embeds LanceDB/Lance via Rust and
  explicitly manages hot and cold placement outside broker nodes.
- Use dedicated LanceDB/Lance-backed persistence nodes that read and write the
  canonical Lance tables while brokers remain admission, transport, and
  coordination nodes.

Future assessments should not assume LanceDB OSS provides a distributed cache or
automatic multi-node hot tier. If those capabilities are required, the
assessment should compare a Kerald-managed persistence tier with LanceDB's
distributed offerings, while keeping the implementation path aligned with Rust
crates where Kerald owns code.

Broker responsibilities remain constrained:

- Brokers may validate Arrow payload shape against the topic's Lance table
  schema reference before admission.
- Brokers may call Lance/LanceDB read/write APIs through the persistence
  boundary.
- Brokers must not add embedded analytics, vector search, SQL, schema inference,
  compaction, optimization, or LanceDB query-engine responsibilities.
- Storage backend decisions must preserve the OpenDAL support path where Kerald
  owns storage abstraction, while LanceDB/Lance deployment assessments must
  account for LanceDB's documented backend tradeoffs.

Open questions for future ADRs:

- Whether production Kerald clusters run dedicated LanceDB/Lance persistence
  nodes by default.
- Whether multi-tier storage is handled by storage backend lifecycle policies,
  Kerald-owned persistence nodes, or a distributed LanceDB deployment.
- How broker admission coordinates with Lance table commits and version
  visibility.
- Which LanceDB storage backend profiles map to local development, small
  production, and horizontally scaled deployments.
