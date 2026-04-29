use arrow_array::{RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use cucumber::{World, given, then, when};
use kerald::{
    AdmissionState, Broker, BrokerConfig, BrokerError, ClusterConfig, DiscoveryState, InterBrokerConfig, MessageNotification,
    TopicDefinition, TopicError,
};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    path::PathBuf,
    sync::Arc,
};

const INVALID_BROKER_CONFIG: &str = "broker configuration values are invalid";

#[derive(Debug, Default, World)]
struct BrokerWorld {
    config: Option<BrokerConfig>,
    config_error: Option<BrokerError>,
    config_path: Option<PathBuf>,
    broker: Option<kerald::RunningBroker>,
    topic: Option<TopicDefinition>,
    topic_error: Option<TopicError>,
    notification: Option<MessageNotification>,
    notifications: Vec<MessageNotification>,
    payloads: Vec<RecordBatch>,
}

#[given("a broker is configured for a single-node cluster")]
async fn single_node_config(world: &mut BrokerWorld) {
    world.config = Some(BrokerConfig::single_node(non_zero_port(9000)));
}

#[given(expr = "the expected cluster size is {int}")]
async fn expected_cluster_size(world: &mut BrokerWorld, expected_brokers: usize) {
    let cluster = ClusterConfig::new(NonZeroUsize::new(expected_brokers).expect("scenario cluster size should be non-zero"));
    let inter_broker = world
        .config
        .as_ref()
        .map(|config| config.inter_broker().clone())
        .unwrap_or_else(|| InterBrokerConfig::new(non_zero_port(9000)));
    world.config = Some(BrokerConfig::new(cluster, inter_broker));
}

#[given(expr = "the inter-broker port is {int}")]
async fn inter_broker_port(world: &mut BrokerWorld, port: u16) {
    let cluster = world
        .config
        .as_ref()
        .map(|config| config.cluster().clone())
        .unwrap_or_else(ClusterConfig::single_node);
    world.config = Some(BrokerConfig::new(cluster, InterBrokerConfig::new(non_zero_port(port))));
}

#[given("a broker configuration file declares inter-broker port zero")]
async fn zero_port_config_file(world: &mut BrokerWorld) {
    world.config_path = Some(cucumber_resource_path("broker-zero-port.json"));
}

#[given("a broker configuration file declares expected cluster size zero")]
async fn zero_expected_brokers_config_file(world: &mut BrokerWorld) {
    world.config_path = Some(cucumber_resource_path("broker-zero-expected-brokers.json"));
}

#[given(expr = "a client requests topic {string}")]
async fn client_requests_topic(world: &mut BrokerWorld, topic_name: String) {
    match TopicDefinition::new(topic_name, order_schema()) {
        Ok(topic) => world.topic = Some(topic),
        Err(error) => world.topic_error = Some(error),
    }
}

#[given(expr = "topic {string} exists")]
async fn topic_exists(world: &mut BrokerWorld, topic_name: String) {
    declare_topic(world, topic_name);
}

#[when(expr = "topic {string} exists")]
async fn topic_exists_after_startup(world: &mut BrokerWorld, topic_name: String) {
    declare_topic(world, topic_name);
}

fn declare_topic(world: &mut BrokerWorld, topic_name: String) {
    let broker = world.broker.as_mut().expect("broker should be started before declaring a topic");
    let topic = TopicDefinition::new(topic_name, order_schema()).expect("scenario topic should be valid");

    broker.declare_topic(topic).expect("topic declaration should succeed");
}

#[when("the broker starts")]
async fn broker_starts(world: &mut BrokerWorld) {
    let config = world.config.take().expect("scenario should configure broker before startup");
    world.broker = Some(Broker::new(config).start().await.expect("broker should start"));
}

#[when("the broker configuration is loaded")]
async fn broker_config_loads(world: &mut BrokerWorld) {
    let path = world.config_path.take().expect("scenario should provide a configuration path");

    match BrokerConfig::from_path(path) {
        Ok(config) => world.config = Some(config),
        Err(error) => world.config_error = Some(error),
    }
}

#[when(expr = "a client publishes order {string} at timestamp {int}")]
async fn client_publishes_order(world: &mut BrokerWorld, order_id: String, timestamp_ns: i64) {
    let broker = world.broker.as_mut().expect("broker should be started before publishing");
    world.notification = Some(
        broker
            .publish("orders.received", timestamp_ns, order_payload(&order_id))
            .expect("message publish should be accepted"),
    );
}

