# ADR 0001: Cluster Coordination Model

Status: Proposed

## Context

Kerald must support both standalone and clustered broker modes while preserving partitionless topics. Any node may accept writes, but ingress must be rejected when eventual delivery guarantees cannot be upheld.

Cluster coordination needs to be resilient under node failure, network interruption, restart, and storage recovery. The model is TigerBeetle-inspired: deterministic state transitions, explicit quorum requirements, durable replicated decisions, and conservative admission when the cluster cannot prove safety.

This decision must not introduce partition-based APIs or offset-based progress. Client progress remains based on nanosecond timestamp cursors, and subscriber notification tracking remains separate from payload delivery tracking.

## Decision

Kerald clustered mode will use a replicated coordination log for cluster metadata and write-admission decisions. The log records deterministic decisions for membership, topic metadata, broker health, durability commitments, and ingress admission epochs.

The initial clustered model will use a fixed voting set configured at cluster creation. A write is admissible only when the accepting broker can prove that the coordination quorum and required durability path are available for the current admission epoch. If quorum, storage, or replication health is uncertain, the broker rejects ingress with an explicit unsafe-admission error.

The specific broker coordination algorithm is selected separately in ADR 0002. This ADR defines the required safety model and externally observable coordination behavior that the algorithm must satisfy.

Replication expectations:

- Coordination decisions require a quorum of the configured voting set.
- A committed coordination decision is durable before it is externally observed.
- Brokers apply committed decisions in log order.
- Broker-local state may cache committed decisions but must not invent authority outside the committed log.

Failure handling:

- Loss of quorum stops new clustered write admission.
- Brokers may continue read or payload-poll paths only when doing so does not violate delivery guarantees.
- Rejoining brokers must replay committed coordination state before accepting clustered writes.
- Conflicting membership or admission epochs are resolved by committed log order, not wall-clock order.

Required observability signals:

- Current coordination role and voting-set membership.
- Quorum availability and loss-of-quorum transitions.
- Coordination commit latency and replication lag.
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

- Clustered mode requires a quorum-capable coordination subsystem before accepting writes.
- Conservative rejection can reduce availability during ambiguous failures.
- Membership changes require careful rollout because they affect quorum and admission epochs.

## Rollout or Migration Notes

Implement standalone mode first with the same admission interface used by clustered mode. Add clustered coordination behind an explicit configuration path, initially with static voting membership.

Before enabling dynamic membership, add a follow-up ADR for membership changes, reconfiguration safety, and operator workflows.

Add behavior tests for loss of quorum, broker rejoin, unsafe-admission rejection, and partitionless writes accepted by different brokers in the same cluster.
