use cucumber::{World, given, then, when};
use kerald::{AdmissionState, Broker, BrokerConfig, BrokerError, ClusterConfig, DiscoveryState, InterBrokerConfig};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    path::PathBuf,
};

const INVALID_BROKER_CONFIG: &str = "broker configuration values are invalid";

#[derive(Debug, Default, World)]
struct BrokerWorld {
    config: Option<BrokerConfig>,
    config_error: Option<BrokerError>,
    config_path: Option<PathBuf>,
    broker: Option<kerald::RunningBroker>,
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

#[tokio::main]
async fn main() {
    BrokerWorld::run("tests/cucumber/features").await;
}
