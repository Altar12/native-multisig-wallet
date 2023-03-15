use crate::error::WalletError;
use crate::state::{AccountType, ProposalType, WalletAuth, WalletConfig};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
    ID as ASSOCIATED_TOKEN_PROGRAM_ID,
};
use spl_token::{state::Mint, ID as TOKEN_PROGRAM_ID};
use std::convert::TryInto;

const OWNER: &'static str = "owner";
const AUTHORITY: &'static str = "authority";

pub fn create_wallet(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    m: u8,
    n: u8,
    owners: &Vec<Pubkey>,
    proposal_lifetime: i64,
) -> ProgramResult {
    if m == 0 || m > n {
        return Err(WalletError::InvalidWalletParameters.into());
    }
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let wallet_config = next_account_info(accounts_iter)?;
    let mut wallet_auth = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !user.is_signer || !wallet_config.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (mut wallet_auth_key, mut bump) = Pubkey::find_program_address(
        &[
            OWNER.as_bytes().as_ref(),
            wallet_config.key.as_ref(),
            user.key.as_ref(),
        ],
        program_id,
    );
    if *wallet_auth.key != wallet_auth_key {
        return Err(WalletError::InvalidWalletAuth.into());
    }
    if *system_program.key != SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if owners.len() != accounts_iter.len() {
        return Err(WalletError::OwnerWalletAuthCountMismatch.into());
    }
    // create user's wallet auth account
    let account_size: u64 = WalletAuth::LEN.try_into().unwrap();
    let rent_amount = Rent::get()?.minimum_balance(WalletAuth::LEN);
    invoke_signed(
        &system_instruction::create_account(
            user.key,
            wallet_auth.key,
            rent_amount,
            account_size,
            program_id,
        ),
        &[user.clone(), wallet_auth.clone()],
        &[&[
            OWNER.as_bytes().as_ref(),
            wallet_config.key.as_ref(),
            user.key.as_ref(),
            &[bump],
        ]],
    )?;
    // initialize user's wallet auth account
    let current_time = Clock::get()?.unix_timestamp;
    let mut user_wallet_auth = WalletAuth {
        discriminator: AccountType::WalletAuth,
        owner: *user.key,
        wallet: *wallet_config.key,
        added_time: current_time,
        id: 0,
    };
    user_wallet_auth.serialize(&mut &mut wallet_auth.data.borrow_mut()[..])?;
    // create and initialize wallet auth accounts for other owners
    let mut id = 1;
    for owner in owners.iter() {
        wallet_auth = next_account_info(accounts_iter)?;
        (wallet_auth_key, bump) = Pubkey::find_program_address(
            &[
                OWNER.as_bytes().as_ref(),
                wallet_config.key.as_ref(),
                owner.as_ref(),
            ],
            program_id,
        );
        if *wallet_auth.key != wallet_auth_key {
            return Err(WalletError::InvalidWalletAuth.into());
        }
        invoke_signed(
            &system_instruction::create_account(
                user.key,
                wallet_auth.key,
                rent_amount,
                account_size,
                program_id,
            ),
            &[user.clone(), wallet_auth.clone()],
            &[&[
                OWNER.as_bytes().as_ref(),
                wallet_config.key.as_ref(),
                owner.as_ref(),
                &[bump],
            ]],
        )?;
        user_wallet_auth.owner = *owner;
        user_wallet_auth.id = id;
        id += 1;
        user_wallet_auth.serialize(&mut &mut wallet_auth.data.borrow_mut()[..])?;
    }
    // create wallet config account
    let account_size: u64 = WalletConfig::LEN.try_into().unwrap();
    let rent_amount = Rent::get()?.minimum_balance(WalletConfig::LEN);
    invoke(
        &system_instruction::create_account(
            user.key,
            wallet_auth.key,
            rent_amount,
            account_size,
            program_id,
        ),
        &[user.clone(), wallet_auth.clone()],
    )?;
    // initialize wallet config account
    let owner_count = 1 + owners.len();
    let mut identities = [0u8; 32];
    let last_owner_byte = (owner_count - 1) / 8;
    let last_owner_pos = (owner_count - 1) % 8;
    for i in 0..last_owner_byte {
        identities[i] = 255;
    }
    let mut identity_str = String::new();
    for _ in 0..=last_owner_pos {
        identity_str.push('1');
    }
    for _ in last_owner_pos + 1..8 {
        identity_str.push('0');
    }
    identities[last_owner_byte] = u8::from_str_radix(&identity_str, 2).unwrap();
    let wallet_info = WalletConfig {
        discriminator: AccountType::WalletConfig,
        m,
        n,
        owners: owner_count.try_into().unwrap(),
        owner_identities: identities,
        proposal_lifetime,
        is_initialized: true,
    };
    wallet_info.serialize(&mut &mut wallet_config.data.borrow_mut()[..])?;
    Ok(())
}

pub fn create_token_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let wallet_config = next_account_info(accounts_iter)?;
    let wallet_authority = next_account_info(accounts_iter)?;
    let mint = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if wallet_config.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let wallet = try_from_slice_unchecked::<WalletConfig>(&wallet_config.data.borrow())?;
    if !wallet.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    let (wallet_authority_key, _) = Pubkey::find_program_address(
        &[AUTHORITY.as_bytes().as_ref(), wallet_config.key.as_ref()],
        program_id,
    );
    if *wallet_authority.key != wallet_authority_key {
        return Err(WalletError::InvalidWalletAuthority.into());
    }
    if *mint.owner != TOKEN_PROGRAM_ID {
        return Err(WalletError::InvalidMint.into());
    }
    if let Err(_) = Mint::unpack(&mint.data.borrow()) {
        return Err(WalletError::InvalidMint.into());
    }
    let ata_key = get_associated_token_address(wallet_authority.key, mint.key);
    if *token_account.key != ata_key {
        return Err(WalletError::IncorrectAssociatedTokenAccount.into());
    }
    if *system_program.key != SYSTEM_PROGRAM_ID
        || *token_program.key != TOKEN_PROGRAM_ID
        || *associated_token_program.key != ASSOCIATED_TOKEN_PROGRAM_ID
    {
        return Err(ProgramError::IncorrectProgramId);
    }
    invoke(
        &create_associated_token_account(
            payer.key,
            wallet_authority.key,
            mint.key,
            token_program.key,
        ),
        &[
            payer.clone(),
            token_account.clone(),
            wallet_authority.clone(),
            mint.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;
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
