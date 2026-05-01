# Product Mission

Kerald is a distributed messaging framework designed to be easier to use than Kafka-class systems while remaining lightweight, operationally simple, and efficiency-first.

Core product requirements:

- Support single-node and multi-node cluster operation.
- Coordinate multi-node clusters with TigerBeetle-style Viewstamped Replication (VSR): deterministic committed control-plane state, view-based primary fencing, quorum-guarded admission, and conservative ingress rejection when quorum or durability cannot be proven.
- Preserve partitionless topics: any node can receive writes, and public APIs must not expose partition concepts. Acknowledgement/admission may route internally through the current VSR primary but must not expose primary ownership to clients.
- Reject ingress when eventual delivery guarantees cannot be upheld.
- Track subscriber notification progress separately from payload delivery progress.
- Use nanosecond timestamp cursors for progress; do not use offset-based progress where timestamp cursors are required.
- Apply TTL precedence in this order: cluster default, topic override, message override.
- Use QUIC as the baseline protocol, with extension points for gRPC, Arrow ADBC, MQTT, and Kafka-compatible front doors.
- Represent payloads with Arrow.
- Persist through Lance read/write boundaries only; brokers must not embed LanceDB query responsibilities.
- Use OpenDAL for storage abstraction across local FS, S3, R2, GCS, and Azure Blob support paths.
- Target Rust 1.95, Python 3.10+ bindings through PyO3, and Java 25+ bindings through the FFM API without JNI.
- Include OpenTelemetry logs, metrics, and traces in production telemetry.
