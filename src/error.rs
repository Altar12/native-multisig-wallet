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
    #[error("Invalid Wallet Authority account passed")]
    InvalidWalletAuthority,
    #[error("The account passed for Mint does not corresponds to a valid mint")]
    InvalidMint,
    #[error("The account passed for Associated Token Account is incorrect")]
    IncorrectAssociatedTokenAccount,
    #[error("Invalid Vote Count account passed")]
    InvalidVoteCount,
}

impl From<WalletError> for ProgramError {
    fn from(value: WalletError) -> Self {
        Self::Custom(value as u32)
    }
}
