use eyre::Result;
use garaga_rs::{
    calldata::full_proof_with_hints::groth16::{
        get_groth16_calldata_felt, risc0_utils::get_risc0_vk, Groth16Proof,
    },
    definitions::CurveID,
};
use methods::STORAGE_INCLUSION_ELF;
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::{compute_image_id, default_prover, ExecutorEnv, ProverOpts};
use starknet::core::types::Felt;
use tokio::task;
use types::ProverInput;

#[derive(Debug, Clone)]
pub struct Groth16 {
    pub calldata: Vec<Felt>,
}

#[derive(Debug, Clone)]
pub struct Risc0Prover {}

impl Risc0Prover {
    pub async fn prove(&self, input: ProverInput) -> Result<Groth16> {
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
            let calldata =
                get_groth16_calldata_felt(&proof, &get_risc0_vk(), CurveID::BN254).unwrap();

            Ok(Groth16 { calldata })
        })
        .await?
    }
}
