use snforge_std::{
    ContractClassTrait, DeclareResultTrait, declare,
};

use super::fixtures::{calldata_default, test_journal};
use verifier::{
    decode_journal,
    groth16_verifier::{
        IRisc0Groth16VerifierBN254Dispatcher, IRisc0Groth16VerifierBN254DispatcherTrait,
    },
    world_relayer_verifier::{IWorldRelayerVerifierDispatcher, IWorldRelayerVerifierDispatcherTrait},
};
use world_relayer_store::{IWorldRelayerStoreDispatcher, IWorldRelayerStoreDispatcherTrait};

fn deploy() -> (IRisc0Groth16VerifierBN254Dispatcher, IWorldRelayerVerifierDispatcher) {
    let ecip_class = declare("UniversalECIP").unwrap().contract_class();
    let contract = declare("Risc0Groth16VerifierBN254").unwrap().contract_class();
    // Alternatively we could use `deploy_syscall` here
    let (groth16_verifier_address, _) = contract
        .deploy(@array![(*ecip_class.class_hash).into()])
        .unwrap();

    let (world_relayer_store_address, _) = declare("WorldRelayerStore")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    let (verifier_address, _) = declare("WorldRelayerVerifier")
        .unwrap()
        .contract_class()
        .deploy(@array![groth16_verifier_address.into(), world_relayer_store_address.into()])
        .unwrap();

    // Create a Dispatcher object that will allow interacting with the deployed contract
    let store_dispatcher = IWorldRelayerStoreDispatcher {
        contract_address: world_relayer_store_address,
    };
    store_dispatcher.initialize(verifier_address);
    (
        IRisc0Groth16VerifierBN254Dispatcher { contract_address: groth16_verifier_address },
        IWorldRelayerVerifierDispatcher { contract_address: verifier_address },
    )
}

#[test]
fn test_verify_groth16_proof_bn254() {
    let (groth16_verifier_dispatcher, _) = deploy();
    let mut calldata = calldata_default();
    let _ = calldata.pop_front();
    let journal = decode_journal(
        groth16_verifier_dispatcher.verify_groth16_proof_bn254(calldata).unwrap(),
    );
    assert_eq!(journal, test_journal());
}

#[test]
fn test_get_verifier_address() {
    let (groth16_verifier_dispatcher, verifier) = deploy();

    assert_eq!(verifier.get_verifier_address(), groth16_verifier_dispatcher.contract_address);
}


#[test]
fn test_get_world_relayer_store_address() {
    let (_, verifier) = deploy();
    // We need to get the store address from deployment
    let store_address = verifier.get_world_relayer_store_address();
    assert!(store_address != starknet::contract_address_const::<0>());
}
