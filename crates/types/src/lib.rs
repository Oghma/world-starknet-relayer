use alloy_consensus::Header;
use alloy_primitives::{FixedBytes, U256};
use header::RlpHeader;
use proofs::AccountProof;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod header;
pub mod proofs;

/// The input structure for generating storage inclusion proofs in the zkVM.
///
/// Contains all required data to prove the existence of a storage slot
/// within a specific Ethereum block.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProverInput {
    /// Block header wrapped with RLP encoding/decoding support
    ///
    /// The RlpHeader ensures proper serialization format and provides
    /// efficient hash computation through cached RLP encoding.
    pub header: RlpHeader<Header>,
    /// The block hash (keccak256 of RLP-encoded header) where the proof is anchored
    ///
    /// This serves as the root of trust for verifying the account and storage proofs.
    /// Must match the hash computed from the RlpHeader to ensure consistency.
    pub block_header: FixedBytes<32>,
    /// Complete account proof including storage information
    ///
    /// Contains the Merkle-Patricia proof for the account's state
    /// and a proof for the specific storage slot being verified.
    pub account_proof: AccountProof,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProverOutput {
    pub block_number: u64,
    pub state_root: U256,
}
