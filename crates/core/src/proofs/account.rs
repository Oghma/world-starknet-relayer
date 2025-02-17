use alloy_primitives::{keccak256, Address, Bytes, FixedBytes};
use alloy_rpc_types::EIP1186AccountProofResponse;
use alloy_trie::{proof::verify_proof, Nibbles, TrieAccount};
use serde::{Deserialize, Serialize};

use super::StorageProof;
use crate::error::{ProverError, TrieErrorContext};

/// Represents a full account proof including storage information.
///
/// Contains all necessary components to verify an account's state and associated
/// storage slot within a Merkle-Patricia trie according to Ethereum's specification.
///
/// The proof structure follows EIP-1186 format with additional trie validation capabilities.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountProof {
    /// The Ethereum address being proven
    pub address: Address,
    /// Complete account state data in trie-compatible format
    pub trie: TrieAccount,
    /// List of RLP-encoded nodes proving the account's existence in the state trie
    pub proof: Vec<Bytes>,
    /// Proof for the first storage slot within the account's storage trie
    pub storage_proof: StorageProof,
}

impl From<EIP1186AccountProofResponse> for AccountProof {
    /// Converts from the RPC response format to our internal proof representation.
    ///
    /// Handles:
    /// - Construction of TrieAccount from individual account fields
    /// - Cloning of the account proof data
    /// - Conversion of the first storage proof entry
    fn from(value: EIP1186AccountProofResponse) -> Self {
        let trie = TrieAccount {
            nonce: value.nonce,
            balance: value.balance,
            storage_root: value.storage_hash,
            code_hash: value.code_hash,
        };

        Self {
            trie,
            address: value.address,
            proof: value.account_proof.clone(),
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
    /// Implements Ethereum's state proof verification using:
    /// - keccak256(address) as the trie key
    /// - TrieAccount structure for proper RLP encoding
    /// - alloy-trie's proof verification with Merkle-Patricia trie rules
    pub fn verify_proof(&self, state_root: FixedBytes<32>) -> Result<(), ProverError> {
        let key = Nibbles::unpack(keccak256(self.address));
        let proof_refs: Vec<&Bytes> = self.proof.iter().collect();
        let account_state = alloy_rlp::encode(self.trie);

        verify_proof(state_root, key, Some(account_state), proof_refs).map_err(|e| {
            ProverError::TrieVerification {
                context: TrieErrorContext::AccountRoot,
                source: e,
            }
        })
    }
}
