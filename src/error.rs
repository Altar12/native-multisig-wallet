use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet parameter constraint violdated, m=0 or m>n")]
    InvalidWalletParameters,
    #[error("Proposal lifetime specified is less than 10 minutes")]
    TooShortLifetime,
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
    #[error("The proposal has already expired")]
    ProposalExpired,
    #[error("User has already voted for the given proposal")]
    AlreadyVoted,
    #[error("The account passed for Proposer does not correspond to the actual proposer")]
    IncorrectProposer,
    #[error("The proposal has insufficient votes to execute")]
    InsufficientVotes,
    #[error("The Send Account specified for transferring funds is incorrect")]
    IncorrectSendAccount,
    #[error("The Receive Account specified for receiving funds is incorrect")]
    IncorrectReceiveAccount,
    #[error("The wallet already has maximum number of owners")]
    MaximumOwnersReached,
}

impl From<WalletError> for ProgramError {
    fn from(value: WalletError) -> Self {
        Self::Custom(value as u32)
    }
}
