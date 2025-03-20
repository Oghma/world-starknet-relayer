use std::time::Duration;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::Address,
    providers::{DynProvider, Provider},
    sol,
};
use eyre::Result;
use futures_util::Stream;
use tokio::time::sleep;

sol!(
    #[sol(rpc)]
    WorldIdentityManager,
    "abi/WorldIdentityManager.json"
);

#[derive(Debug, Clone)]
pub struct WorldIDListener {
    provider: DynProvider,
    world_idm: Address,
}

impl WorldIDListener {
    pub fn new(provider: DynProvider, world_idm: Address) -> Self {
        Self {
            provider,
            world_idm,
        }
    }

    /// Create a stream that periodically polls for new finalized events
    pub async fn subscribe(
        &self,
    ) -> Result<impl Stream<Item = (WorldIdentityManager::TreeChanged, u64)>> {
        // Get initial finalized block
        let initial_finalized = get_finalized_block_number(&self.provider).await?;
        tracing::info!("Starting relay from finalized block {initial_finalized}");

        // Create contract instance once, outside the closure
        let world_contract = WorldIdentityManager::new(self.world_idm, self.provider.clone());

        // Create a manual polling stream
        let stream = futures_util::stream::unfold(
            (initial_finalized, self.provider.clone(), world_contract),
            move |(mut last_block, provider, world_contract)| async move {
                loop {
                    // Sleep to avoid excessive polling
                    sleep(Duration::from_secs(12)).await;

                    // Get current finalized block
                    let latest_finalized = match get_finalized_block_number(&provider).await {
                        Ok(num) => num,
                        Err(e) => {
                            tracing::error!("Failed to get finalized block: {}", e);
                            continue;
                        }
                    };

                    // If we have new finalized blocks
                    if latest_finalized > last_block {
                        tracing::info!(
                            "Checking for events from blocks {} to {}",
                            last_block + 1,
                            latest_finalized
                        );

                        // Query for new events in the finalized range
                        let new_events = match world_contract
                            .TreeChanged_filter()
                            .from_block(BlockNumberOrTag::Number(last_block + 1))
                            .to_block(BlockNumberOrTag::Number(latest_finalized))
                            .query()
                            .await
                        {
                            Ok(events) => events,
                            Err(e) => {
                                tracing::error!("Failed to query events: {}", e);
                                continue;
                            }
                        };

                        // Update state
                        last_block = latest_finalized;

                        // Process each new event
                        for (event, log) in new_events {
                            // Event results from query() are already typed correctly
                            tracing::info!("New TreeChanged event");
                            let block_number = log.block_number.unwrap();

                            // Skip events where root hasn't changed
                            if event.preRoot == event.postRoot {
                                tracing::info!("latesRoot has not changed, ignoring...");
                                continue;
                            }

                            return Some((
                                (event, block_number),
                                (last_block, provider, world_contract),
                            ));
                        }
                    }
                }
            },
        );

        Ok(stream)
    }
}

/// Helper to get the latest finalized block number
async fn get_finalized_block_number(provider: &DynProvider) -> Result<u64> {
    let finalized_block = provider
        .get_block(
            BlockId::from(BlockNumberOrTag::Finalized),
            Default::default(),
        )
        .await?
        .ok_or_else(|| eyre::eyre!("Failed to get finalized block"))?;

    Ok(finalized_block.header.number)
}
