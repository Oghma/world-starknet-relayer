use std::str::FromStr;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::{Address, FixedBytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::BlockTransactionsKind,
    sol,
};
use eyre::Result;
use futures_util::StreamExt;
use methods::STORAGE_INCLUSION_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use types::{header::RlpHeader, proofs::AccountProof, ProverInput};

use crate::Config;

sol!(
    #[sol(rpc)]
    WorldIdentityManager,
    "abi/WorldIdentityManager.json"
);

pub async fn run(config: Config) -> Result<()> {
    let storage_slot = FixedBytes::from(U256::from(config.root_slot));

    let main_provider = ProviderBuilder::new()
        .on_builtin(&config.first_rpc_url)
        .await?;
    let block_provider = ProviderBuilder::new()
        .on_builtin(&config.second_rpc_url)
        .await?;
    let world_id_addr = Address::from_str(&config.world_im)?;
    let world_id = WorldIdentityManager::new(world_id_addr, main_provider.clone());
    tracing::info!("World id manager address: {world_id_addr}");

    let block_number = main_provider.get_block_number().await?;
    tracing::info!("Starting relay from block {block_number}");

    let latest_root = world_id.latestRoot().call().await?._0;
    tracing::info!("Current worldId latest root: {}", latest_root);

    let filter = world_id.TreeChanged_filter().watch().await?;
    let mut stream = filter.into_stream();

    while let Some(log) = stream.next().await {
        let (event, log) = log.unwrap();
        tracing::info!("New TreeChanged event");

        if event.preRoot == event.postRoot {
            tracing::info!("latesRoot has not changed, ignoring...");
            continue;
        }

        let new_root = event.postRoot;
        tracing::info!("New root detected: {:?}", new_root);

        // Make the calls for proving
        let block_number = log.block_number.unwrap();
        let block = main_provider
            .get_block(BlockId::from(block_number), BlockTransactionsKind::Hashes)
            .await?
            .unwrap()
            .into_consensus();

        let account_proof = main_provider
            .get_proof(world_id_addr, vec![storage_slot])
            .block_id(BlockId::from(block_number))
            .await?;

        // Fetch block hash from another node
        let block_hash = block_provider
            .get_block_by_number(
                BlockNumberOrTag::Number(block_number),
                BlockTransactionsKind::Hashes,
            )
            .await?
            .unwrap()
            .header
            .hash;

        let input = ProverInput {
            header: RlpHeader::new(block.header),
            account_proof: AccountProof::from(account_proof),
            block_header: block_hash,
        };

        run_prover(&input)?
    }

    Ok(())
}

fn run_prover(input: &ProverInput) -> Result<()> {
    let env = ExecutorEnv::builder()
        .write(input)
        .expect("failed to write the prover input")
        .build()
        .unwrap();

    tracing::info!("Starting proof generation");
    let prover = default_prover();
    let _receipt = prover
        .prove_with_opts(env, STORAGE_INCLUSION_ELF, &ProverOpts::groth16())
        .unwrap()
        .receipt;

    Ok(())
}
