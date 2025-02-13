use alloy_primitives::{keccak256, Bytes, FixedBytes, U256};
use alloy_rpc_types::EIP1186StorageProof;
use alloy_trie::{
    proof::{verify_proof, ProofVerificationError},
    Nibbles,
};
use serde::{Deserialize, Serialize};

/// Represents a storage proof for a specific storage slot in an Ethereum account.
///
/// Contains the original storage key, corresponding value, and the Merkle-Patricia proof
/// needed to verify the slot's inclusion in the storage trie.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageProof {
    /// The original storage slot key (32-byte big-endian format)
    pub key: FixedBytes<32>,
    /// The value stored in the storage slot
    pub value: U256,
    /// The Merkle-Patricia proof for this storage entry
    pub proof: Vec<Bytes>,
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
    ///
    /// * `storage_hash` - The root hash of the storage trie being verified against
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the proof is valid
    /// - `Err(ProofVerificationError)` if the proof is invalid or doesn't match the key/value
    ///
    /// # Note
    ///
    /// This hashes the storage key using keccak256 (per Ethereum storage trie conventions)
    /// and uses RLP encoding of the storage value in big-endian format for verification.
    pub fn verify_proof(&self, storage_hash: FixedBytes<32>) -> Result<(), ProofVerificationError> {
        let key = Nibbles::unpack(keccak256(self.key));
        let proof_refs: Vec<&Bytes> = self.proof.iter().collect();
        let value = alloy_rlp::encode(self.value.to_be_bytes::<32>());

        verify_proof(storage_hash, key, Some(value), proof_refs)
    }
}
