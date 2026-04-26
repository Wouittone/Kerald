# Integration Tests

Integration tests cover cross-module behavior and broker subsystem interactions.

Use this suite when behavior depends on multiple components working together, such as broker startup, write admission with storage, subscriber progress tracking, protocol ingress, or single-node and multi-node cluster subsystem wiring.

Integration tests may use local processes and local filesystem storage, but should keep external service dependencies explicit.
