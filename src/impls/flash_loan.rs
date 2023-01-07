// TODO::think should we emit events on set_as_collateral

#![allow(unused_variables)]
use crate::traits::{
    flash_loan::FlashLoanError, flash_loan::*, flash_loan_receiver::FlashLoanReceiverRef,
};
use ink_env::CallFlags;
use ink_prelude::{string::*, vec, vec::Vec};

use openbrush::traits::DefaultEnv;
use openbrush::{
    contracts::traits::psp22::PSP22Ref,
    traits::{AccountId, Balance},
};

pub trait FlashLoanEventEmit {
    /// !!! should be overriden in contract !!!
    /// emits FlashLoanEvent
    fn _emit_flash_loan_event(
        &mut self,
        receiver: AccountId,
        asset: AccountId,
        amount: Balance,
        fee: Balance,
    );
    fn _emit_flash_loan_events(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        fees: &Vec<Balance>,
    ) -> Result<(), FlashLoanError>;
}

impl<T> FlashLoanEventEmit for T {
    default fn _emit_flash_loan_event(
        &mut self,
        receiver: AccountId,
        asset: AccountId,
        amount: Balance,
        fee: Balance,
    ) {
    }
    default fn _emit_flash_loan_events(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        fees: &Vec<Balance>,
    ) -> Result<(), FlashLoanError> {
        for i in 0..assets.len() {
            self._emit_flash_loan_event(*receiver, assets[i], amounts[i], fees[i]);
        }
        Ok(())
    }
}

pub trait FlashLoanInternal {
    /// may be overriden in contract to perform any operation before sending flash laon
    fn _before_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<(), FlashLoanError>;

    /// may be overriden in contract to perform any operation after sending flash laon
    fn _after_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        fees: &Vec<Balance>,
        data: &Vec<u8>,
    ) -> Result<(), FlashLoanError>;

    /// !!! should be overriden in contract !!!
    ///  returns list of Balances (amounts) of fee that will be taken after fhlash loan.
    fn _calculate_fees(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<Vec<Balance>, FlashLoanError>;

    /// may be overriden in contract
    /// transfers assets to the receiver
    fn _send_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<(), FlashLoanError>;

    /// may be overriden in contract
    /// transfers assets back from the receiver to the contract
    fn _get_back_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        fees: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<(), FlashLoanError>;
}
impl<T: FlashLoanEventEmit> FlashLoanInternal for T {
    default fn _before_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<(), FlashLoanError> {
        Ok(())
    }

    default fn _after_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        fees: &Vec<Balance>,
        data: &Vec<u8>,
    ) -> Result<(), FlashLoanError> {
        Ok(())
    }
    default fn _calculate_fees(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<Vec<Balance>, FlashLoanError> {
        let fees: Vec<Balance> = vec![0; assets.len()];
        Ok(fees)
    }

    default fn _send_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<(), FlashLoanError> {
        for i in 0..assets.len() {
            PSP22Ref::transfer_builder(&assets[i], *receiver, amounts[i], Vec::<u8>::new())
                .call_flags(CallFlags::default().set_allow_reentry(true))
                .fire()
                .unwrap()?;
        }
        Ok(())
    }

    default fn _get_back_flash_loan(
        &mut self,
        receiver: &AccountId,
        assets: &Vec<AccountId>,
        amounts: &Vec<Balance>,
        fees: &Vec<Balance>,
        data: &mut Vec<u8>,
    ) -> Result<(), FlashLoanError> {
        for i in 0..assets.len() {
            ink_env::debug_println!("[flash] before transfer_from");
            PSP22Ref::transfer_from_builder(
                &assets[i],
                *receiver,
                Self::env().account_id(),
                amounts[i] + fees[i],
                Vec::<u8>::new(),
            )
            .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
            .fire()
            .unwrap()?;
            ink_env::debug_println!("[flash] after transfer_from");
        }
        Ok(())
    }
}

impl<T: FlashLoanEventEmit + FlashLoanInternal> FlashLoan for T {
    default fn flash_loan(
        &mut self,
        receiver: AccountId,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        mut data: Vec<u8>,
    ) -> Result<(), FlashLoanError> {
        if !(assets.len() == amounts.len()) {
            return Err(FlashLoanError::Parameters);
        }
        ink_env::debug_println!("flash_loan | before_flash_loan");
        self._before_flash_loan(&receiver, &assets, &amounts, &mut data)?;
        let fees: Vec<Balance> = self._calculate_fees(&receiver, &assets, &amounts, &mut data)?;

        ink_env::debug_println!("flash_loan | _send_flash_loan");
        self._send_flash_loan(&receiver, &assets, &amounts, &mut data)?;

        ink_env::debug_println!("flash_loan | execute_operation_builder");
        FlashLoanReceiverRef::execute_operation_builder(
            &receiver,
            assets.clone(),
            amounts.clone(),
            fees.clone(),
            data.clone(),
        )
        .call_flags(CallFlags::default().set_allow_reentry(true))
        .fire()
        .unwrap()?;
        ink_env::debug_println!("flash_loan | _get_back_flash_loan");
        self._get_back_flash_loan(&receiver, &assets, &amounts, &fees, &mut data)?;
        ink_env::debug_println!("flash_loan | _after_flash_loan");
        self._after_flash_loan(&receiver, &assets, &amounts, &fees, &mut data)?;

        Ok(())
    }
}
