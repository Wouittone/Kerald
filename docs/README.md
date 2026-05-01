# Kerald Documentation

Kerald is a distributed messaging framework intended to be easier to use than Kafka-class systems while staying lightweight, operationally simple, and efficiency-first across CPU, memory, network, and storage.

Documentation is organized by decision type:

- `requirements/`: product requirements, externally visible guarantees, and compatibility constraints.
- `adr/`: architecture decision records for long-lived technical decisions.
- `architecture/`: system design notes, component boundaries, protocols, and data-flow explanations.
- `operations/`: configuration, rollout, failure handling, CI, release, production operation guidance, and the multi-agent development flow.

Significant behavior or architecture changes should update the relevant requirements, ADR, test, telemetry, and operations documents as part of the same pull request.
