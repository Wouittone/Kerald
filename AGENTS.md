# AGENTS.md — Strict Operating Rules for Kerald

This file defines **mandatory** rules for any agent operating in this repository.

## 1) Product Mission (Non-Negotiable)
Build Kerald as a distributed messaging framework that is:
- Easier to use than Kafka-class systems.
- Lightweight and operationally simple.
- Efficiency-first (CPU, memory, network, storage), while preserving strong performance.

## 2) Core Architecture Constraints (MUST)
1. Support both single-node and multi-node cluster operation.
2. Cluster coordination follows TigerBeetle-inspired consistency and resilience principles.
3. Topics are **partitionless**:
   - Any node can accept writes.
   - No partition-based API or internal assumptions.
4. Safety-first write admission:
   - If eventual delivery guarantees cannot be upheld, reject ingress.
5. Subscriber semantics are split:
   - Notification tracking is independent from payload delivery tracking.
   - Clients can poll payloads independently without broker-side blocking semantics.
6. Progress/cursors use **nanosecond timestamps**, not offsets.
7. TTL precedence is fixed:
   - Cluster-level default < Topic-level override < Message-level override.
8. Protocol baseline is QUIC; extension points for gRPC, Arrow ADBC, MQTT, and Kafka-compatible front doors.
9. Payload representation is Arrow.
10. Persistence is Lance read/write only (no embedded LanceDB query responsibilities in brokers).
11. Storage abstraction must use OpenDAL (local FS, S3, R2, GCS, Azure Blob support path).
12. Language/runtime targets:
   - Rust 1.95
   - Python 3.10+ bindings via PyO3
   - Java 25+ bindings via FFM API (no JNI)
13. Production telemetry must include OTel logs, metrics, and traces.

## 3) Testing Strategy (MUST)
All testing work must be organized into separate suites:
- **Unit tests**: fine-grained module and function behavior.
- **Integration tests**: cross-module and broker subsystem interactions.
- **Performance tests**: throughput/latency and efficiency benchmarks.
- **Behavior tests (Cucumber)**: externally observable system behavior and acceptance scenarios.

Do not merge changes that introduce observable behavior without corresponding behavior-test coverage updates (or an explicit documented rationale).

## 4) Container & Build Strategy (MUST)
To keep nodes lightweight, containerization must target:
- **musl Alpine Docker images**
- **Multi-stage builds** for minimal runtime footprint and efficient layering.

Any production Dockerfile work should:
- Build artifacts in a dedicated builder stage.
- Copy only required runtime binaries/assets into the final stage.
- Avoid unnecessary tooling in the final image.

## 5) Required Deliverables for Significant Changes
For non-trivial features or behavior changes, include all applicable artifacts:
1. Requirements/spec updates.
2. ADR(s) for significant architecture/technical decisions.
3. Test updates across the appropriate test suite(s).
4. Observability updates (logs/metrics/traces).
5. Operational notes (configuration, failure mode handling, rollout impact).

## 6) ADR Discipline (Strict)
Create or update ADRs when introducing:
- Consistency/coordination changes.
- Protocol model changes.
- Storage/durability boundary changes.
- Delivery/notification semantics changes.
- API changes with long-term compatibility impact.

Each ADR must include:
- Context
- Decision
- Alternatives considered
- Consequences (positive/negative)
- Rollout or migration notes when applicable

## 7) Branching, Commits, and Scope Control
- One atomic concern per branch and PR.
- Branch naming: `type/scope-short-description`.
- Conventional commits required.
- No hidden scope creep; park non-essential work as explicit backlog items.

## 8) Forbidden Patterns
Agents MUST NOT:
- Reintroduce partition semantics.
- Reintroduce offset-based progress tracking where timestamp cursors are required.
- Silently degrade admission safety guarantees.
- Add embedded query-engine responsibilities to broker persistence path.
- Add JNI for Java bindings.

## 9) PR Quality Gate Checklist (MANDATORY)
Before requesting review, verify and document:
- [ ] Partitionless semantics preserved.
- [ ] Timestamp cursor semantics preserved.
- [ ] Notification vs delivery separation preserved.
- [ ] Safety-first write admission preserved.
- [ ] Tests added/updated in correct suite(s): unit/integration/performance/behavior.
- [ ] Telemetry impact reviewed (OTel logs/metrics/traces).
- [ ] Docs/ADR updated if decision surface changed.
- [ ] Runtime/container impact considered (musl Alpine multi-stage expectation).

## 10) Repository Documentation Layout (Preferred)
- `docs/requirements/`
- `docs/adr/`
- `docs/architecture/`
- `docs/operations/`
- `tests/unit-tests/`
- `tests/integration/`
- `tests/performance/`
- `tests/cucumber/`

## 11) Agent Operating Procedure
1. Read this AGENTS.md before planning or editing.
2. Restate constraints when proposing architectural changes.
3. Make conservative assumptions around delivery guarantees.
4. Prefer explicitness and deterministic behavior.
5. If a requested change conflicts with this policy, flag the conflict and propose compliant alternatives.
