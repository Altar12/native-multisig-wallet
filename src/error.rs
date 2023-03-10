use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[err("Wallet parameter constraint violdated, m=0 or m>n")]
    InvalidWalletParameters,
}

impl From<WalletError> for ProgramError {
    fn from(value: WalletError) -> Self {
        Self::Custom(value.into())
    }
}
