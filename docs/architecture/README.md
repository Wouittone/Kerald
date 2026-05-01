# Architecture

This directory contains system architecture notes for Kerald.

Architecture documents should explain component boundaries, protocols, data flow, consistency models, storage responsibilities, and operational behavior. They should preserve the mandatory system constraints:

- Topics are partitionless.
- Progress uses nanosecond timestamp cursors, not offsets.
- Notification tracking is separate from payload delivery tracking.
- Write admission is safety-first.
- Persistence is Lance read/write only behind OpenDAL-backed storage boundaries.
- Cluster coordination uses TigerBeetle-style Viewstamped Replication (VSR) for control-plane agreement and safety-first admission.

Current architecture notes:

- `broker-coordination.md`: VSR control-plane coordination, admission, schema, and message-flow architecture.
- `broker-runtime.md`: Async-first broker lifecycle and runtime boundary.
- `storage.md`: Lance/OpenDAL payload storage boundary and timestamp cursor polling.
- `topics.md`: Partitionless topic metadata, Arrow schema, and public model boundary.

When an architecture document records a long-lived decision rather than explanatory context, add or update an ADR in `docs/adr/`.
