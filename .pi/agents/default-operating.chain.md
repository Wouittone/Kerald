---
name: default-operating
description: Default Kerald operating workflow using every project custom agent, and only project custom agents.
---

## kerald-orchestrator

Lead: classify current repository scope, preserve Kerald invariants, and create a concrete task distribution plan. Do not code. Include current uncommitted changes and recent commits in context. Assign coding tasks to coding specialists, validation to QA, docs/ADR to the steward, ops to ops, readability/bindings/performance reviews to the relevant reviewers, and final review to release gate. Output explicit assignments, non-goals, stop conditions, and the invariant checklist.

## kerald-requirements-adr-steward

Using the orchestrator plan below, identify and apply only required requirements/docs/ADR updates for architecture, protocol, storage, delivery, API, coordination, or long-term compatibility decisions. If this is compile/test maintenance with no docs requirement, do not edit; provide doc/ADR risk notes only.

Orchestrator plan:
{previous}

## kerald-broker-core-engineer

Implement only broker/core Rust changes assigned by the orchestrator plan and refined by prior output below. If no broker/core coding work is assigned, do not edit; provide review/risks only. Preserve partitionless topics, timestamp cursors, notification/delivery separation, and safety-first admission.

Prior output:
{previous}

## kerald-storage-persistence-engineer

Implement only Lance/OpenDAL persistence changes assigned by the operating plan and prior output below. If no storage coding work is assigned, do not edit; provide review/risks only. Preserve Lance read/write-only persistence and avoid broker query-engine responsibilities.

Prior output:
{previous}

## kerald-coordination-safety-engineer

Review coordination, quorum, failure-mode, safety-first admission, and TigerBeetle-inspired consistency implications for the operating plan and prior output below. Do not edit unless explicitly assigned a tiny in-scope docs/test clarification.

Prior output:
{previous}

## kerald-protocol-bindings-engineer

Review and implement only protocol or binding-surface work assigned by the operating plan and prior output below. Preserve QUIC as the baseline, Python via PyO3, Java 25+ via FFM API with no JNI, and compatibility extension points. If no protocol/binding work is assigned, do not edit; provide risks only.

Prior output:
{previous}

## kerald-python-bindings-junior-engineer

Validate Python binding ergonomics, examples, docs, and ease of use for the operating plan and prior output below. Do not edit unless explicitly assigned tiny Python-binding docs/example fixes. Report confusing APIs, missing examples, and user-facing risks.

Prior output:
{previous}

## kerald-java-bindings-junior-engineer

Validate Java 25 FFM binding ergonomics, examples, docs, resource ownership, and ease of use for the operating plan and prior output below. Do not edit unless explicitly assigned tiny Java-binding docs/example fixes. Report confusing APIs, missing examples, and user-facing risks.

Prior output:
{previous}

## kerald-performance-efficiency-engineer

Review CPU, memory, network, storage, throughput, latency, and operational simplicity implications for the operating plan and prior output below. Do not edit unless explicitly assigned a tiny in-scope benchmark/test/docs clarification. Preserve efficiency-first design without weakening safety.

Prior output:
{previous}

## kerald-observability-ops-engineer

Review OTel logs/metrics/traces, configuration, operations, rollout, CI/container impact, and musl Alpine multi-stage expectations for the operating plan and prior output below. Edit only if explicitly assigned docs/ops work by the plan.

Prior output:
{previous}

## kerald-qa-stability-engineer

Own validation for the operating plan and prior output below: unit, integration, performance, and Cucumber behavior coverage; focused test runs; and test fixes only when assigned. Validate current in-scope changes and make only tiny in-scope test-code fixes if required.

Prior output:
{previous}

## kerald-polyglot-readability-reviewer

Review the current diff and prior specialist output for readability, maintainability, comprehensibility, and appropriate functional-style simplification across Rust/Python/Java-facing code. Do not weaken Kerald invariants. Do not edit unless explicitly assigned tiny readability-only fixes.

Prior output:
{previous}

## kerald-release-gate-reviewer

Final independent quality-gate review. Inspect the current diff and the full specialist output below. Check that only project custom agents were used, all Kerald invariants are preserved, validation is adequate, docs/ADR/test coverage is appropriate, telemetry impact is reviewed, and residual risks are explicit. Do not edit.

Specialist outputs:
{previous}
