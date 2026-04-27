use cucumber::{World, given, then, when};
use kerald::{AdmissionState, Broker, BrokerConfig, ClusterConfig, DiscoveryState, InterBrokerConfig};
use std::num::{NonZeroU16, NonZeroUsize};

#[derive(Debug, Default, World)]
struct BrokerWorld {
    config: Option<BrokerConfig>,
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

#[when("the broker starts")]
async fn broker_starts(world: &mut BrokerWorld) {
    let config = world.config.take().expect("scenario should configure broker before startup");
    world.broker = Some(Broker::new(config).start().await.expect("broker should start"));
}

#[then(expr = "the cluster quorum is {int}")]
async fn cluster_quorum_is(world: &mut BrokerWorld, quorum: usize) {
    let broker = world.broker.as_ref().expect("broker should be started");
    assert_eq!(broker.config().cluster().quorum_size().get(), quorum);
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

#[tokio::main]
async fn main() {
    BrokerWorld::run("tests/cucumber/features/broker_modes.feature").await;
}
