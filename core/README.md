# Kerald Core

Rust broker and core library implementation lives under this directory.

The initial core surface defines cluster startup configuration. A single-node
cluster is the lightweight local mode, while multi-node clusters keep write
admission conservative until the coordination subsystem discovers quorum.
