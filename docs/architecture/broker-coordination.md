# Broker Coordination Architecture

Kerald clustered coordination uses TigerBeetle-style Viewstamped Replication (VSR) for control-plane agreement and safety-first write admission. This document describes the architecture shape selected by ADR 0002. It does not define the data-plane payload replication format or storage internals.

## Scope and invariants

VSR coordinates cluster control-plane state: the current view, primary identity, voter set, fencing epoch, admission epoch, topic metadata decisions, storage-health gates, and replay progress. Payloads remain Arrow values persisted through Lance read/write paths behind OpenDAL-backed storage. Brokers must not expose VSR operation numbers, commit positions, primary ownership, partitions, or offsets through client APIs.

Any broker may receive a producer request for a partitionless topic. A broker may acknowledge/admit the write only after the active VSR view can prove quorum health and the required payload/notification durability path is safe. If that proof is unavailable, ingress is rejected with an explicit unsafe-admission reason.

## Component schema

```mermaid
flowchart LR
    producer[Producer client] -->|QUIC baseline\nfuture front doors| ingress[Any Kerald broker]

    subgraph cluster[Kerald cluster]
        ingress --> admission[Admission gate]
        admission -->|if backup/non-primary| primary[Current VSR primary]
        admission -->|if primary| primary
        primary --> log[VSR coordination log]
        primary --> backups[VSR backup replicas]
        backups --> log
        log --> state[Deterministic coordination state]
        state --> durability[Durability proof\nLance + OpenDAL]
        durability --> ack[Acknowledge/admit write]
        durability --> reject[Reject unsafe admission]
    end

    state -.internal only.-> topic[Partitionless topic metadata]
    ack --> producer
    reject --> producer
```

## Replica roles and state transitions

```mermaid
stateDiagram-v2
    [*] --> Discovering
    Discovering --> Recovering: durable identity found\nand candidate peers discovered
    Recovering --> ViewChanging: replay complete\nno active safe view
    ViewChanging --> Primary: elected for new view\nwith quorum proof
    ViewChanging --> Backup: accepted start-view\nfrom primary
    Primary --> Rejecting: quorum lost\nor storage unsafe
    Backup --> Rejecting: stale view\nor replay incomplete
    Rejecting --> ViewChanging: higher view required
    Rejecting --> Recovering: restart/rejoin
    Primary --> ViewChanging: progress stalls\nor higher view observed
    Backup --> ViewChanging: primary suspected
```

Roles:

- `Discovering`: finds candidate brokers through inter-broker communication. Discovery alone does not mutate voter membership.
- `Recovering`: reloads durable broker identity, VSR log, commit point, and coordination state.
- `ViewChanging`: exchanges durable log and commit metadata to establish a higher view safely.
- `Primary`: orders control-plane commands for the active view and commits after quorum acknowledgement.
- `Backup`: persists valid prepares from the active primary and rejects stale-view messages.
- `Rejecting`: process may be running, but write admission is unavailable until safety can be proven.

## Normal operation sequence

```mermaid
sequenceDiagram
    participant C as Client
    participant B as Receiving broker
    participant P as VSR primary
    participant R1 as Backup replica
    participant R2 as Backup replica
    participant S as Lance/OpenDAL durability path

    C->>B: Produce(topic, Arrow payload)
    B->>B: Validate topic schema and admission preconditions
    alt B is not current primary
        B->>P: Forward admission command
    else B is current primary
        B->>P: Enter local admission command
    end
    P->>S: Verify payload/notification durability path
    alt durability is unsafe
        P-->>B: Unsafe-admission rejection, no admit decision committed
        B-->>C: Reject write
    else durability path is safe
        P->>P: Assign internal VSR operation number
        P->>R1: Prepare(view, op, admit command)
        P->>R2: Prepare(view, op, admit command)
        R1-->>P: PrepareOk after durable append
        R2-->>P: PrepareOk after durable append
        P->>P: Commit admit decision after quorum
        P-->>B: Admission committed
        B-->>C: Acknowledge write
    end
```

