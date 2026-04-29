# Requirements

This directory contains product and system requirements for Kerald.

Requirements should describe observable commitments and constraints rather than implementation details. Keep the core mission visible in each requirement set:

- Easier to use than Kafka-class systems.
- Lightweight and operationally simple.
- Efficiency-first across CPU, memory, network, and storage while preserving strong performance.

When a change affects public behavior, delivery guarantees, protocol compatibility, cursor semantics, storage boundaries, or runtime support, update or add a requirement document in this directory.

Current requirement notes:

- `broker-modes.md`: Single-node and multi-node startup, quorum, and admission behavior.
- `product-mission.md`: Core product mission and mandatory system requirements.
- `runtime-and-bindings.md`: Rust, Python, and Java runtime/binding targets.
- `topics.md`: Partitionless topic identity, schema, and validation requirements.
