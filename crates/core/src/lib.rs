use alloy_consensus::Header;
use header::RlpHeader;
use proofs::AccountProof;
use serde::{Deserialize, Serialize};

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
    /// Complete account proof including storage information
    ///
    /// Contains the Merkle-Patricia proof for the account's state
    /// and a proof for the specific storage slot being verified.
    pub account_proof: AccountProof,
}