## Durable coordination log record schema

```mermaid
classDiagram
    class VsrLogRecord {
        ClusterId cluster_id
        BrokerId replica_id
        uint64 view
        uint64 operation_number
        uint64 commit_number
        CoordinationCommand command
        Checksum previous_record_checksum
        Checksum record_checksum
    }

    class CoordinationCommand {
        CommandId request_id
        CommandType type
        AdmissionEpoch admission_epoch
        FencingToken fencing_token
        TimestampNs proposed_timestamp
    }

    class CoordinationState {
        ClusterId cluster_id
        BrokerId primary_id
        uint64 view
        uint64 commit_number
        AdmissionEpoch admission_epoch
        FencingToken fencing_token
        VoterSet voter_set
        StorageHealth storage_health
        ReplayPoint replay_point
    }

    VsrLogRecord --> CoordinationCommand
    VsrLogRecord --> CoordinationState : applied in order to produce
```

The `operation_number` and `commit_number` fields are internal replication coordinates. They must never become client-visible progress, cursor, delivery, or polling semantics. Client progress remains nanosecond timestamp based.

## Admission decision schema

```mermaid
flowchart TD
    req[Write request\nrequest id + topic + Arrow payload] --> schema[Validate topic name\nand Arrow schema]
    schema --> view{Active VSR view\nwith fenced primary?}
    view -- no --> reject_view[Reject: unknown or stale view]
    view -- yes --> quorum{Quorum durable\nfor current voter set?}
    quorum -- no --> reject_quorum[Reject: quorum unavailable]
    quorum -- yes --> replay{Local/primary replay\ncomplete?}
    replay -- no --> reject_replay[Reject: replay incomplete]
    replay -- yes --> storage{Payload and notification\ndurability path safe?}
    storage -- no --> reject_storage[Reject: storage unsafe]
    storage -- yes --> commit[Commit admission command\nand assign timestamp cursor]
    commit --> admit[Admit/acknowledge write]
```

Admission timestamp assignment must be deterministic and durable. The timestamp cursor is distinct from VSR operation numbers and remains the only client-visible progress coordinate.

## Membership and discovery

Inter-broker discovery finds candidate brokers; it does not silently add voters. Initial VSR bootstrap forms the voter set deterministically from durable broker UUIDs and the configured expected broker count. Membership changes after bootstrap require committed VSR reconfiguration records and a follow-up ADR before implementation.

Durable broker identity is required for VSR recovery. A broker UUID is generated automatically only on first initialization, persisted with the broker's data, and reused across restarts. A changed UUID represents a different replica and must not silently rejoin an existing voter set.

## Required observability

Production telemetry must include OTel logs, metrics, and traces for:

- current VSR view, primary identity, replica role, and voter-set identity;
- view-change count, duration, and reason;
- quorum availability transitions;
- stale-view and stale-primary rejection counts;
- prepare, commit, and admission latency;
- internal coordination replication lag and replay progress;
- unsafe-admission rejection counts by reason;
- storage-durability health used by admission decisions.

## Required validation coverage

- Unit tests: quorum math, view comparison, fencing token validation, durable identity reuse, voter-set bootstrap validation, admission-state transitions, and timestamp/operation-number separation.
- Integration tests: active view establishment, normal quorum commit path, stale-primary fencing, quorum loss, broker restart/replay, and non-primary write forwarding.
- Cucumber behavior tests: multi-node ingress rejection until VSR quorum/view is healthy, partitionless writes through any broker without exposing primary or partition concepts, and unsafe-admission rejection during quorum degradation.
- Performance tests: steady-state control-plane commit throughput, batching/pipelining latency, forwarded admission latency, failover blackout duration, replay/rejoin time, and CPU/memory/network overhead per committed command.
