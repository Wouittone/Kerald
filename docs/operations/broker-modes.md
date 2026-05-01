# Broker Cluster Startup Operation

Kerald currently exposes a minimal broker cluster startup surface for
development.

Development startup without a config file creates an ephemeral single-node
cluster with a generated broker UUID. This mode is for development only; durable
clustered deployments must use a configured data directory so the broker UUID
and VSR state survive restarts:

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
cluster_id = "dev-cluster"
data_dir = "./kerald-data"

[inter_broker]
port = 9000
```

Configuration values are validated before startup. For example,
`expected_brokers = 0` or an inter-broker port of `0` is rejected as invalid
configuration instead of being replaced with the development default.

Broker IDs are generated automatically only on first initialization and then
persisted in the broker data directory. Peer addresses are not configured
explicitly; brokers discover each other dynamically through inter-broker
communication on the configured port.

Discovery identifies candidate brokers. It does not silently add voters after
VSR bootstrap. A single-node cluster has quorum 1 and may admit local writes
when its durable state is valid. Multi-node clusters start with write admission
disabled until VSR establishes an active view with a fenced primary and quorum
durability. Operators should treat this as an explicit safety signal: the
process is running, but ingress remains rejected while quorum-backed
coordination is unavailable.

Do not reopen existing multi-node data as single-node unless an explicit
recovery or migration procedure says it is safe. Changing a durable broker UUID,
cluster identity, expected broker count, or voter membership without committed
VSR reconfiguration can break quorum safety and must be rejected rather than
silently accepted.
