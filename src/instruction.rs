use crate::state::ProposalType;
use borsh::BorshDeserialize;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::convert::TryInto;

pub enum WalletInstruction {
    /*
    User: signer, mutable
    WalletConfig: signer, mutable
    WalletAuth: mutable ["owner", wallet_config.key, user.key]
    SystemProgram
    WalletAuths: mutable - optional multiple accounts
     */
    CreateWallet {
        m: u8,
        n: u8,
        owners: Vec<Pubkey>,
        proposal_lifetime: i64,
    },
    /*
    Payer: signer, mutable
    WalletConfig
    WalletAuthority
    Mint
    AssociatedTokenAccount: mutable
    SystemProgram
    TokenProgram
    AssociatedTokenProgram
     */
    CreateTokenAccount,
    /*
    User: signer, mutable
    WalletConfig: mutable
    WalletAuth: mutable ["owner", wallet_config.key, user.key]
    ...all below accounts can be either present or not...
    WalletAuthority
    TokenProgram
    pairs of send and receive accounts
     */
    GiveupOwnership,
    /*
    User: signer, mutable
    WalletConfig
    WalletAuth ["owner", wallet_config.key, user.key]
    Proposal: signer, mutable
    VoteCount: mutable ["votes", wallet_config.key, proposal.key]
    SystemProgram
     */
    CreateProposal {
        proposal: ProposalType,
    },
    /*
    User: signer
    WalletConfig
    WalletAuth ["owner", wallet_config.key, user.key]
    Proposal
    VoteCount: mutable ["votes", wallet_config.key, proposal.key]
     */
    Vote,
    /*
    Proposer: mutable
    WalletConfig: mutable(if ChangeLifetime or AddOwner proposal)
    Proposal: mutable
    VoteCount: mutable ["votes", wallet_config.key, proposal.key]
    ...rest of the accounts vary depending on the proposal type and only required if proposal is still valid and got majority votes...
    ...for Transfer
    SendAccount: mutable
    ReceiveAccount: mutable
    WalletAuthority
    TokenProgram
    ...for AddOwner
    WalletAuth: mutable ["owner", wallet_config.key, user.key] user present in proposal
    SystemProgram
    ...for ChangeLifetime no other accounts required
     */
    CloseProposal,
}

impl WalletInstruction {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        let res = match variant {
            0 => {
                let (&m, rest) = rest
                    .split_first()
                    .ok_or(ProgramError::InvalidInstructionData)?;
                let (&n, rest) = rest
                    .split_first()
                    .ok_or(ProgramError::InvalidInstructionData)?;
                let proposal_lifetime = i64::deserialize(&mut &rest[..8])?;
                let rest = &rest[8..];
                if rest.len() == 0 {
                    Self::CreateWallet {
                        m,
                        n,
                        owners: Vec::new(),
                        proposal_lifetime,
                    }
                } else {
                    let mut owners = Vec::new();
                    let (&owner_count, rest) = rest.split_first().unwrap();
                    let owner_count = owner_count as usize;
                    let mut count = 0;
                    while count < owner_count {
                        owners.push(Pubkey::deserialize(&mut &rest[count..count + 32]).unwrap());
                        count += 32;
                    }
                    Self::CreateWallet {
                        m,
                        n,
                        owners,
                        proposal_lifetime,
                    }
                }
            }
            1 => Self::CreateTokenAccount,
            2 => Self::GiveupOwnership,
            3 => {
                let (&proposal_type, rest) = rest
                    .split_first()
                    .ok_or(ProgramError::InvalidInstructionData)?;
                match proposal_type {
                    0 => {
                        let token_mint = Pubkey::deserialize(&mut &rest[0..32])?;
                        let receive_account = Pubkey::deserialize(&mut &rest[32..64])?;
                        let amount = u64::from_be_bytes((&rest[64..]).try_into().unwrap());
                        Self::CreateProposal {
                            proposal: ProposalType::Transfer {
                                token_mint,
                                receive_account,
                                amount,
                            },
                        }
                    }
                    1 => {
                        let user = Pubkey::deserialize(&mut &rest[..])?;
                        Self::CreateProposal {
                            proposal: ProposalType::AddOwner { user },
                        }
                    }
                    2 => {
                        let duration = i64::from_be_bytes(rest.try_into().unwrap());
                        Self::CreateProposal {
                            proposal: ProposalType::ChangeProposalLifetime { duration },
                        }
                    }
                    _ => return Err(ProgramError::InvalidInstructionData),
                }
            }
            4 => Self::Vote,
            5 => Self::CloseProposal,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(res)
    }
}
