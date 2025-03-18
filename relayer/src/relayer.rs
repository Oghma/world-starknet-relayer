use std::{fs, str::FromStr};

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::{Address, FixedBytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::BlockTransactionsKind,
    sol,
};
use eyre::Result;
use futures_util::StreamExt;
use garaga_rs::{
    calldata::full_proof_with_hints::groth16::{
        get_groth16_calldata_felt, risc0_utils::get_risc0_vk, Groth16Proof,
    },
    definitions::CurveID,
};
use methods::STORAGE_INCLUSION_ELF;
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::{compute_image_id, default_prover, ExecutorEnv, ProverOpts, Receipt};
use serde::{Deserialize, Serialize};
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    core::{
        chain_id,
        types::{Call, Felt},
        utils::get_selector_from_name,
    },
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Url},
    signers::{LocalWallet, SigningKey},
};
use tokio::task;
use types::{header::RlpHeader, proofs::AccountProof, ProverInput};

use crate::Config;

sol!(
    #[sol(rpc)]
    WorldIdentityManager,
    "abi/WorldIdentityManager.json"
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16 {
    receipt: Receipt,
    calldata: Vec<Felt>,
}

pub async fn run(config: Config) -> Result<()> {
    let storage_slot = FixedBytes::from(U256::from(config.root_slot));

    let starknet_provider =
        JsonRpcClient::new(HttpTransport::new(Url::from_str(&config.starknet_rpc_url)?));
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        &config.private_key,
    )?));
    tracing::info!("{:?}", config.account_address);
    let account_address = Felt::from_hex(&config.account_address)?;
    let verifier_contract = Felt::from_hex(&config.world_verifier)?;
    let starknet_account = SingleOwnerAccount::new(
        starknet_provider,
        signer,
        account_address,
        chain_id::SEPOLIA,
        ExecutionEncoding::New,
    );

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

    // let filter = world_id
    //     .TreeChanged_filter()
    //     .from_block(BlockNumberOrTag::Number(21905010))
    //     .subscribe()
    //     .await?;
    //let mut stream = filter.into_stream();
    let logs = world_id
        .TreeChanged_filter()
        .from_block(BlockNumberOrTag::Number(21905010))
        .query()
        .await?;
    //let logs = main_provider.get_logs(&filter).await?;

    for log in logs {
        //while let Some(log) = stream.next().await {
        let (event, log) = log; //.unwrap();
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

        // let proof = run_prover(input).await?;

        // let proof_serialized = serde_json::to_string(&proof)?;
        // fs::write("proof.json", proof_serialized)?;

        let proof = serde_json::from_str(&fs::read_to_string("proof.json")?)?;
        publish_proof(&starknet_account, &proof, verifier_contract).await?;
        panic!("");
    }

    Ok(())
}

async fn run_prover(input: ProverInput) -> Result<Groth16> {
    tracing::info!("Starting proof generation");
    task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&input)
            .expect("failed to write the prover input")
            .build()
            .unwrap();

        let prover = default_prover();
        let receipt = prover
            .prove_with_opts(env, STORAGE_INCLUSION_ELF, &ProverOpts::groth16())
            .unwrap()
            .receipt;

        tracing::info!("Proof generated");
        let seal = encode_seal(&receipt).unwrap();
        let image_id = compute_image_id(STORAGE_INCLUSION_ELF).unwrap();
        let journal = receipt.journal.bytes.clone();

        let proof = Groth16Proof::from_risc0(seal, image_id.as_bytes().to_vec(), journal);
        let calldata = get_groth16_calldata_felt(&proof, &get_risc0_vk(), CurveID::BN254).unwrap();
        Ok(Groth16 { receipt, calldata })
    })
    .await?
}

async fn publish_proof(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    proof: &Groth16,
    verifier_contract: Felt,
) -> Result<()> {
    let selector = get_selector_from_name("verify_latest_root_proof").unwrap();
    let call = Call {
        to: verifier_contract,
        selector,
        calldata: proof.calldata.clone(),
    };

    let txn = account.execute_v3(vec![call]).send().await;
    tracing::info!("{:#?}", txn);
    let txn = txn?;
    tracing::info!("Update latest root transaction:  {}", txn.transaction_hash);

    Ok(())
}
