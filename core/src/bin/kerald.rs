use clap::Parser;
use kerald::{AdmissionState, Broker, BrokerConfig};
use std::{num::NonZeroU16, path::PathBuf};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[derive(Debug, Parser)]
#[command(name = "kerald", version, about = "Kerald broker")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let cli = Cli::parse();
    let config = match cli.config {
        Some(path) => BrokerConfig::from_path(path)?,
        None => BrokerConfig::single_node(NonZeroU16::new(9000).expect("default inter-broker port is non-zero")),
    };

    let broker = Broker::new(config).start().await?;
    let cluster = broker.config().cluster();

    info!(
        local_node_id = %broker.local_node_id(),
        expected_brokers = cluster.expected_brokers().get(),
        quorum = cluster.quorum_size().get(),
        inter_broker_port = broker.config().inter_broker().port().get(),
        "kerald broker started"
    );

    if let AdmissionState::RejectingUntilCoordinationReady { reason } = broker.admission_state() {
        warn!(reason, "write admission disabled");
    }

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
