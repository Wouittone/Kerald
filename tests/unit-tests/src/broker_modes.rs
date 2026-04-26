use kerald::{
    BrokerConfig, BrokerError, BrokerNodeId, COORDINATION_QUORUM_NOT_DISCOVERED, ClusterConfig, INVALID_BROKER_NODE_UUID, InterBrokerConfig,
};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    path::PathBuf,
};

fn cluster_size(size: usize) -> NonZeroUsize {
    NonZeroUsize::new(size).expect("test cluster size should be non-zero")
}

fn port(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).expect("test port should be non-zero")
}

#[test]
fn broker_node_id_rejects_non_uuid_values() {
    let error = BrokerNodeId::parse("not-a-uuid").expect_err("node ids must be UUID values");

    assert_eq!(error, BrokerError::InvalidConfig(INVALID_BROKER_NODE_UUID));
}

#[test]
fn broker_node_id_generation_returns_uuid() {
    assert_ne!(BrokerNodeId::generate().as_uuid(), uuid::Uuid::nil());
}

#[test]
fn single_node_cluster_has_quorum_one() {
    let config = ClusterConfig::single_node();

    assert_eq!(config.expected_brokers().get(), 1);
    assert_eq!(config.quorum_size().get(), 1);
    assert!(config.is_single_node());
}

#[test]
fn multi_node_cluster_calculates_majority_quorum() {
    let config = ClusterConfig::new(cluster_size(5));

    assert_eq!(config.quorum_size().get(), 3);
    assert!(!config.is_single_node());
}

#[test]
fn inter_broker_config_uses_only_a_port() {
    let config = InterBrokerConfig::new(port(9000));

    assert_eq!(config.port().get(), 9000);
}

#[test]
fn broker_config_loads_from_toml_resource() {
    let config = BrokerConfig::from_path(resource_path("broker-multi-node.toml")).expect("TOML config should load");

    assert_eq!(config.cluster().expected_brokers().get(), 3);
    assert_eq!(config.cluster().quorum_size().get(), 2);
    assert_eq!(config.inter_broker().port().get(), 9000);
}

#[test]
fn broker_config_loads_from_json_resource() {
    let config = BrokerConfig::from_path(resource_path("broker-single-node.json")).expect("JSON config should load");

    assert_eq!(config.cluster().quorum_size().get(), 1);
    assert_eq!(config.inter_broker().port().get(), 9000);
}

#[test]
fn broker_config_loads_from_yaml_resource() {
    let config = BrokerConfig::from_path(resource_path("broker-multi-node.yaml")).expect("YAML config should load");

    assert_eq!(config.cluster().expected_brokers().get(), 3);
    assert_eq!(config.inter_broker().port().get(), 9002);
}

#[test]
fn rejecting_coordination_uses_static_reason() {
    let reason = COORDINATION_QUORUM_NOT_DISCOVERED;

    assert_eq!(reason, "cluster coordination has not discovered a voting quorum");
}

fn resource_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("unit-tests")
        .join("resources")
        .join("broker")
        .join(name)
}
