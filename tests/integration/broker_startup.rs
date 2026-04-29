use kerald::{AdmissionState, Broker, BrokerConfig, ClusterConfig, DiscoveryState, InterBrokerConfig};
use std::num::{NonZeroU16, NonZeroUsize};

const COORDINATION_QUORUM_NOT_DISCOVERED: &str = "cluster coordination has not discovered a voting quorum";

fn port(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).expect("test port should be non-zero")
}

#[tokio::test]
async fn single_node_cluster_starts_with_generated_identity_and_local_admission_enabled() {
    let broker = Broker::new(BrokerConfig::single_node(port(9000)))
        .start()
        .await
        .expect("single-node broker should start");

    assert_ne!(broker.local_node_id().as_uuid(), uuid::Uuid::nil());
    assert_eq!(broker.config().cluster().expected_brokers().get(), 1);
    assert_eq!(broker.config().cluster().quorum_size().get(), 1);
    assert_eq!(broker.config().inter_broker().port().get(), 9000);
    assert_eq!(
        broker.discovery_state(),
        &DiscoveryState::Complete {
            discovered_voters: NonZeroUsize::MIN
        }
    );
    assert_eq!(broker.admission_state(), &AdmissionState::AcceptingSingleNodeCluster);
    assert!(broker.admission_state().admits_writes());
}

#[tokio::test]
async fn single_node_cluster_preserves_configured_inter_broker_port_at_startup() {
    let broker = Broker::new(BrokerConfig::single_node(port(9010)))
        .start()
        .await
        .expect("single-node broker should start with configured inter-broker port");

    assert_eq!(broker.config().cluster().quorum_size().get(), 1);
    assert_eq!(broker.config().inter_broker().port().get(), 9010);
    assert_eq!(
        broker.discovery_state(),
        &DiscoveryState::Complete {
            discovered_voters: NonZeroUsize::MIN
        }
    );
    assert!(broker.admission_state().admits_writes());
}

#[tokio::test]
async fn multi_node_cluster_discovers_only_local_voter_at_startup_and_rejects_writes_until_quorum() {
    let broker = Broker::new(BrokerConfig::new(
        ClusterConfig::new(NonZeroUsize::new(3).expect("cluster size should be non-zero")),
        InterBrokerConfig::new(port(9000)),
    ))
    .start()
    .await
    .expect("multi-node broker should start in rejecting admission state");

    assert_ne!(broker.local_node_id().as_uuid(), uuid::Uuid::nil());
    assert_eq!(broker.config().cluster().expected_brokers().get(), 3);
    assert_eq!(broker.config().cluster().quorum_size().get(), 2);
    assert_eq!(broker.config().inter_broker().port().get(), 9000);
    assert_eq!(
        broker.discovery_state(),
        &DiscoveryState::Discovering {
            discovered_voters: NonZeroUsize::MIN,
            required_voters: NonZeroUsize::new(2).expect("required voters should be non-zero"),
        }
    );
    assert_eq!(
        broker.admission_state(),
        &AdmissionState::RejectingUntilCoordinationReady {
            reason: COORDINATION_QUORUM_NOT_DISCOVERED
        }
    );
    assert!(!broker.admission_state().admits_writes());
}
