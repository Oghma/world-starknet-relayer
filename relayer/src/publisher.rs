use std::str::FromStr;

use alloy_chains::NamedChain;
use eyre::Result;
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

use crate::prover::Groth16;

#[derive(Debug, Clone)]
pub struct ProofPublisher {
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    relayer_verifier: Felt,
}

impl ProofPublisher {
    pub fn new(
        rpc_url: &str,
        private_key: &str,
        account_address: &str,
        world_verifier: &str,
        chain: &alloy_chains::Chain,
    ) -> Result<Self> {
        let provider = JsonRpcClient::new(HttpTransport::new(Url::from_str(rpc_url)?));
        let signer =
            LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(private_key)?));

        let account_address = Felt::from_hex(account_address)?;
        let relayer_verifier = Felt::from_hex(world_verifier)?;
        let chain = match chain.named().unwrap() {
            NamedChain::Mainnet => chain_id::MAINNET,
            NamedChain::Sepolia => chain_id::SEPOLIA,
            _ => return Err(eyre::eyre!("Unsupported chain for Starknet")),
        };

        let account = SingleOwnerAccount::new(
            provider,
            signer,
            account_address,
            chain,
            ExecutionEncoding::New,
        );

        Ok(Self {
            account,
            relayer_verifier,
        })
    }

    pub async fn publish(&self, proof: &Groth16) -> Result<()> {
        let selector = get_selector_from_name("verify_latest_root_proof").unwrap();
        let call = Call {
            to: self.relayer_verifier,
            selector,
            calldata: proof.calldata.clone(),
        };

        let txn = self.account.execute_v3(vec![call]).send().await?;
        tracing::info!("Update latest root transaction {}", txn.transaction_hash);

        Ok(())
    }
}
