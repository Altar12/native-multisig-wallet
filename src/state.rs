use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum AccountType {
    WalletConfig,
    WalletAuth,
    Proposal,
    VoteCount,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum ProposalType {
    Transfer {
        token_mint: Pubkey,
        receive_account: Pubkey,
        amount: u64,
    },
    AddOwner {
        user: Pubkey,
    },
    ChangeProposalLifetime {
        duration: i64,
    },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct WalletConfig {
    discriminator: AccountType,
    m: u8,
    n: u8,
    owners: u8,
    owner_identities: [u8; 32],
    proposal_lifetime: i64,
    is_initialized: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct WalletAuth {
    discriminator: AccountType,
    owner: Pubkey,
    wallet: Pubkey,
    added_time: i64,
    id: u8,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    discriminator: AccountType,
    wallet: Pubkey,
    proposer: Pubkey,
    proposal: ProposalType,
    is_initialized: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VoteCount {
    discriminator: AccountType,
    proposed_time: i64,
    votes: u8,
    vote_record: [u8; 32],
}

impl IsInitialized for WalletConfig {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl IsInitialized for Proposal {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for WalletConfig {}
impl Pack for WalletConfig {
    const LEN: usize = std::mem::size_of::<Self>();
    fn pack_into_slice(&self, dst: &mut [u8]) {
        self.serialize(&mut &mut dst[..]).unwrap()
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if let Ok(result) = Self::deserialize(&mut &src[..]) {
            Ok(result)
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
}
impl Sealed for WalletAuth {}
impl Pack for WalletAuth {
    const LEN: usize = std::mem::size_of::<Self>();
    fn pack_into_slice(&self, dst: &mut [u8]) {
        self.serialize(&mut &mut dst[..]).unwrap()
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if let Ok(result) = Self::deserialize(&mut &src[..]) {
            Ok(result)
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
}
impl Sealed for Proposal {}
impl Pack for Proposal {
    const LEN: usize = std::mem::size_of::<Self>();
    fn pack_into_slice(&self, dst: &mut [u8]) {
        self.serialize(&mut &mut dst[..]).unwrap()
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if let Ok(result) = Self::deserialize(&mut &src[..]) {
            Ok(result)
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
}
impl Sealed for VoteCount {}
impl Pack for VoteCount {
    const LEN: usize = std::mem::size_of::<Self>();
    fn pack_into_slice(&self, dst: &mut [u8]) {
        self.serialize(&mut &mut dst[..]).unwrap()
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if let Ok(result) = Self::deserialize(&mut &src[..]) {
            Ok(result)
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
}
