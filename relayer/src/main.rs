//! World ID Relayer Service
//!
//! A zk-SNARK based relayer that monitors World ID identity changes and generates
//! storage inclusion proofs for state transitions.

mod listener;
mod prover;
mod publisher;
mod relayer;

use clap::Parser;
use eyre::Result;
use tracing_subscriber::{fmt, EnvFilter};
use alloy_chains::Chain;

use relayer::RelayerBuilder;

#[derive(Debug, Parser)]
#[command(version, about, author)]
struct Config {
    /// Chain to operate on (e.g., "sepolia", "mainnet")
    #[arg(short = 'c', long, env = "CHAIN", default_value = "sepolia")]
    chain: Chain,

    /// Ethereum JSON-RPC endpoint URL
    #[arg(
        short = 'e',
        long,
        env = "ETH_RPC_URL",
        default_value = "https://eth.llamarpc.com"
    )]
    ethereum_rpc_url: String,

    /// Starknet JSON-RPC endpoint URL
    #[arg(
        short = 's',
        long,
        env = "STARKNET_RPC_URL",
        default_value = "https://starknet-mainnet.public.blastapi.io"
    )]
    starknet_rpc_url: String,

    /// Starknet private key for transaction signing
    #[arg(long, env = "STARKNET_PRIVATE_KEY", required = true)]
    starknet_private_key: String,

    /// Starknet account address
    #[arg(long, env = "STARKNET_ACCOUNT_ADDRESS", required = true)]
    starknet_account: String,

    /// Address of the WorldRelayerVerifier contract
    #[arg(short = 'v', long, env = "RELAYER_VERIFIER", required = true)]
    relayer_verifier: String,

    /// Address of the WorldIdentityManager contract
    #[arg(short = 'm', long, env = "WORLD_IDENTITY_MANAGER", required = true)]
    world_id_manager: String,

    /// Storage slot of the latestRoot variable in WorldIdentityManager contract
    #[arg(short = 'r', long, env = "WORLD_ID_LATEST_ROOT_SLOT", default_value = "302")]
    world_id_latest_root_slot: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    tracing::info!("Starting relayer");

    let config = Config::parse();
    tracing::debug!(?config, "Loaded configuration");

    let relayer = RelayerBuilder::new(config).build().await?;
    relayer.relay().await
}
