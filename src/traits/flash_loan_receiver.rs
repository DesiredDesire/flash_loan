use ink_prelude::{string::String, vec::Vec};
use openbrush::traits::AccountId;

#[openbrush::wrapper]
pub type FlashLoanReceiverRef = dyn FlashLoanReceiver;

#[openbrush::trait_definition]
pub trait FlashLoanReceiver {
    #[ink(message)]
    /// is called during flash_loan message to perform operation during flash_loan
    ///
    ///  * `assets` - list of AccountIds (aka adrress) of PSP22 Tokens that were lend.
    ///  * `amounts` - list of Balances (amounts) that were lend and should be available to be transfer back to flash_loan contract. Order in list coresponds to order of list of the assets.
    ///  * `fees` - list of Balances (amounts) that must be additionaly paid back to flash loan contract. Order in list coresponds to order of list of the assets.
    ///  * `data` - list of bytes that can be used for any purpose
    fn execute_operation(
        &mut self,
        assets: Vec<AccountId>,
        amounts: Vec<u128>,
        fees: Vec<u128>,
        data: Vec<u8>,
    ) -> Result<(), FlashLoanReceiverError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FlashLoanReceiverError {
    Custom(String),
    ExecuteOperationFailed,
}
