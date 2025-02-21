#[starknet::interface]
pub trait IWorldRelayerVerifier<TContractState> {
    fn verify_latest_root_proof(ref self: TContractState, proof: Span<felt252>) -> bool;
    fn get_verifier_address(self: @TContractState) -> starknet::ContractAddress;
}

#[starknet::contract]
mod WorldRelayerVerifier {
    use verifier::decode_journal;
    use verifier::groth16_verifier::{
        IRisc0Groth16VerifierBN254Dispatcher, IRisc0Groth16VerifierBN254DispatcherTrait,
    };
    use world_relayer_store::{IWorldRelayerStoreDispatcher, IWorldRelayerStoreDispatcherTrait};

    #[storage]
    struct Storage {
        bn254_verifier: IRisc0Groth16VerifierBN254Dispatcher,
        world_relayer_store: IWorldRelayerStoreDispatcher,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        LatestRootVerified: LatestRootVerified,
    }

    #[derive(Drop, starknet::Event)]
    struct LatestRootVerified {
        old_latest_root: u256,
        old_latest_block: u64,
        new_latest_root: u256,
        new_latest_block: u64,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        verifier_address: starknet::ContractAddress,
        world_relayer_store_address: starknet::ContractAddress,
    ) {
        self
            .bn254_verifier
            .write(IRisc0Groth16VerifierBN254Dispatcher { contract_address: verifier_address });
        self
            .world_relayer_store
            .write(IWorldRelayerStoreDispatcher { contract_address: world_relayer_store_address });
    }

    #[external(v0)]
    fn verify_latest_root_proof(ref self: ContractState, mut proof: Span<felt252>) -> bool {
        let _ = proof.pop_front();
        let journal = self
            .bn254_verifier
            .read()
            .verify_groth16_proof_bn254(proof)
            .expect('Failed to verify proof');

        let journal = decode_journal(journal);
        let world_relayer_store = self.world_relayer_store.read();

        let (old_latest_root, old_latest_block) = world_relayer_store.get_latest_root_block();
        world_relayer_store.update_latest_root_state(journal);

        self
            .emit(
                LatestRootVerified {
                    old_latest_root,
                    old_latest_block,
                    new_latest_block: journal.latest_block,
                    new_latest_root: journal.state_root,
                },
            );

        true
    }
}
