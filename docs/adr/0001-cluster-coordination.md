# ADR 0001: Cluster Coordination Model

Status: Proposed

## Context

Kerald must support both single-node and multi-node cluster operation while preserving partitionless topics. Any node may receive writes, but acknowledgement/admission may require internal VSR primary routing and ingress must be rejected when eventual delivery guarantees cannot be upheld.

Cluster coordination needs to be resilient under node failure, network interruption, restart, and storage recovery. The model is TigerBeetle-inspired: deterministic state transitions, explicit quorum requirements, durable replicated decisions, and conservative admission when the cluster cannot prove safety.

ADR 0002 selects TigerBeetle-style Viewstamped Replication (VSR) as the concrete coordination algorithm. This ADR remains the higher-level safety and externally observable coordination model.

This decision must not introduce partition-based APIs or offset-based progress. Client progress remains based on nanosecond timestamp cursors, and subscriber notification tracking remains separate from payload delivery tracking.

## Decision

Kerald clustered mode will use a replicated coordination log for cluster metadata and write-admission decisions. The log records deterministic decisions for membership, topic metadata, broker health, durability commitments, and ingress admission epochs.

The initial cluster model will use the configured expected broker count to calculate quorum. Brokers configure the inter-broker communication port, not a peer-address list. Discovery identifies candidate brokers. The VSR voter set is formed deterministically from durable broker UUIDs and the configured expected broker count, and later changes require committed VSR reconfiguration. Discovery alone must not silently add or remove voters. A write is admissible only when the accepting broker can prove that the coordination quorum and required durability path are available for the current admission epoch. If quorum, storage, or replication health is uncertain, the broker rejects ingress with an explicit unsafe-admission error.

ADR 0002 selects TigerBeetle-style VSR as the broker coordination algorithm. This ADR defines the required safety model and externally observable coordination behavior that VSR must satisfy.

Replication expectations:

- Coordination decisions require a VSR quorum, `floor(expected_brokers / 2) + 1`, from replicas in the current view/voter set.
- A committed coordination decision is durable before it is externally observed.
- Brokers apply committed decisions in log order.
- Broker-local state may cache committed decisions but must not invent authority outside the committed log.
- VSR operation numbers and commit positions are internal replication coordinates and must not become client-visible progress or cursor semantics.

Failure handling:

- Loss of quorum stops new clustered write admission.
- Brokers may continue read or payload-poll paths only when doing so does not violate delivery guarantees.
- Rejoining brokers must replay committed coordination state before accepting clustered writes.
- Stale primaries and brokers in older views are fenced from admitting writes.
- View changes must preserve every operation committed in an earlier view before exposing the new coordination state.
- Conflicting membership or admission epochs are resolved by committed log order, not wall-clock order.

Required observability signals:

- Current VSR view number, primary identity, replica role, expected broker count, and voter-set identity.
- Quorum availability and loss-of-quorum transitions.
- View-change count and duration.
- Stale-view and stale-primary rejection counts.
- Coordination prepare/commit latency and internal replication lag.
- Admission epoch changes.
- Unsafe-admission rejection counts and reasons.
- Broker replay progress after restart or rejoin.

## Alternatives Considered

Partition-owner coordination was rejected because Kerald topics are partitionless and public APIs must not expose partition concepts.

Best-effort gossip-only coordination was rejected because it cannot provide a strong enough basis for safety-first write admission.

Single-leader metadata stored only in local broker state was rejected because it creates a durability and recovery boundary that is too weak for clustered operation.

Offset-based replication progress was rejected because Kerald progress and cursors use nanosecond timestamps where client-visible progress is involved. Internal coordination log positions may exist as implementation details, but they must not become client progress semantics.

## Consequences

Positive consequences:

- Clustered write admission has an explicit safety boundary.
- Brokers converge through deterministic committed coordination state.
- Failure modes are observable and can be tested against concrete quorum and replay behavior.
- Partitionless topic semantics remain intact.

Negative consequences:

- Multi-node operation requires a quorum-capable coordination subsystem before accepting writes.
- Conservative rejection can reduce availability during ambiguous failures.
- Membership changes require careful rollout because they affect quorum and admission epochs.

## Rollout or Migration Notes

Implement single-node cluster operation first with the same admission interface used by multi-node operation. Add multi-node coordination behind an explicit configuration path, initially using expected broker count plus deterministic VSR voter-set bootstrap from durable broker UUIDs discovered through inter-broker communication.

Before enabling expected broker count changes, online membership changes, or voter replacement workflows, add a follow-up ADR for membership changes, reconfiguration safety, and operator workflows.

Add behavior tests for single-node quorum, loss of quorum, broker rejoin, unsafe-admission rejection, and partitionless writes accepted by different brokers in the same cluster.
