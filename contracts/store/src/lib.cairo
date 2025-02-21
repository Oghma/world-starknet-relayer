#[starknet::interface]
pub trait IWorldRelayerStore<TContractState> {
    fn initialize(ref self: TContractState, verifier_address: starknet::ContractAddress);
    fn update_latest_root_state(ref self: TContractState, journal: verifier::Journal);
    fn get_latest_root(self: @TContractState) -> u256;
    fn get_latest_block(self: @TContractState) -> u64;
    fn get_latest_root_block(self: @TContractState) -> (u256, u64);
}

#[starknet::contract]
mod WorldRelayerStore {
    use core::starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        initialized: bool,
        verifier_address: starknet::ContractAddress,
        latest_root: u256,
        latest_block: u64,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        LatestRootUpdated: LatestRootUpdated,
    }

    #[derive(Drop, starknet::Event)]
    struct LatestRootUpdated {
        old_latest_root: u256,
        old_latest_block: u64,
        new_latest_root: u256,
        new_latest_block: u64,
    }

    #[abi(embed_v0)]
    impl WorldRelayerStore of super::IWorldRelayerStore<ContractState> {
        fn initialize(ref self: ContractState, verifier_address: starknet::ContractAddress) {
            assert!(!self.initialized.read(), "Contract already initialized");
            self.initialized.write(true);
            self.verifier_address.write(verifier_address);
        }

        fn update_latest_root_state(ref self: ContractState, journal: verifier::Journal) {
            assert!(
                starknet::get_caller_address() == self.verifier_address.read(),
                "Only the World relayer verifier can update latest_root",
            );

            let old_latest_block = self.latest_block.read();
            assert!(
                journal.latest_block >= old_latest_block,
                "The new block must be greater than or equal to {}",
                old_latest_block,
            );
            let old_latest_root = self.latest_root.read();

            self.latest_root.write(journal.state_root);
            self.latest_block.write(journal.latest_block);

            self
                .emit(
                    LatestRootUpdated {
                        old_latest_root,
                        old_latest_block,
                        new_latest_root: journal.state_root,
                        new_latest_block: journal.latest_block,
                    },
                );
        }

        fn get_latest_root(self: @ContractState) -> u256 {
            self.latest_root.read()
        }

        fn get_latest_block(self: @ContractState) -> u64 {
            self.latest_block.read()
        }

        fn get_latest_root_block(self: @ContractState) -> (u256, u64) {
            (self.latest_root.read(), self.latest_block.read())
        }
    }
}
