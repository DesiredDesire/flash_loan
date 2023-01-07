#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use openbrush::contracts::{ownable::OwnableError, psp22::PSP22Error};

#[openbrush::contract]
pub mod flash_loan_contract {
    use flash_loan::impls::flash_loan::*;
    use flash_loan::traits::flash_loan::{FlashLoanError, *};
    use ink_lang::codegen::{EmitEvent, Env};
    use ink_prelude::{string::*, vec, vec::Vec};
    use ink_storage::traits::SpreadAllocate;
    use openbrush::contracts::ownable::*;
    use openbrush::contracts::psp22::{PSP22Error, *};
    use openbrush::modifiers;
    use openbrush::storage::Mapping;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct FlashLoanContract {
        #[storage_field]
        ownable: ownable::Data,

        fees_e6: Mapping<AccountId, u128>,
        allowed: Mapping<AccountId, bool>,
        free: Mapping<AccountId, bool>,
        earned: Mapping<AccountId, Balance>,
    }

    #[ink(event)]
    pub struct FlashLoanEvent {
        #[ink(topic)]
        receiver: AccountId,
        #[ink(topic)]
        asset: AccountId,
        amount: Balance,
        fee: Balance,
    }

    impl Ownable for FlashLoanContract {}

    impl FlashLoanEventEmit for FlashLoanContract {
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

    impl FlashLoanInternal for FlashLoanContract {
        fn _before_flash_loan(
            &mut self,
            receiver: &AccountId,
            assets: &Vec<AccountId>,
            amounts: &Vec<Balance>,
            data: &mut Vec<u8>,
        ) -> Result<(), FlashLoanError> {
            if !self.allowed.get(&self.env().caller()).unwrap_or_default() {
                return Err(FlashLoanError::Custom(String::from("NotAllowed")));
            }
            Ok(())
        }

        fn _calculate_fees(
            &mut self,
            receiver: &AccountId,
            assets: &Vec<AccountId>,
            amounts: &Vec<Balance>,
            data: &mut Vec<u8>,
        ) -> Result<Vec<Balance>, FlashLoanError> {
            let mut fees: Vec<Balance> = vec![0; assets.len()];
            if self.free.get(receiver).unwrap_or_default() {
                return Ok(fees);
            }
            for i in 0..assets.len() {
                let fee =
                    amounts[i] * self.fees_e6.get(&assets[i]).unwrap_or_default() / 1_000_000_u128;
                fees[i] = fee;
            }
            Ok(fees)
        }

        fn _after_flash_loan(
            &mut self,
            receiver: &AccountId,
            assets: &Vec<AccountId>,
            amounts: &Vec<Balance>,
            fees: &Vec<Balance>,
            data: &Vec<u8>,
        ) -> Result<(), FlashLoanError> {
            for i in 0..assets.len() {
                let earned = self.earned.get(&assets[i]).unwrap_or_default();
                self.earned.insert(&assets[i], &(earned + fees[i]));
            }
            Ok(())
        }
    }
    impl FlashLoan for FlashLoanContract {}

    impl FlashLoanContract {
        #[ink(constructor)]
        pub fn new(
            fees_e6: Vec<(AccountId, u128)>,
            allowed: Vec<AccountId>,
            free: Vec<AccountId>,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_owner(caller);

                for asset_and_fee in fees_e6 {
                    instance.fees_e6.insert(&asset_and_fee.0, &asset_and_fee.1);
                }

                for account in allowed {
                    instance.allowed.insert(&account, &true);
                }

                for account in free {
                    instance.allowed.insert(&account, &true);
                }
            })
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn withdraw(
            &mut self,
            token: AccountId,
            to: AccountId,
            amount: Option<Balance>,
        ) -> Result<(), FlashLoanContractError> {
            let balance = PSP22Ref::balance_of(&token, self.env().caller());
            if amount.is_some() {
                PSP22Ref::transfer(&token, to, amount.unwrap(), vec![])?;
            } else {
                PSP22Ref::transfer(&token, to, balance, vec![])?;
            }
            Ok(())
        }
    }
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FlashLoanContractError {
        Custom(String),
        PSP22Error(PSP22Error),
        OwnableError(OwnableError),
    }

    impl From<PSP22Error> for FlashLoanContractError {
        fn from(error: PSP22Error) -> Self {
            FlashLoanContractError::PSP22Error(error)
        }
    }

    impl From<OwnableError> for FlashLoanContractError {
        fn from(error: OwnableError) -> Self {
            FlashLoanContractError::OwnableError(error)
        }
    }
}
