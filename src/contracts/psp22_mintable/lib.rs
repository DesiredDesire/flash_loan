#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod psp22_mintable {

    // use lending_project::traits::managing::*;
    use openbrush::{
        contracts::psp22::extensions::{burnable::*, metadata::*, mintable::*},
        traits::Storage,
    };

    use ink_prelude::string::String;
    use ink_storage::traits::SpreadAllocate;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct PSP22MintableContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl PSP22 for PSP22MintableContract {}

    impl PSP22Metadata for PSP22MintableContract {}

    impl PSP22MintableContract {
        #[ink(constructor)]
        pub fn new(name: Option<String>, symbol: Option<String>, decimal: u8) -> Self {
            ink_env::debug_println!("PSP22constructor");
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // metadata
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;
            })
        }
    }

    impl PSP22Mintable for PSP22MintableContract {}
}
