#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod flash_loan_receiver_mock {
    use flash_loan::traits::flash_loan_receiver::{FlashLoanReceiverError, *};
    use ink_lang::codegen::{EmitEvent, Env};
    use ink_prelude::{format, vec::Vec};
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::traits::psp22::{extensions::mintable::PSP22MintableRef, *},
        traits::Storage,
    };

    #[ink(event)]
    pub struct ExecutedWithSuccess {
        #[ink(topic)]
        assets: Vec<AccountId>,
        #[ink(topic)]
        amounts: Vec<u128>,
        #[ink(topic)]
        fees: Vec<u128>,
    }
    #[ink(event)]
    pub struct ExecutedWithFail {
        #[ink(topic)]
        assets: Vec<AccountId>,
        #[ink(topic)]
        amounts: Vec<u128>,
        #[ink(topic)]
        fees: Vec<u128>,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct FlashLoanReceiverMock {
        fail_execute_operation: bool,
        simulate_balance_to_cover_fee: bool,
        custom_amount_to_approve: Option<Balance>,
    }
    impl FlashLoanReceiver for FlashLoanReceiverMock {
        #[ink(message)]
        #[allow(unused_variables)]
        fn execute_operation(
            &mut self,
            assets: Vec<AccountId>,
            amounts: Vec<u128>,
            fees: Vec<u128>,
            receiver_params: Vec<u8>,
        ) -> Result<(), FlashLoanReceiverError> {
            if self.fail_execute_operation {
                self.env().emit_event(ExecutedWithFail {
                    assets,
                    amounts,
                    fees,
                });
                return Err(FlashLoanReceiverError::ExecuteOperationFailed);
            }
            for i in 0..assets.len() {
                let balance = PSP22Ref::balance_of(&assets[i], self.env().account_id());
                if amounts[i] > balance {
                    return Err(FlashLoanReceiverError::Custom(format!(
                        "Insufficient balance for the contract for asset {:X?}",
                        assets[i],
                    )));
                }

                if self.simulate_balance_to_cover_fee {
                    if PSP22MintableRef::mint(&assets[i], self.env().account_id(), fees[i]).is_err()
                    {
                        return Err(FlashLoanReceiverError::Custom(format!(
                            "Asset {:X?} is not mintable",
                            assets[i]
                        )));
                    }
                }

                let amount_to_return = self
                    .custom_amount_to_approve
                    .unwrap_or(amounts[i] + fees[i]);
                if PSP22Ref::approve(&assets[i], self.env().caller(), amount_to_return).is_err() {
                    return Err(FlashLoanReceiverError::Custom(format!("Can't approve")));
                }
            }

            self.env().emit_event(ExecutedWithSuccess {
                assets,
                amounts,
                fees,
            });
            Ok(())
        }
    }

    impl FlashLoanReceiverMock {
        #[ink(constructor)]
        // pub fn new(lending_pool: AccountId) -> Self {
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // instance.lending_pool = lending_pool;
                instance.custom_amount_to_approve = None;
                instance.fail_execute_operation = false;
                instance.simulate_balance_to_cover_fee = true;
            })
        }

        #[ink(message)]
        pub fn set_fail_execute_operation(&mut self, should_fail_execute_operation: bool) {
            self.fail_execute_operation = should_fail_execute_operation;
        }

        #[ink(message)]
        pub fn set_custom_amount_to_approve(&mut self, custom_amount_to_approve: u128) {
            self.custom_amount_to_approve = Some(custom_amount_to_approve);
        }

        #[ink(message)]
        pub fn set_simulate_balance_to_cover_fee(&mut self, simulate_balance_to_cover_fee: bool) {
            self.simulate_balance_to_cover_fee = simulate_balance_to_cover_fee;
        }
    }
}
