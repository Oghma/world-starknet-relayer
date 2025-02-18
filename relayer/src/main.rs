//! World ID Relayer Service
//!
//! A zk-SNARK based relayer that monitors World ID identity changes and generates
//! storage inclusion proofs for state transitions.

mod relayer;

use alloy::primitives::{FixedBytes, U256};
use clap::Parser;
use eyre::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Parser)]
#[command(version, about, author)]
struct Config {
    /// Primary Ethereum JSON-RPC endpoint URL
    #[arg(short, long, env, default_value = "https://eth.llamarpc.com")]
    first_rpc_url: String,

    /// Secondary Ethereum JSON-RPC endpoint URL for fetching block hash
    #[arg(short, long, env, default_value = "https://eth.merkle.io")]
    second_rpc_url: String,

    /// Storage slot number for identity merkle root in WorldID contract
    #[arg(short, long, env, default_value = "302")]
    root_slot: u64,

    /// Address of the WorldIdentityManager contract
    #[arg(short = 'm', long, env = "WORLD_IDENTITY_MANAGER", required = true)]
    world_im: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    tracing::info!("Starting relayer");

    let config = Config::parse();
    tracing::debug!(?config, "Loaded configuration");

    let _storage_slot = FixedBytes::from(U256::from(config.root_slot));

    relayer::run(config).await
}
