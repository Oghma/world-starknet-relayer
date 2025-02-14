use alloy_primitives::{keccak256, Address, Bytes, FixedBytes, U256};
use alloy_rlp::Encodable;
use alloy_rpc_types::EIP1186AccountProofResponse;
use alloy_trie::{
    proof::{verify_proof, ProofVerificationError},
    Nibbles,
};
use serde::{Deserialize, Serialize};

use super::StorageProof;

/// Represents a full account proof including storage information.
///
/// Contains all necessary components to verify an account's state and associated
/// storage slot within a Merkle-Patricia trie.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountProof {
    /// The Ethereum address of the account being proven
    pub address: Address,
    /// The account's current balance in wei
    pub balance: U256,
    /// Hash of the account's contract code (keccak256 of empty bytes for EOA)
    pub code_hash: FixedBytes<32>,
    /// The account's transaction count/nonce
    pub nonce: u64,
    /// Merkle-Patricia proof for the account's state
    pub proof: Vec<Bytes>,
    /// Root hash of the account's storage trie
    pub storage_hash: FixedBytes<32>,
    /// Proof for a specific storage slot within the account's storage trie
    pub storage_proof: StorageProof,
}

impl From<EIP1186AccountProofResponse> for AccountProof {
    /// Converts from the RPC response format to our internal proof representation.
    ///
    /// Handles:
    /// - Direct field mapping for address, balance, code_hash, and nonce
    /// - Cloning of the account proof data
    /// - Conversion of the first storage proof entry
    fn from(value: EIP1186AccountProofResponse) -> Self {
        Self {
            address: value.address,
            balance: value.balance,
            code_hash: value.code_hash,
            nonce: value.nonce,
            proof: value.account_proof.clone(),
            storage_hash: value.storage_hash,
            storage_proof: StorageProof::from(value.storage_proof[0].clone()),
        }
    }
}

impl AccountProof {
    /// Verifies the account proof against a given state root.
    ///
    /// # Arguments
    /// * `state_root` - The root hash of the state trie being verified against
    ///
    /// # Returns
    /// - `Ok(())` if the proof is valid
    /// - `Err(ProofVerificationError)` if the proof is invalid or doesn't match the account state
    ///
    /// # Note
    /// Uses keccak256 of the address as the trie key and RLP-encoded account state data
    /// following Ethereum's state trie conventions.
    pub fn verify_proof(&self, state_root: FixedBytes<32>) -> Result<(), ProofVerificationError> {
        let key = Nibbles::unpack(keccak256(self.address));
        let proof_refs: Vec<&Bytes> = self.proof.iter().collect();

        let mut out = Vec::new();
        self.nonce.encode(&mut out);
        self.balance.encode(&mut out);
        self.storage_hash.encode(&mut out);
        self.code_hash.encode(&mut out);
        let account_state = alloy_rlp::encode(out);

        verify_proof(state_root, key, Some(account_state), proof_refs)
    }
}
