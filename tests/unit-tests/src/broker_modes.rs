use arrow_schema::{DataType, Field, Schema, SchemaRef};
use kerald::{
    BrokerConfig, BrokerError, BrokerNodeId, ClusterConfig, InterBrokerConfig, TOPIC_NAME_MAX_LEN_BYTES, TopicDefinition, TopicError,
    parse_topic_name,
};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    path::PathBuf,
    sync::Arc,
};

const CONFIG_LOAD_FAILED: &str = "configuration file could not be loaded";
const COORDINATION_QUORUM_NOT_DISCOVERED: &str = "cluster coordination has not discovered a voting quorum";
const INVALID_BROKER_CONFIG: &str = "broker configuration values are invalid";
const INVALID_BROKER_NODE_UUID: &str = "broker node id must be a UUID";
const EMPTY_TOPIC_NAME: &str = "topic name must not be empty";
const INVALID_TOPIC_NAME_CHARACTER: &str = "topic name must contain only ASCII letters, numbers, dots, underscores, or hyphens";
const TOPIC_NAME_TOO_LONG: &str = "topic name must be at most 255 bytes";

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
    assert_eq!(config.cluster_id(), "kerald-dev");
    assert_eq!(config.data_dir(), std::path::Path::new("kerald-data"));
    assert!(config.is_single_node());
}

#[test]
fn cluster_config_carries_durable_identity_settings() {
    let config = ClusterConfig::with_identity(cluster_size(3), "orders-prod", "data/orders-prod");

    assert_eq!(config.expected_brokers().get(), 3);
    assert_eq!(config.cluster_id(), "orders-prod");
    assert_eq!(config.data_dir(), std::path::Path::new("data/orders-prod"));
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
    assert_eq!(config.cluster().cluster_id(), "test-cluster");
    assert_eq!(config.cluster().data_dir(), std::path::Path::new("target/test-data/toml-broker"));
    assert_eq!(config.inter_broker().port().get(), 9000);
}

#[test]
fn broker_config_loads_from_json_resource() {
    let config = BrokerConfig::from_path(resource_path("broker-single-node.json")).expect("JSON config should load");

    assert_eq!(config.cluster().quorum_size().get(), 1);
    assert_eq!(config.cluster().cluster_id(), "test-cluster");
    assert_eq!(config.cluster().data_dir(), std::path::Path::new("target/test-data/json-broker"));
    assert_eq!(config.inter_broker().port().get(), 9000);
}

#[test]
fn broker_config_loads_from_yaml_resource() {
    let config = BrokerConfig::from_path(resource_path("broker-multi-node.yaml")).expect("YAML config should load");

    assert_eq!(config.cluster().expected_brokers().get(), 3);
    assert_eq!(config.cluster().cluster_id(), "test-cluster");
    assert_eq!(config.cluster().data_dir(), std::path::Path::new("target/test-data/yaml-broker"));
    assert_eq!(config.inter_broker().port().get(), 9002);
}

#[test]
fn broker_config_reports_missing_file_as_load_failure() {
    let error = BrokerConfig::from_path(resource_path("missing.toml")).expect_err("missing config should fail to load");

    assert_eq!(error, BrokerError::ConfigLoad(CONFIG_LOAD_FAILED));
}

#[test]
fn broker_config_rejects_zero_expected_brokers() {
    let error = BrokerConfig::from_path(resource_path("broker-zero-expected-brokers.json"))
        .expect_err("zero expected broker count should be invalid");

    assert_eq!(error, BrokerError::InvalidConfig(INVALID_BROKER_CONFIG));
}

#[test]
fn broker_config_rejects_zero_inter_broker_port() {
    let error = BrokerConfig::from_path(resource_path("broker-zero-port.json")).expect_err("port zero should be invalid");

    assert_eq!(error, BrokerError::InvalidConfig(INVALID_BROKER_CONFIG));
}

#[test]
fn rejecting_coordination_uses_static_reason() {
    let reason = COORDINATION_QUORUM_NOT_DISCOVERED;

    assert_eq!(reason, "cluster coordination has not discovered a voting quorum");
}

#[test]
fn topic_names_are_trimmed_and_validated() {
    let name = parse_topic_name(" orders.received_v1 ").expect("topic name should be valid");

    assert_eq!(name.as_str(), "orders.received_v1");
    assert_eq!(name.to_string(), "orders.received_v1");
}

#[test]
fn topic_names_reject_empty_values() {
    let error = parse_topic_name(" ").expect_err("empty topic names should be rejected");

    assert_eq!(error, TopicError::InvalidName(EMPTY_TOPIC_NAME));
}

#[test]
fn topic_names_reject_unsupported_characters() {
    let error = parse_topic_name("orders/received").expect_err("topic names should not accept path separators");

    assert_eq!(error, TopicError::InvalidName(INVALID_TOPIC_NAME_CHARACTER));
}

#[test]
fn topic_names_reject_values_longer_than_255_bytes() {
    let error = parse_topic_name("a".repeat(TOPIC_NAME_MAX_LEN_BYTES + 1)).expect_err("oversized topic names should fail");

    assert_eq!(error, TopicError::InvalidName(TOPIC_NAME_TOO_LONG));
}

#[test]
fn topic_definition_is_partitionless_metadata_with_schema() {
    let topic = TopicDefinition::new("orders.received", order_schema()).expect("topic definition should be valid");

    assert_eq!(topic.name().as_str(), "orders.received");
}

#[test]
fn topic_definition_carries_arrow_schema_metadata() {
    let topic = TopicDefinition::new("orders.received", order_schema()).expect("topic definition should be valid");
    let schema = topic.schema();

    assert_eq!(schema.fields().len(), 2);
    assert_eq!(schema.field(0).name(), "order_id");
    assert_eq!(schema.field(0).data_type(), &DataType::Utf8);
    assert_eq!(schema.field(1).name(), "received_at_ns");
    assert_eq!(
        schema.field(1).data_type(),
        &DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, None)
    );
}

fn resource_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("unit-tests")
        .join("resources")
        .join("broker")
        .join(name)
}

fn order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("order_id", DataType::Utf8, false),
        Field::new(
            "received_at_ns",
            DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, None),
            false,
        ),
    ]))
}
