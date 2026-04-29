use arrow_array::{RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use kerald::{AdmissionState, Broker, BrokerConfig, ClusterConfig, DiscoveryState, InterBrokerConfig, TopicDefinition};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    sync::Arc,
};

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

#[tokio::test]
async fn partitionless_topic_metadata_is_independent_of_cluster_size() {
    let single_node = Broker::new(BrokerConfig::single_node(port(9000)))
        .start()
        .await
        .expect("single-node broker should start");
    let multi_node = Broker::new(BrokerConfig::new(
        ClusterConfig::new(NonZeroUsize::new(3).expect("cluster size should be non-zero")),
        InterBrokerConfig::new(port(9001)),
    ))
    .start()
    .await
    .expect("multi-node broker should start");

    let topic = TopicDefinition::new("orders.received", order_schema()).expect("topic definition should be valid");

    assert_eq!(topic.name().as_str(), "orders.received");
    assert_eq!(topic.schema().field(0).name(), "order_id");
    assert!(single_node.config().cluster().is_single_node());
    assert!(!multi_node.config().cluster().is_single_node());
    assert!(single_node.admission_state().admits_writes());
    assert!(!multi_node.admission_state().admits_writes());
}

#[tokio::test]
async fn single_node_broker_accepts_a_message_and_exposes_notification_and_payload_progress_independently() {
    let mut broker = Broker::new(BrokerConfig::single_node(port(9000)))
        .start()
        .await
        .expect("single-node broker should start");
    let topic = TopicDefinition::new("orders.received", order_schema()).expect("topic definition should be valid");
    broker.declare_topic(topic).expect("topic declaration should succeed");

    let notification = broker
        .publish("orders.received", 1_700_000_000_000_000_000, order_payload("order-1"))
        .expect("single-node broker should accept the message");

    assert_eq!(notification.topic().as_str(), "orders.received");
    assert_eq!(notification.timestamp_ns(), 1_700_000_000_000_000_000);
    assert_eq!(notification.row_count(), 1);

    let notifications = broker
        .notifications_since("orders.received", 0)
        .expect("notification polling should succeed");
    let payloads = broker.payloads_since("orders.received", 0).expect("payload polling should succeed");

    assert_eq!(notifications.len(), 1);
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].num_rows(), 1);
    assert_eq!(payloads[0].schema().field(0).name(), "order_id");
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("order_id", DataType::Utf8, false)]))
}

fn order_payload(order_id: &str) -> RecordBatch {
    RecordBatch::try_new(order_schema(), vec![Arc::new(StringArray::from(vec![order_id])) as _])
        .expect("test payload should be a valid Arrow record batch")
}
