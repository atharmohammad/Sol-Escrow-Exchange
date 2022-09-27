use crate::id;
use crate::error::{self, EscrowError};
use solana_program::{
    program_error::{ProgramError}, pubkey::Pubkey, sysvar::{rent, self}, instruction::{Instruction, AccountMeta},
};
use borsh::{BorshDeserialize,BorshSerialize};
#[derive(Debug,BorshDeserialize,BorshSerialize,Clone,PartialEq)]
pub struct Payload {
    pub variant : u8,
    pub arg1 : u64
}
#[derive(Debug,BorshDeserialize,BorshSerialize,Clone,PartialEq)]
pub enum EscrowInstruction{
    /// Starts the trade by creating a escrow account owned by program and storing the amount of tokens that are getting exchanged
    /// accounts required :
    /// 0 - [signer] Account which is starting the escrow trade
    /// 1 - [writer] Token Account which will be sending Y tokens to escrow
    /// 2 - [] Account of user that will be receiving the Y tokens
    /// 3 - [writer] escrow account which will hold all info and tokens
    /// 4 - [] rent sysvar
    /// 5 - [] token program
    Initialize{
        amount : u64
    }
}

pub fn initialize(user_sender:&Pubkey,senders_token_account:&Pubkey,escrow_token_account:&Pubkey,escrow_account:&Pubkey,rent:&Pubkey,token_program:&Pubkey,amount:u64) -> Instruction {
    Instruction::new_with_borsh(id(), &EscrowInstruction::Initialize { amount },
     vec![
        AccountMeta::new(*user_sender, true),
        AccountMeta::new(*senders_token_account, false),
        AccountMeta::new(*escrow_token_account, false),
        AccountMeta::new(*escrow_account, false),
        AccountMeta::new(*rent, false),
        AccountMeta::new(*token_program, false),
     ])
}
