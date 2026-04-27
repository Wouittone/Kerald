# Architecture

This directory contains system architecture notes for Kerald.

Architecture documents should explain component boundaries, protocols, data flow, consistency models, storage responsibilities, and operational behavior. They should preserve the mandatory system constraints:

- Topics are partitionless.
- Progress uses nanosecond timestamp cursors, not offsets.
- Notification tracking is separate from payload delivery tracking.
- Write admission is safety-first.
- Persistence is Lance read/write only behind OpenDAL-backed storage boundaries.

Current architecture notes:

- `broker-runtime.md`: Async-first broker lifecycle and runtime boundary.
- `cursor-semantics.md`: Nanosecond timestamp cursor model for client-visible progress.
- `lancedb-persistence.md`: LanceDB/Lance persistence deployment assessment guardrails.
- `topic-schemas.md`: Topic-declared Arrow schema boundary for payload ingress.

When an architecture document records a long-lived decision rather than explanatory context, add or update an ADR in `docs/adr/`.
