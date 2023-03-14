use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet parameter constraint violdated, m=0 or m>n")]
    InvalidWalletParameters,
    #[error("Invalid Wallet Auth account passed")]
    InvalidWalletAuth,
    #[error("Number of owner keys passed does not equal to number of Wallet Auth accounts passed")]
    OwnerWalletAuthCountMismatch,
}

impl From<WalletError> for ProgramError {
    fn from(value: WalletError) -> Self {
        Self::Custom(value as u32)
    }
}
