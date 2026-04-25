# Quality Assurance

Kerald tests are organized into separate suites so each change can be validated at the right level.

- `unit/`: fine-grained module and function behavior.
- `integration/`: cross-module behavior and broker subsystem interactions.
- `performance/`: throughput, latency, and efficiency benchmarks.
- `cucumber/`: externally observable behavior and acceptance scenarios.

Observable behavior changes must update the Cucumber behavior suite or include an explicit documented rationale in the pull request.

Keep tests aligned with Kerald's mandatory invariants: partitionless topics, nanosecond timestamp cursors, separated notification and payload delivery tracking, safety-first write admission, Lance read/write persistence boundaries, and OpenDAL-backed storage abstraction.
