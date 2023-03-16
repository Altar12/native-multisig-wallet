use crate::error::WalletError;
use crate::state::{AccountType, Proposal, ProposalType, VoteCount, WalletAuth, WalletConfig};
use borsh::BorshSerialize;
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
use spl_token::{
    instruction as token_instruction,
    state::{Account, Mint},
    ID as TOKEN_PROGRAM_ID,
};
use std::convert::TryInto;

const OWNER: &'static str = "owner";
const AUTHORITY: &'static str = "authority";
const VOTES: &'static str = "votes";

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
    if proposal_lifetime < 600 {
        return Err(WalletError::TooShortLifetime.into());
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
        is_initialized: true,
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
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let wallet_config = next_account_info(accounts_iter)?;
    let wallet_auth = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if wallet_config.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut wallet = try_from_slice_unchecked::<WalletConfig>(&wallet_config.data.borrow())?;
    if !wallet.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    let (wallet_auth_key, _) = Pubkey::find_program_address(
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

    let mut user_wallet_id = try_from_slice_unchecked::<WalletAuth>(&wallet_auth.data.borrow())?;
    if !user_wallet_id.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    user_wallet_id.is_initialized = false;
    user_wallet_id.serialize(&mut &mut wallet_auth.data.borrow_mut()[..])?;
    let mut balance = wallet_auth.lamports();
    **wallet_auth.try_borrow_mut_lamports()? -= balance;
    **user.try_borrow_mut_lamports()? += balance;

    if wallet.owners == 1 {
        wallet.is_initialized = false;
        wallet.serialize(&mut &mut wallet_config.data.borrow_mut()[..])?;
        balance = wallet_config.lamports();
        **wallet_config.try_borrow_mut_lamports()? -= balance;
        **user.try_borrow_mut_lamports()? += balance;
        if accounts.iter().len() == 0 {
            return Ok(());
        }

        let wallet_authority = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        let (wallet_authority_key, bump) = Pubkey::find_program_address(
            &[AUTHORITY.as_bytes().as_ref(), wallet_config.key.as_ref()],
            program_id,
        );
        if *wallet_authority.key != wallet_authority_key {
            return Err(WalletError::InvalidWalletAuthority.into());
        }
        if *token_program.key != TOKEN_PROGRAM_ID {
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut send_account;
        let mut receive_account;
        let mut amount;
        while accounts_iter.len() > 0 {
            send_account = next_account_info(accounts_iter)?;
            receive_account = next_account_info(accounts_iter)?;
            amount = Account::unpack(&send_account.data.borrow())?.amount;
            invoke_signed(
                &token_instruction::transfer(
                    token_program.key,
                    send_account.key,
                    receive_account.key,
                    wallet_authority.key,
                    &[],
                    amount,
                )?,
                &[
                    send_account.clone(),
                    receive_account.clone(),
                    wallet_authority.clone(),
                ],
                &[&[
                    AUTHORITY.as_bytes().as_ref(),
                    wallet_config.key.as_ref(),
                    &[bump],
                ]],
            )?;
        }
    } else {
        let owner_id: usize = user_wallet_id.id.try_into().unwrap();
        let owner_byte_pos = owner_id / 8;
        let owner_bit_pos = owner_id % 8;
        let mut owner_byte = format!("{:08b}", wallet.owner_identities[owner_byte_pos]);
        owner_byte.replace_range(owner_bit_pos..owner_bit_pos + 1, "0");
        wallet.owner_identities[owner_byte_pos] = u8::from_str_radix(&owner_byte, 2).unwrap();
        wallet.owners -= 1;
        wallet.serialize(&mut &mut wallet_config.data.borrow_mut()[..])?;
    }

    Ok(())
}

pub fn create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_proposal: ProposalType,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let wallet_config = next_account_info(accounts_iter)?;
    let wallet_auth = next_account_info(accounts_iter)?;
    let proposal = next_account_info(accounts_iter)?;
    let vote_count = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if wallet_config.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    if wallet_auth.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let (wallet_auth_key, _) = Pubkey::find_program_address(
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
    if !proposal.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (vote_count_key, bump) = Pubkey::find_program_address(
        &[
            VOTES.as_bytes().as_ref(),
            wallet_config.key.as_ref(),
            proposal.key.as_ref(),
        ],
        program_id,
    );
    if *vote_count.key != vote_count_key {
        return Err(WalletError::InvalidVoteCount.into());
    }
    if *system_program.key != SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // create proposal account
    let mut account_size: u64 = Proposal::LEN.try_into().unwrap();
    let mut rent_amount = Rent::get()?.minimum_balance(Proposal::LEN);
    invoke(
        &system_instruction::create_account(
            user.key,
            proposal.key,
            rent_amount,
            account_size,
            program_id,
        ),
        &[user.clone(), proposal.clone()],
    )?;
    // initialize proposal account
    let proposal_details = Proposal {
        discriminator: AccountType::Proposal,
        wallet: *wallet_config.key,
        proposer: *user.key,
        proposal: new_proposal,
        is_initialized: true,
    };
    proposal_details.serialize(&mut &mut proposal.data.borrow_mut()[..])?;
    // create vote count account
    account_size = VoteCount::LEN.try_into().unwrap();
    rent_amount = Rent::get()?.minimum_balance(VoteCount::LEN);
    invoke_signed(
        &system_instruction::create_account(
            user.key,
            vote_count.key,
            rent_amount,
            account_size,
            program_id,
        ),
        &[user.clone(), vote_count.clone()],
        &[&[
            VOTES.as_bytes().as_ref(),
            wallet_config.key.as_ref(),
            proposal.key.as_ref(),
            &[bump],
        ]],
    )?;
    // initialize vote count account
    let user_wallet_id = try_from_slice_unchecked::<WalletAuth>(&wallet_auth.data.borrow())?;
    if !user_wallet_id.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    let owner_id: usize = user_wallet_id.id.try_into().unwrap();
    let owner_byte_pos = owner_id / 8;
    let owner_bit_pos = owner_id % 8;
    let mut owner_byte_str = String::new();
    for _ in 0..owner_bit_pos {
        owner_byte_str.push('0');
    }
    owner_byte_str.push('1');
    for _ in owner_bit_pos + 1..8 {
        owner_byte_str.push('0');
    }
    let mut vote_record = [0u8; 32];
    vote_record[owner_byte_pos] = u8::from_str_radix(&owner_byte_str, 2).unwrap();
    let voting_details = VoteCount {
        discriminator: AccountType::VoteCount,
        proposed_time: Clock::get()?.unix_timestamp,
        votes: 1,
        vote_record,
        is_initialized: true,
    };
    voting_details.serialize(&mut &mut vote_count.data.borrow_mut()[..])?;
    Ok(())
}

pub fn vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let wallet_config = next_account_info(accounts_iter)?;
    let wallet_auth = next_account_info(accounts_iter)?;
    let proposal = next_account_info(accounts_iter)?;
    let vote_count = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if wallet_config.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let (wallet_auth_key, _) = Pubkey::find_program_address(
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
    if proposal.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let (vote_count_key, _) = Pubkey::find_program_address(
        &[
            VOTES.as_bytes().as_ref(),
            wallet_config.key.as_ref(),
            proposal.key.as_ref(),
        ],
        program_id,
    );
    if *vote_count.key != vote_count_key {
        return Err(WalletError::InvalidVoteCount.into());
    }
    // check that proposal is active
    let wallet = try_from_slice_unchecked::<WalletConfig>(&wallet_config.data.borrow())?;
    if !wallet.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    let lifetime = wallet.proposal_lifetime;
    let mut voting_details = try_from_slice_unchecked::<VoteCount>(&vote_count.data.borrow())?;
    if !voting_details.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    if Clock::get()?.unix_timestamp > voting_details.proposed_time + lifetime {
        return Err(WalletError::ProposalExpired.into());
    }
    // check that user has not voted yet
    let user_wallet_id = try_from_slice_unchecked::<WalletAuth>(&wallet_auth.data.borrow())?;
    if !user_wallet_id.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    let owner_id: usize = user_wallet_id.id.try_into().unwrap();
    let owner_byte_pos = owner_id / 8;
    let owner_bit_pos = owner_id % 8;
    let mut owner_byte_str = format!("{:08b}", voting_details.vote_record[owner_byte_pos]);
    if let Some("1") = owner_byte_str.get(owner_bit_pos..owner_bit_pos + 1) {
        return Err(WalletError::AlreadyVoted.into());
    }
    owner_byte_str.replace_range(owner_bit_pos..owner_bit_pos + 1, "1");
    voting_details.vote_record[owner_byte_pos] = u8::from_str_radix(&owner_byte_str, 2).unwrap();
    voting_details.votes += 1;
    voting_details.serialize(&mut &mut vote_count.data.borrow_mut()[..])?;
    Ok(())
}

pub fn close_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
