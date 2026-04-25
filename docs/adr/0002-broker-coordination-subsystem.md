# ADR 0002: Broker Coordination Subsystem Based on TigerBeetle-Style VSR

Status: Proposed

## Context

ADR 0001 defines Kerald's clustered coordination model: partitionless brokers, deterministic committed coordination state, and safety-first write admission whenever delivery guarantees cannot be proven.

Kerald must now choose the concrete broker coordination subsystem that implements that model. In clustered mode, this subsystem is responsible for:

- Maintaining a single, authoritative ordering of cluster metadata mutations.
- Determining leadership and quorum health for admission control.
- Ensuring that writes are rejected when eventual delivery guarantees cannot be upheld.
- Enabling fast failover with bounded operational complexity.

The system constraints in this repository require consistency and resilience principles inspired by TigerBeetle for coordination decisions. This ADR makes that requirement explicit and defines algorithmic boundaries for implementation.

## Decision

Kerald will implement broker coordination using a **TigerBeetle-style Viewstamped Replication (VSR) approach** as the default cluster coordination algorithm.

Decision boundary:

- **In scope**
  - Replication and agreement for cluster control-plane state, including membership, leader view, fencing epochs, admission state, and configuration changes.
  - Leader-based quorum protocol with deterministic state machine execution.
  - Safety-first ingress gating tied to quorum and replication health.
  - Timestamp-cursor compatibility for downstream delivery tracking without introducing offset semantics.
- **Out of scope**
  - Data-plane payload replication format and storage internals.
  - External API protocol framing for QUIC, gRPC, MQTT, Kafka-compatible front doors, or other protocol extensions.

Protocol-level requirements for the coordination subsystem:

1. **Deterministic state machine**: all admitted control-plane commands must be applied in exactly the same order on all quorum replicas.
2. **Single active leader per view**: fencing must prevent stale leaders from admitting writes.
3. **Quorum-guarded admission**: brokers must reject ingress when quorum durability thresholds are not met.
4. **Recoverable view change**: failover must preserve safety and avoid split-brain commitment.
5. **Efficient batching and pipelining**: implementation should favor high throughput and low overhead, aligned with Kerald's efficiency-first mission.

## Alternatives Considered

1. **Classic Raft**
   - Pros: mature ecosystem, widely understood, abundant operational knowledge.
   - Cons: common implementations often optimize for generality over peak efficiency; less aligned with the TigerBeetle-inspired direction already required by project constraints.

2. **Multi-Paxos / Paxos-family variants**
   - Pros: strong theoretical basis and flexible deployment models.
   - Cons: typically higher implementation and operational complexity; harder to keep ergonomics aligned with Kerald's simplicity goals.

3. **EPaxos / leaderless bleeding-edge approaches**
   - Pros: potential latency gains in some contention or topology patterns.
   - Cons: significantly higher complexity, conflict handling overhead, and operational debugging burden for the control plane.

4. **Centralized coordinator service without consensus**
   - Pros: simpler implementation.
   - Cons: single point of failure and inability to meet safety-first write admission in cluster failure scenarios.

TigerBeetle-style VSR is selected because it balances modern performance-oriented design with deterministic safety properties and a manageable operational model.

## Consequences

Positive consequences:

- Aligns directly with the repository's TigerBeetle-inspired coordination mandate.
- Preserves safety-first admission under partial failures by coupling ingress to quorum health.
- Supports deterministic, auditable control-plane progression and failover semantics.
- Creates a clear boundary between control-plane consensus and partitionless data-plane ingestion.

Negative consequences:

- Smaller off-the-shelf ecosystem than generic Raft libraries; more implementation detail is likely to be in-repo.
- Team onboarding requires familiarity with VSR and view-change semantics.
- Additional rigor is needed for model-based and fault-injection testing of edge cases.

## Rollout or Migration Notes

Standalone mode remains unchanged and does not require consensus.

Cluster mode rollout should proceed behind a coordination feature flag while validation suites are expanded. Required follow-up artifacts:

1. Architecture document for broker coordination message flow and state transitions.
2. Integration tests for leader failover, quorum loss, stale-leader fencing, and recovery.
3. Performance tests for steady-state commit throughput and failover impact.
4. Cucumber behavior scenarios for safety-first ingress rejection during quorum degradation.
5. OTel metrics, traces, and log events for view changes, quorum health, commit latency, and admission rejections.

No historical data migration is required for this ADR alone because it defines control-plane architecture, not persisted data format changes.
