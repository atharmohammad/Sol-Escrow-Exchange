use crate::error::{self, EscrowError};
use crate::id;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{self, rent},
};
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
pub struct Payload {
    pub variant: u8,
    pub arg1: u64,
}
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
pub enum EscrowInstruction {
    /// Starts the trade by creating a escrow account owned by program and storing the amount of tokens that are getting exchanged
    /// accounts required :
    /// 0 - [signer] Account which is starting the escrow trade
    /// 1 - [] Account of user that will be receiving the Y tokens
    /// 2 - [writer] Token Account which will be sending Y tokens to escrow
    /// 3 - [writer] escrow account which will hold all info and tokens
    /// 4 - [] rent sysvar
    /// 5 - [] token program
    Initialize { amount: u64 },
    /// Compelete the trade by exchanging the token with escrow account
    /// accounts required :
    /// 0 - [signer] Account which is compeleting the escrow trade
    /// 1 - [writer] Token Account which will be sending Y tokens to alice
    /// 2 - [] Account of user that will be receiving the Y tokens
    /// 3 - [] token program
    Compelete { amount: u64 },
}

pub fn initialize(
    user_sender: &Pubkey,
    senders_token_account: &Pubkey,
    escrow_token_account: &Pubkey,
    escrow_account: &Pubkey,
    rent: &Pubkey,
    token_program: &Pubkey,
    amount: u64,
) -> Instruction {
    Instruction::new_with_borsh(
        id(),
        &EscrowInstruction::Initialize { amount },
        vec![
            AccountMeta::new(*user_sender, true),
            AccountMeta::new(*senders_token_account, false),
            AccountMeta::new(*escrow_token_account, false),
            AccountMeta::new(*escrow_account, false),
            AccountMeta::new(*rent, false),
            AccountMeta::new(*token_program, false),
        ],
    )
}
