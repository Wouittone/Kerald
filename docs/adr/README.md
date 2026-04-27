# Architecture Decision Records

Use ADRs for decisions with long-term compatibility, operational, or architecture impact.

Create or update an ADR when changing:

- Consistency or cluster coordination.
- Protocol models or wire compatibility.
- Storage and durability boundaries.
- Delivery and notification semantics.
- Public APIs with long-term compatibility impact.

Each ADR must include:

- Context
- Decision
- Alternatives considered
- Consequences, both positive and negative
- Rollout or migration notes when applicable

Number ADR files sequentially with a short kebab-case title, for example `0001-cluster-coordination.md`.

## Current ADRs

- `0001-cluster-coordination.md`: Cluster coordination model, safety boundary, quorum behavior, and observability expectations.
- `0002-broker-coordination-subsystem.md`: Default clustered broker coordination algorithm using TigerBeetle-style VSR.
- `0003-async-broker-runtime.md`: Async-first broker lifecycle and runtime boundary.
