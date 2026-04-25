# Pull Request Quality Gate

Before requesting review, every Kerald pull request must document:

- Partitionless semantics are preserved.
- Timestamp cursor semantics are preserved.
- Notification tracking remains separate from payload delivery tracking.
- Safety-first write admission is preserved.
- Tests are added or updated in the correct suite: unit, integration, performance, or Cucumber behavior.
- Observable behavior changes include Cucumber coverage or an explicit documented rationale.
- Telemetry impact is reviewed for OpenTelemetry logs, metrics, and traces.
- Requirements, architecture docs, ADRs, and operations notes are updated when the decision surface changes.
- Runtime and container impact is considered, including the musl Alpine multi-stage expectation.
- New Rust crates are actively maintained when added and use the MIT or Apache-2.0 license.

Forbidden patterns:

- Reintroducing partition semantics.
- Reintroducing offset-based progress tracking where timestamp cursors are required.
- Silently degrading admission safety guarantees.
- Adding embedded query-engine responsibilities to broker persistence.
- Adding JNI for Java bindings.
