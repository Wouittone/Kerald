# Unit Tests

Unit tests cover fine-grained module and function behavior.

Use this suite for deterministic checks of parsing, validation, cursor comparison, TTL precedence, admission preconditions, payload representation helpers, and small storage or protocol boundary utilities.

Unit tests should be fast, isolated, and avoid network or broker process dependencies.

Rust test sources live under `src/`. Static fixture data for unit tests lives under `resources/`.
