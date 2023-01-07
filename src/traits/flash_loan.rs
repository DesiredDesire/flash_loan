use ink_prelude::{string::String, vec::Vec};
use openbrush::traits::{AccountId, Balance};

use openbrush::contracts::psp22::PSP22Error;

use super::flash_loan_receiver::FlashLoanReceiverError;

#[openbrush::wrapper]
pub type FlashLoanRef = dyn FlashLoan;

#[openbrush::trait_definition]
pub trait FlashLoan {
    #[ink(message)]
    /// is used to perform a flash loan
    ///
    ///  * `receiver` - AccountId (aka address) of a contract that will receive the flash loan.
    ///  * `assets` - list of AccountIds (aka adrress) of PSP22 Tokens that should be lent.
    ///  * `amount` - list of Balances (amounts) to be lent. Order in list coresponds to order of list of the assets.
    ///  * `data` - list of bytes that can be used for any purpose
    fn flash_loan(
        &mut self,
        reciever: AccountId,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        data: Vec<u8>,
    ) -> Result<(), FlashLoanError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FlashLoanError {
    Custom(String),
    PSP22Error(PSP22Error),
    FlashLoanReceiverError(FlashLoanReceiverError),
    /// returned if assets.len() != amounts.len()
    Parameters,
}

impl From<PSP22Error> for FlashLoanError {
    fn from(error: PSP22Error) -> Self {
        FlashLoanError::PSP22Error(error)
    }
}

impl From<FlashLoanReceiverError> for FlashLoanError {
    fn from(error: FlashLoanReceiverError) -> Self {
        FlashLoanError::FlashLoanReceiverError(error)
    }
}
