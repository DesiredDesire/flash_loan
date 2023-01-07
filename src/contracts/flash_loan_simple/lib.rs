#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod flash_loan_contract {
    use flash_loan::impls::flash_loan::*;
    use flash_loan::traits::flash_loan::*;
    use ink_lang::codegen::{EmitEvent, Env};
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;

    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct FlashLoanSimple {}

    #[ink(event)]
    pub struct FlashLoanEvent {
        #[ink(topic)]
        receiver: AccountId,
        #[ink(topic)]
        asset: AccountId,
        amount: Balance,
        fee: Balance,
    }

    impl FlashLoanEventEmit for FlashLoanSimple {
        fn _emit_flash_loan_event(
            &mut self,
            receiver: AccountId,
            asset: AccountId,
            amount: Balance,
            fee: Balance,
        ) {
            self.env().emit_event(FlashLoanEvent {
                receiver,
                asset,
                amount,
                fee,
            });
        }
    }
    impl FlashLoan for FlashLoanSimple {}

    impl FlashLoanSimple {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {})
        }
    }
}
