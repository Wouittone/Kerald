use arrow_schema::{DataType, Field, Schema, SchemaRef};
use cucumber::{World, given, then, when};
use kerald::{
    AdmissionState, Broker, BrokerConfig, BrokerError, ClusterConfig, DiscoveryState, InterBrokerConfig, TopicDefinition, TopicError,
};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::TempDir;

const INVALID_BROKER_CONFIG: &str = "broker configuration values are invalid";

#[derive(Debug, Default, World)]
struct BrokerWorld {
    config: Option<BrokerConfig>,
    config_error: Option<BrokerError>,
    config_path: Option<PathBuf>,
    broker: Option<kerald::RunningBroker>,
    restarted_broker: Option<kerald::RunningBroker>,
    topic: Option<TopicDefinition>,
    topic_error: Option<TopicError>,
    temp_dirs: Vec<TempDir>,
}

#[given("a broker is configured for a single-node cluster")]
async fn single_node_config(world: &mut BrokerWorld) {
    let data_dir = scenario_data_dir(world).to_path_buf();
    world.config = Some(single_node_broker_config(&data_dir, non_zero_port(9000)));
}

#[given(expr = "the expected cluster size is {int}")]
async fn expected_cluster_size(world: &mut BrokerWorld, expected_brokers: usize) {
    let data_dir = scenario_data_dir(world).to_path_buf();
    let cluster = ClusterConfig::with_identity(
        NonZeroUsize::new(expected_brokers).expect("scenario cluster size should be non-zero"),
        "scenario-cluster",
        data_dir,
    );
    let inter_broker = world
        .config
        .as_ref()
        .map(|config| config.inter_broker().clone())
        .unwrap_or_else(|| InterBrokerConfig::new(non_zero_port(9000)));
    world.config = Some(BrokerConfig::new(cluster, inter_broker));
}

#[given(expr = "the inter-broker port is {int}")]
async fn inter_broker_port(world: &mut BrokerWorld, port: u16) {
    let cluster = match world.config.as_ref() {
        Some(config) => config.cluster().clone(),
        None => ClusterConfig::with_identity(NonZeroUsize::MIN, "scenario-cluster", scenario_data_dir(world).to_path_buf()),
    };
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

#[when("the broker starts")]
async fn broker_starts(world: &mut BrokerWorld) {
    let config = world.config.take().expect("scenario should configure broker before startup");
    world.broker = Some(Broker::new(config).start().await.expect("broker should start"));
}

#[when("the broker restarts with the same data directory")]
async fn broker_restarts_with_same_data_directory(world: &mut BrokerWorld) {
    let config = world
        .config
        .as_ref()
        .expect("scenario should configure broker before startup")
        .clone();

    world.broker = Some(
        Broker::new(config.clone())
            .start()
            .await
            .expect("first broker startup should succeed"),
    );
    world.restarted_broker = Some(Broker::new(config).start().await.expect("second broker startup should succeed"));
}

#[when("the broker configuration is loaded")]
async fn broker_config_loads(world: &mut BrokerWorld) {
    let path = world.config_path.take().expect("scenario should provide a configuration path");

    match BrokerConfig::from_path(path) {
        Ok(config) => world.config = Some(config),
        Err(error) => world.config_error = Some(error),
    }
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

#[then("the broker UUID is reused")]
async fn broker_uuid_reused(world: &mut BrokerWorld) {
    let broker = world.broker.as_ref().expect("broker should be started");
    let restarted_broker = world.restarted_broker.as_ref().expect("broker should be restarted");

    assert_eq!(broker.local_node_id(), restarted_broker.local_node_id());
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

fn scenario_data_dir(world: &mut BrokerWorld) -> &Path {
    world.temp_dirs.push(TempDir::new().expect("scenario data dir should be created"));
    world.temp_dirs.last().expect("scenario data dir should be retained").path()
}

fn single_node_broker_config(data_dir: &Path, port: NonZeroU16) -> BrokerConfig {
    BrokerConfig::new(
        ClusterConfig::with_identity(NonZeroUsize::MIN, "scenario-cluster", data_dir),
        InterBrokerConfig::new(port),
    )
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("order_id", DataType::Utf8, false)]))
}

#[tokio::main]
async fn main() {
    BrokerWorld::run("tests/cucumber/features").await;
}
