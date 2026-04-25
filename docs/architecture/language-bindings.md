# Language Binding Architecture

Kerald language bindings should wrap the same core broker/client semantics instead of creating language-specific behavior.

Shared binding requirements:

- Preserve partitionless topic APIs.
- Use nanosecond timestamp cursors for progress.
- Keep notification tracking separate from payload delivery tracking.
- Surface safety-first admission rejection as an explicit error.
- Represent payloads through Arrow-compatible data boundaries.

## Python Binding Shape

Python bindings use PyO3 to expose a native Python package for Python 3.10+.

The binding should keep native resource ownership explicit and should avoid broker-side blocking semantics for payload polling. Python APIs may provide ergonomic wrappers, but those wrappers must not change Kerald's delivery, notification, or cursor guarantees.

## Java Binding Shape

Java bindings use Java 25+ FFM API access to native Kerald libraries.

The Java package should manage native memory through FFM resource scopes and must not introduce JNI. Any Java compatibility layer should remain a front door over the same Kerald client semantics rather than a separate protocol model.
