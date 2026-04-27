# Operations

This directory contains operational guidance for running, releasing, configuring, and protecting Kerald.

Operational documents should cover deployment expectations, failure handling, rollout impact, production telemetry, CI, and repository governance. Production guidance must account for:

- OpenTelemetry logs, metrics, and traces.
- musl Alpine multi-stage container builds for lightweight runtime nodes.
- Explicit failure-mode handling for write admission, storage, delivery, and cluster coordination.

See `container-images.md` for production image expectations and
`github-merge-gates.md` for repository merge protection and CI requirements.
