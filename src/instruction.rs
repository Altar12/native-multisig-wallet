use solana_program::pubkey::Pubkey;

use crate::state::ProposalType;

pub enum WalletInstruction {
    /*
    User: signer, mutable
    WalletConfig: signer, mutable
    WalletAuth: mutable
    SystemProgram
    WalletAuths: mutable - optional multiple accounts
     */
    CreateWallet { m: u8, n: u8, owners: Vec<Pubkey> },
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
    Owner: signer, mutable
    WalletConfig
     */
    GiveupOwnership,
    CreateProposal { proposal: ProposalType },
    Vote,
    CloseProposal,
}
