use alloy_primitives::FixedBytes;
use alloy_trie::proof::ProofVerificationError;
use thiserror::Error;

/// Represents potential verification failures during proof generation.
///
/// This error type encapsulates all possible verification errors that can occur
/// when validating Ethereum state proofs in the zkVM prover.
#[derive(Error, Debug, PartialEq)]
pub enum ProverError {
    /// Indicates a mismatch between the computed and expected block hash.
    ///
    /// This critical error occurs when the hash derived from the block header
    /// doesn't match the expected block hash. This could indicate:
    /// - Tampered header data
    /// - Incorrect block reference
    /// - RLP encoding/decoding issues
    #[error("Block hash mismatch (expected {expected}, found {found})")]
    BlockHashMismatch {
        /// The expected block hash from trusted sources
        expected: FixedBytes<32>,
        /// The computed hash from provided header data
        found: FixedBytes<32>,
    },
    /// Wraps trie verification errors with context
    #[error("{context} verification failed: {source}")]
    TrieVerification {
        context: TrieErrorContext,
        source: ProofVerificationError,
    },
}

/// Error context for trie verification failures
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum TrieErrorContext {
    /// Failure in account state trie verification
    AccountRoot,
    /// Failure in storage trie verification
    StorageRoot,
}

impl std::fmt::Display for TrieErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountRoot => write!(f, "Account state trie"),
            Self::StorageRoot => write!(f, "Storage trie"),
        }
    }
}
