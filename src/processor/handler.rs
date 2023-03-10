use crate::state::ProposalType;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn create_wallet(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    m: u8,
    n: u8,
    owners: &Vec<Pubkey>,
) -> ProgramResult {
    Ok(())
}

pub fn create_token_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

pub fn give_up_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

pub fn create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proposal: ProposalType,
) -> ProgramResult {
    Ok(())
}

pub fn vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

pub fn close_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
