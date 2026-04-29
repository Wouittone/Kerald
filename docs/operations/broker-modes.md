# Broker Cluster Startup Operation

Kerald currently exposes a minimal broker cluster startup surface for
development.

Development startup without a config file creates an ephemeral single-node
cluster with a generated broker UUID:

```sh
cargo run
```

Configured startup uses `--config` and may load TOML, JSON, or YAML:

```sh
cargo run -- --config kerald.toml
```

Example TOML configuration:

```toml
[cluster]
expected_brokers = 3

[inter_broker]
port = 9000
```

Configuration values are validated before startup. For example,
`expected_brokers = 0` or an inter-broker port of `0` is rejected as invalid
configuration instead of being replaced with the development default.

Broker IDs are generated automatically. Peer addresses are not configured
explicitly; brokers discover each other dynamically through inter-broker
communication on the configured port.

All discovered brokers are voters. A single-node cluster has quorum 1 and may
admit local writes. Multi-node clusters start with write admission disabled until
coordination discovers a voting quorum. Operators should treat this as an
explicit safety signal: the process is running, but ingress remains rejected
while quorum-backed coordination is unavailable.