#[when(expr = "a subscriber polls topic {string} after timestamp {int}")]
async fn subscriber_polls_topic(world: &mut BrokerWorld, topic_name: String, after_timestamp_ns: i64) {
    let broker = world.broker.as_ref().expect("broker should be started before polling");
    world.notifications = broker
        .notifications_since(&topic_name, after_timestamp_ns)
        .expect("notification polling should succeed");
    world.payloads = broker
        .payloads_since(&topic_name, after_timestamp_ns)
        .expect("payload polling should succeed");
}

#[then(expr = "the cluster quorum is {int}")]
async fn cluster_quorum_is(world: &mut BrokerWorld, quorum: usize) {
    let broker = world.broker.as_ref().expect("broker should be started");
    assert_eq!(broker.config().cluster().quorum_size().get(), quorum);
}

#[then(expr = "the running broker inter-broker port is {int}")]
async fn running_broker_port_is(world: &mut BrokerWorld, port: u16) {
    let broker = world.broker.as_ref().expect("broker should be started");
    assert_eq!(broker.config().inter_broker().port().get(), port);
}

#[then("the broker configuration is rejected as invalid")]
async fn broker_config_rejected_as_invalid(world: &mut BrokerWorld) {
    assert_eq!(world.config_error, Some(BrokerError::InvalidConfig(INVALID_BROKER_CONFIG)));
    assert!(world.config.is_none());
}

#[then("write admission is enabled for local operation")]
async fn write_admission_enabled(world: &mut BrokerWorld) {
    let broker = world.broker.as_ref().expect("broker should be started");
    assert!(broker.admission_state().admits_writes());
    assert_eq!(broker.admission_state(), &AdmissionState::AcceptingSingleNodeCluster);
}

#[then("a broker UUID is generated")]
async fn broker_uuid_generated(world: &mut BrokerWorld) {
    let broker = world.broker.as_ref().expect("broker should be started");
    assert_ne!(broker.local_node_id().as_uuid(), uuid::Uuid::nil());
}

#[then("write admission is rejected until voter discovery reaches quorum")]
async fn write_admission_rejected_until_discovery_quorum(world: &mut BrokerWorld) {
    let broker = world.broker.as_ref().expect("broker should be started");
    assert!(!broker.admission_state().admits_writes());
    assert_eq!(
        broker.discovery_state(),
        &DiscoveryState::Discovering {
            discovered_voters: NonZeroUsize::MIN,
            required_voters: broker.config().cluster().quorum_size(),
        }
    );
}

#[then(expr = "the topic name is {string}")]
async fn topic_name_is(world: &mut BrokerWorld, topic_name: String) {
    let topic = world.topic.as_ref().expect("topic should be defined");

    assert_eq!(topic.name().as_str(), topic_name);
    assert_eq!(world.topic_error, None);
}

#[then("no partition input is required")]
async fn no_partition_input_required(world: &mut BrokerWorld) {
    assert!(world.topic.is_some());
    assert_eq!(world.topic_error, None);
}

#[then(expr = "the topic Arrow schema contains field {string}")]
async fn topic_schema_contains_field(world: &mut BrokerWorld, field_name: String) {
    let topic = world.topic.as_ref().expect("topic should be defined");

    assert!(topic.schema().fields().iter().any(|field| field.name() == &field_name));
}

#[then(expr = "the accepted message notification timestamp is {int}")]
async fn accepted_notification_timestamp_is(world: &mut BrokerWorld, timestamp_ns: i64) {
    let notification = world.notification.as_ref().expect("publish should return a notification");

    assert_eq!(notification.topic().as_str(), "orders.received");
    assert_eq!(notification.timestamp_ns(), timestamp_ns);
}

#[then(expr = "the subscriber sees {int} notification")]
async fn subscriber_sees_notification_count(world: &mut BrokerWorld, notification_count: usize) {
    assert_eq!(world.notifications.len(), notification_count);
}

#[then(expr = "the subscriber receives {int} Arrow payload batch")]
async fn subscriber_receives_payload_batch_count(world: &mut BrokerWorld, payload_count: usize) {
    assert_eq!(world.payloads.len(), payload_count);
    assert!(world.payloads.iter().all(|payload| payload.schema().field(0).name() == "order_id"));
}

fn non_zero_port(port: u16) -> NonZeroU16 {
    NonZeroU16::new(port).expect("scenario port should be non-zero")
}

fn cucumber_resource_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cucumber")
        .join("resources")
        .join(name)
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("order_id", DataType::Utf8, false)]))
}

fn order_payload(order_id: &str) -> RecordBatch {
    RecordBatch::try_new(order_schema(), vec![Arc::new(StringArray::from(vec![order_id])) as _])
        .expect("scenario payload should be a valid Arrow record batch")
}

#[tokio::main]
async fn main() {
    BrokerWorld::run("tests/cucumber/features").await;
}
