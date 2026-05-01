# Broker Cluster Startup

Kerald brokers start from cluster configuration. A single-node cluster is the
lightweight local mode; there is no separate standalone broker mode.

## Broker Identity

Every broker node id is a UUID. Human-readable aliases may be added later for
operator ergonomics, but durable broker identity uses UUID values. Broker node
IDs are generated automatically only when no durable broker identity exists.
Once generated, the UUID must be persisted and reused across restarts.
Regenerating a UUID makes the process a new broker identity and must not
silently rejoin an existing VSR replica set.

## Cluster Configuration

Cluster configuration must declare:

- The expected broker count.
- The cluster identity used to reject accidental cross-cluster discovery.
- The broker data directory used to persist broker identity and VSR state.
- Inter-broker communication settings, including the port each broker uses for
  discovery traffic.

Inter-broker discovery identifies candidate brokers; it does not by itself
change the VSR voter set. At bootstrap, the initial voter set is formed
deterministically from the configured expected broker count and unique durable
broker UUIDs discovered through inter-broker communication. Discovery after
bootstrap must not silently add or remove voters. Membership changes require an
explicit committed VSR reconfiguration decision and a follow-up ADR before
implementation. Operators do not configure a peer-address list.

A single-node cluster has quorum 1 and may accept local writes once the local
broker is running, its durable identity/state is valid, and no other safety
precondition is violated. Single-node behavior should still use the same
coordination/admission interface as multi-node behavior, as a degenerate VSR
view with quorum 1.

A multi-node cluster must reject write admission until VSR has established an
active view with a fenced primary and a quorum of durable replicas for the
configured expected broker count. Non-primary brokers may receive client write
requests only if they route them through the current VSR admission/commit path
before acknowledgement. This preserves the safety-first admission rule while the
VSR coordination layer remains internal to the broker boundary.

The cluster startup model must preserve the same client-facing guarantees across
all cluster sizes: topics are partitionless, progress uses nanosecond timestamp
cursors, and notification tracking remains independent from payload delivery
tracking. VSR operation numbers and commit positions are internal replication
coordinates and must never become client-visible cursor or delivery semantics.

## Startup and Admission States

Broker startup and health reporting must distinguish at least these states:

- `single-node-ready`: quorum 1 is established and local durable state is valid.
- `discovering`: candidate brokers are being discovered, but no safe active view
  exists.
- `recovering`: durable identity, log, and committed coordination state are
  being replayed.
- `quorum-ready`: an active VSR view has a fenced primary and quorum durability.
- `degraded-rejecting`: the process is running, but write admission is rejected
  because safety cannot be proven.

Write admission must be rejected for no quorum, unknown view, stale primary,
uncommitted membership, incomplete replay, unavailable storage durability,
duplicate broker identity, or cluster identity mismatch.

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

Unit tests cover UUID validation, durable broker UUID reuse, cluster size and
quorum behavior, VSR voter-set bootstrap validation, inter-broker configuration
validation, TOML/JSON/YAML config loading, and invalid value rejection.
Integration tests cover startup state for single-node and multi-node clusters,
active-view establishment, stale-primary fencing, quorum loss, broker
rejoin/replay, and non-primary write forwarding without exposing partition
concepts. Cucumber behavior coverage documents the externally observable
startup, unsafe-admission rejection, quorum degradation, failover, and
configuration rejection expectations.
