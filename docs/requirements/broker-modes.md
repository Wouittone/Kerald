# Broker Cluster Startup

Kerald brokers start from cluster configuration. A single-node cluster is the
lightweight local mode; there is no separate standalone broker mode.

## Broker Identity

Every broker node id is a UUID. Human-readable aliases may be added later for
operator ergonomics, but durable broker identity uses UUID values. Broker node
IDs are generated automatically at startup and are not configured manually.

## Cluster Configuration

Cluster configuration must declare:

- The expected broker count.
- Inter-broker communication settings, including the port each broker uses for
  discovery traffic.

All discovered brokers in the cluster are voters. Voter membership is determined
dynamically through inter-broker communication rather than through a configured
voter or peer-address list.

A single-node cluster has quorum 1 and may accept local writes once the local
broker is running and no other safety precondition is violated.

A multi-node cluster must reject write admission until the coordination subsystem
has discovered enough voting brokers to prove quorum health. This preserves the
safety-first admission rule while the VSR coordination layer is implemented
behind the cluster boundary.

The cluster startup model must preserve the same client-facing guarantees across
all cluster sizes: topics are partitionless, progress uses nanosecond timestamp
cursors, and notification tracking remains independent from payload delivery
tracking.

Broker lifecycle APIs are async-first. Startup is fallible and awaited by the
runtime owner so future QUIC, coordination, storage, polling, telemetry, and
shutdown work can be initialized without blocking worker threads.

## Configuration Files

Broker configuration is loaded through a parser that supports multiple
configuration languages, including TOML, JSON, and YAML.

Configuration files that load successfully but contain invalid values, such as a
zero expected broker count or zero inter-broker port, must fail as invalid
configuration rather than being treated as an admission-state fallback. Startup
must not silently replace invalid cluster settings with defaults.

## Test Coverage

Unit tests cover UUID validation, cluster size and quorum behavior, inter-broker
configuration validation, TOML/JSON/YAML config loading, and invalid value
rejection. Integration tests cover startup state for single-node and multi-node
clusters. Cucumber behavior coverage documents the externally observable startup
and configuration rejection expectations.
