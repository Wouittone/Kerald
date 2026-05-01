# Multi-Agent Development Flow

Kerald uses a specialist-agent workflow to keep development fast while protecting the repository's safety and quality invariants. The canonical project personas live in `.pi/agents/` and can be invoked directly by pi subagents or copied into other agent platforms as role prompts.

## Goals

- Keep one clear orchestrator for scope, decisions, and final synthesis.
- Use focused specialists for architecture, implementation, tests, operations, and review.
- Preserve Kerald's mandatory constraints from `AGENTS.md` on every task.
- Keep writes single-threaded by default; use parallel agents for analysis and review unless worktrees are intentionally used.
- Make quality gates explicit before requesting human review.

## Agent roster

| Agent | Purpose | Primary artifacts |
| --- | --- | --- |
| `kerald-orchestrator` | Scope requests, preserve invariants, route work, synthesize specialist feedback. | Plans, handoff prompts, final summaries. |
| `kerald-requirements-adr-steward` | Maintain requirements, ADRs, architecture notes, and operations docs. | `docs/requirements/`, `docs/adr/`, `docs/architecture/`, `docs/operations/`. |
| `kerald-broker-core-engineer` | Implement Rust broker/core changes. | `core/`, `Cargo.toml`, unit/integration tests. |
| `kerald-coordination-safety-engineer` | Review cluster coordination, quorum, admission safety, and failure modes. | ADRs, broker mode requirements, coordination tests. |
| `kerald-storage-persistence-engineer` | Maintain Arrow/Lance/OpenDAL persistence boundaries. | Storage code, storage docs/tests. |
| `kerald-protocol-bindings-engineer` | Maintain QUIC-front-door and language binding surfaces. | Protocol/bindings docs and tests. |
| `kerald-qa-stability-engineer` | Design and review correct test-suite coverage. | Unit, integration, performance, and Cucumber tests. |
| `kerald-observability-ops-engineer` | Maintain OTel, configuration, CI, containers, rollout, and operations readiness. | Operations docs, telemetry/config/container changes. |
| `kerald-performance-efficiency-engineer` | Protect CPU, memory, network, storage, throughput, and latency. | Performance tests and optimization reviews. |
| `kerald-release-gate-reviewer` | Read-only final PR readiness review. | PR checklist findings and validation evidence. |

## Default workflow

1. **Intake and scope**
   - `kerald-orchestrator` reads `AGENTS.md`, checks `git status`, classifies the request, and states the smallest compliant scope.
   - For architecture or behavior work, it restates relevant constraints: partitionless topics, timestamp cursors, notification/delivery separation, safety-first admission, Lance/OpenDAL persistence, QUIC baseline, binding rules, telemetry, and container expectations.

2. **Specialist context**
   - Route docs/decision questions to `kerald-requirements-adr-steward`.
   - Route implementation to the relevant domain engineer.
   - Route test strategy to `kerald-qa-stability-engineer`.
   - Route production behavior to `kerald-observability-ops-engineer`.
   - Route efficiency-sensitive work to `kerald-performance-efficiency-engineer`.

3. **Plan approval**
   - For non-trivial or decision-surface changes, produce a plan with scope, non-goals, affected files, tests by suite, docs/ADR impact, telemetry impact, and open questions.
   - Ask for approval before implementing if the plan introduces new product, architecture, API, storage, delivery, or coordination decisions.

4. **Single-writer implementation**
   - Keep edits in one writer thread unless worktree-isolated parallel implementation is explicitly chosen.
   - Preserve unrelated user changes.
   - Update required tests/docs/ADRs in the same change set.

5. **Parallel review**
   - Use fresh-context, read-only specialist reviews where useful:
     - correctness/regression: domain engineer or `kerald-broker-core-engineer`
     - test adequacy: `kerald-qa-stability-engineer`
     - operations/telemetry/container: `kerald-observability-ops-engineer`
     - performance: `kerald-performance-efficiency-engineer`
     - final policy: `kerald-release-gate-reviewer`
   - The orchestrator synthesizes findings into blockers, fixes worth doing now, optional improvements, and deferred items.

6. **Validation and closure**
   - Run focused checks first, then CI-equivalent checks when practical:
     - `cargo fmt --check`
     - `cargo clippy --all-targets --all-features -- -D warnings`
     - `cargo test --all --all-features`
   - If validation cannot run, document the blocker and next safest command.
   - End with the PR quality gate checklist from `docs/operations/pr-quality-gate.md`.

## Routing guide

| Change type | Required agents |
| --- | --- |
| Requirements-only or doc-policy change | `kerald-orchestrator`, `kerald-requirements-adr-steward`, `kerald-release-gate-reviewer` |
| Broker runtime/core behavior | `kerald-orchestrator`, `kerald-broker-core-engineer`, `kerald-qa-stability-engineer` |
| Cluster coordination or admission safety | `kerald-orchestrator`, `kerald-coordination-safety-engineer`, `kerald-requirements-adr-steward`, `kerald-qa-stability-engineer` |
| Storage/durability | `kerald-orchestrator`, `kerald-storage-persistence-engineer`, `kerald-qa-stability-engineer`, `kerald-observability-ops-engineer` |
| Protocol or binding API | `kerald-orchestrator`, `kerald-protocol-bindings-engineer`, `kerald-requirements-adr-steward`, `kerald-qa-stability-engineer` |
| Telemetry/config/container/CI | `kerald-orchestrator`, `kerald-observability-ops-engineer`, `kerald-qa-stability-engineer` |
| Performance-sensitive change | `kerald-orchestrator`, `kerald-performance-efficiency-engineer`, domain engineer, `kerald-qa-stability-engineer` |

## Persona prompt maintenance

- Keep persona prompts in `.pi/agents/` concise and role-specific.
- Update this document when adding, renaming, or retiring a persona.
- Do not weaken `AGENTS.md`; persona prompts may add stricter local behavior but cannot override repository policy.
- Prefer read-only final review personas for gatekeeping and one implementation persona for edits.
