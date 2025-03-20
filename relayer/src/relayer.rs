use std::str::FromStr;

use alloy::{
    eips::BlockId,
    primitives::{Address, FixedBytes, U256},
    providers::{DynProvider, Provider, ProviderBuilder},
    rpc::types::BlockTransactionsKind,
};
use alloy_chains::Chain;
use eyre::{Result, WrapErr};
use futures_util::StreamExt;
use types::{header::RlpHeader, proofs::AccountProof, ProverInput};

use crate::{listener::WorldIDListener, prover::Risc0Prover, publisher::ProofPublisher, Config};

#[derive(Debug, Clone)]
pub struct Relayer {
    latest_root_slot: FixedBytes<32>,
    world_listener: WorldIDListener,
    provider: DynProvider,
    world_id_addr: Address,
    prover: Risc0Prover,
    proof_publisher: ProofPublisher,
    chain: Chain,
}

/// Builder for the Relayer struct to simplify initialization
pub struct RelayerBuilder {
    config: Config,
}

impl RelayerBuilder {
    /// Create a new RelayerBuilder with the provided configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn build(self) -> Result<Relayer> {
        let latest_root_slot =
            FixedBytes::<32>::from(U256::from(self.config.world_id_latest_root_slot));
        let provider = ProviderBuilder::new()
            .on_builtin(&self.config.ethereum_rpc_url)
            .await?
            .erased();

        let world_idm = Address::from_str(&self.config.world_id_manager)
            .wrap_err("Failed to parse World Identity Manager address")?;
        let world_listener = WorldIDListener::new(provider.clone(), world_idm.clone());

        let prover = Risc0Prover {};
        let publisher = ProofPublisher::new(
            &self.config.starknet_rpc_url,
            &self.config.starknet_private_key,
            &self.config.starknet_account,
            &self.config.relayer_verifier,
            &self.config.chain,
        )?;

        Ok(Relayer {
            latest_root_slot,
            world_listener,
            provider,
            world_id_addr: world_idm,
            prover,
            proof_publisher: publisher,
            chain: self.config.chain,
        })
    }
}

impl Relayer {
    pub async fn relay(&self) -> Result<()> {
        let stream = self.world_listener.subscribe().await?;
        tokio::pin!(stream);

        while let Some((event, block_number)) = stream.next().await {
            let new_root = event.postRoot;
            tracing::info!("New root detected: {:?}", new_root);

            let prover_input = self.prepare_prover_input(block_number).await?;
            let proof = self.prover.prove(prover_input).await?;
            self.proof_publisher.publish(&proof).await?;
        }

        Ok(())
    }

    async fn prepare_prover_input(&self, block_number: u64) -> Result<ProverInput> {
        // Make the calls for proving
        let block = self
            .provider
            .get_block(BlockId::from(block_number), BlockTransactionsKind::Hashes)
            .await?
            .unwrap();

        let block_hash = block.header.hash;
        let block = block.into_consensus();

        let account_proof = self
            .provider
            .get_proof(self.world_id_addr, vec![self.latest_root_slot])
            .block_id(BlockId::from(block_number))
            .await?;

        Ok(ProverInput {
            header: RlpHeader::new(block.header),
            account_proof: AccountProof::from(account_proof),
            block_header: block_hash,
        })
    }
}
