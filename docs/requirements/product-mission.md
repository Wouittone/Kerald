# Product Mission

Kerald is a distributed messaging framework designed to be easier to use than Kafka-class systems while remaining lightweight, operationally simple, and efficiency-first.

Core product requirements:

- Support standalone broker mode and clustered broker mode.
- Preserve partitionless topics: any node can accept writes, and public APIs must not expose partition concepts.
- Reject ingress when eventual delivery guarantees cannot be upheld.
- Track subscriber notification progress separately from payload delivery progress.
- Use nanosecond timestamp cursors for progress; do not use offset-based progress where timestamp cursors are required.
- Apply TTL precedence in this order: cluster default, topic override, message override.
- Use QUIC as the baseline protocol, with extension points for gRPC, Arrow ADBC, MQTT, and Kafka-compatible front doors.
- Represent payloads with Arrow.
- Persist through Lance read/write boundaries only; brokers must not embed LanceDB query responsibilities.
- Use OpenDAL for storage abstraction across local FS, S3, R2, GCS, and Azure Blob support paths.
- Target Rust 1.92, Python 3.10+ bindings through PyO3, and Java 25+ bindings through the FFM API without JNI.
- Include OpenTelemetry logs, metrics, and traces in production telemetry.
