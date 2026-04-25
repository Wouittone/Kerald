# Runtime and Binding Targets

Kerald targets these language and runtime boundaries:

- Rust 1.92 for broker and core library implementation.
- Python 3.10+ bindings through PyO3.
- Java 25+ bindings through the FFM API.

JNI is not allowed for Java bindings.

## Rust

Rust is the implementation language for Kerald's broker and core libraries. Repository build configuration should pin or document Rust 1.92 before Rust source is introduced.

New Rust crates may be added only when they are actively maintained at the time they are added and licensed under MIT or Apache-2.0.

## Python

Python bindings must use PyO3 and support Python 3.10 or newer.

The Python binding boundary should expose client-facing APIs over stable Kerald concepts: topics, timestamp cursors, payload polling, notifications, and admission errors. It must not expose partition APIs or offset-based progress.

## Java

Java bindings must target Java 25 or newer and use the Foreign Function and Memory API.

Java bindings must not use JNI. Any build or source layout that introduces JNI headers, JNI loaders, or JNI bridge code is incompatible with Kerald's binding policy.

## CI Expectations

The repository CI already includes conditional Rust, Python, and Java build jobs. Those jobs activate when the matching project manifests are added.

When implementation files are introduced, the same pull request should add the relevant manifest, build command, and test coverage so CI enforces the target runtime.
