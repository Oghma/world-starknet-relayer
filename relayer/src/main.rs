//! World ID Relayer Service
//!
//! A zk-SNARK based relayer that monitors World ID identity changes and generates
//! storage inclusion proofs for state transitions.

mod relayer;

use clap::Parser;
use eyre::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Parser)]
#[command(version, about, author)]
struct Config {
    /// Primary Ethereum JSON-RPC endpoint URL
    #[arg(short, long, env, default_value = "https://eth.llamarpc.com")]
    first_rpc_url: String,

    /// Starknet JSON-RPC endpoint URL
    #[arg(
        short = 'b',
        long,
        env = "STARKNET_RPC",
        default_value = "https://starknet-mainnet.public.blastapi.io"
    )]
    starknet_rpc_url: String,

    /// Private key for transaction signing
    #[arg(long, env = "PRIVATE_KEY", required = true)]
    private_key: String,

    /// Starknet account address
    #[arg(long, env = "STARKNET_ACCOUNT", required = true)]
    account_address: String,

    /// Address of the WorldRelayerVerifier contract
    #[arg(
        short = 'v',
        long,
        env = "WORLD_RELAYER_VERIFIER_ADDRESS",
        required = true
    )]
    world_verifier: String,

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

    relayer::run(config).await
}
