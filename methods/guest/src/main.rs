use risc0_zkvm::guest::env;

use types::{error::ProverError, ProverInput, ProverOutput};

/// ZKVM guest program for verifying Ethereum state proofs.
///
/// The verification process follows these steps:
/// 1. Validates block header consistency
/// 2. Verifies account proof against state root
/// 3. Verifies storage proof against account's storage root
/// 4. Commits proven storage value and block hash to journal
fn main() {
    let input: ProverInput = env::read();

    // Verify block header matches the expected state root
    let block_header = input.header.hash_slow();
    if input.block_header != block_header {
        panic!(
            "{}",
            ProverError::BlockHashMismatch {
                expected: input.block_header,
                found: block_header
            }
        );
    }

    // Verify account existence in the state trie
    let account_proof = &input.account_proof;
    account_proof.verify_proof(input.header.state_root).unwrap();

    // Verify storage slot value in the account's storage trie
    let storage_proof = &account_proof.storage_proof;
    // Uses storage root from the verified account trie
    storage_proof
        .verify_proof(account_proof.trie.storage_root)
        .unwrap();

    // All clear, the storage proof value is correct. Add WorldID latestRoot to
    // the journal
    let output = ProverOutput {
        block_number: input.header.number,
        state_root: storage_proof.value,
    };
    env::commit(&output);
}
