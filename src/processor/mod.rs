mod handler;

use crate::instruction::WalletInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let instruction = WalletInstruction::unpack(data)?;
    match instruction {
        WalletInstruction::CreateWallet {
            m,
            n,
            owners,
            proposal_lifetime,
        } => handler::create_wallet(program_id, accounts, m, n, &owners, proposal_lifetime),
        WalletInstruction::CreateTokenAccount => {
            handler::create_token_account(program_id, accounts)
        }
        WalletInstruction::GiveupOwnership => handler::give_up_ownership(program_id, accounts),
        WalletInstruction::CreateProposal { proposal } => {
            handler::create_proposal(program_id, accounts, proposal)
        }
        WalletInstruction::Vote => handler::vote(program_id, accounts),
        WalletInstruction::CloseProposal => handler::close_proposal(program_id, accounts),
    }
}
