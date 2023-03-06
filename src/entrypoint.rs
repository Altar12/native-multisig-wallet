use processor::process_instruction;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::{entrypoint, ProgramResult},
    pubkey::Pubkey,
};

entrypoint!(entrypoint_function);

pub fn entrypoint_function(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    process_instruction(program_id, accounts, data)
}
