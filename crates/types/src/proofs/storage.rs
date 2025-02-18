use alloy_primitives::{keccak256, Bytes, FixedBytes, U256};
use alloy_rpc_types::EIP1186StorageProof;
use alloy_trie::{proof::verify_proof, Nibbles};
use serde::{Deserialize, Serialize};

use crate::error::{ProverError, TrieErrorContext};

/// Represents a storage slot proof within an account's storage trie.
///
/// Contains the Merkle-Patricia proof for a specific storage slot value
/// and its associated verification logic.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageProof {
    /// The storage slot key being proven (keccak256 hash of the slot position)
    pub key: FixedBytes<32>,
    /// RLP-encoded nodes proving the slot's existence in the storage trie
    pub proof: Vec<Bytes>,
    /// The value stored in the storage slot
    pub value: U256,
}

impl From<EIP1186StorageProof> for StorageProof {
    /// Converts from the RPC format storage proof to our internal representation.
    ///
    /// This handles the conversion of the storage key to a 32-byte representation
    /// while maintaining the same proof structure and value.
    fn from(value: EIP1186StorageProof) -> Self {
        Self {
            key: value.key.as_b256(),
            value: value.value,
            proof: value.proof,
        }
    }
}

impl StorageProof {
    /// Verifies the storage proof against a given storage root.
    ///
    /// # Arguments
    /// * `storage_root` - The root hash of the storage trie being verified against
    ///
    /// # Returns
    /// - `Ok(())` if the proof is valid
    /// - `Err(ProverError)` if the proof is invalid or doesn't match the storage value
    pub fn verify_proof(&self, storage_root: FixedBytes<32>) -> Result<(), ProverError> {
        let key = Nibbles::unpack(keccak256(self.key));
        let proof_refs: Vec<&Bytes> = self.proof.iter().collect();
        let value = alloy_rlp::encode(self.value.to_be_bytes::<32>());

        verify_proof(storage_root, key, Some(value), proof_refs).map_err(|e| {
            ProverError::TrieVerification {
                context: TrieErrorContext::StorageRoot,
                source: e,
            }
        })
    }
}
