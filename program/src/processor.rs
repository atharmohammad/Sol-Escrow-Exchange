use borsh::BorshDeserialize;
use solana_program::{
    entrypoint::{ProgramResult},
    pubkey::Pubkey,
    account_info::{AccountInfo,next_account_info},
    msg,
    program_error::ProgramError, rent::Rent, sysvar::Sysvar, bpf_loader::id, program::invoke
};
use spl_token::instruction::set_authority;
use crate::{instructions::*, error::EscrowError};
use crate::state::*;

pub fn process_instruction(
    _program_id:&Pubkey,
    accounts:&[AccountInfo],
    input:&[u8]
) -> ProgramResult {
    let instruction = EscrowInstruction::try_from_slice(input)?;
    match instruction {
        EscrowInstruction::Initialize { amount } =>{
            msg!("Instruction : Initialize the Escrow");
            let accounts_iter = &mut accounts.iter();
            let user_sender = next_account_info(accounts_iter)?;
            if !user_sender.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let senders_token_account = next_account_info(accounts_iter)?;
            let escrow_token_account = next_account_info(accounts_iter)?; // senders temporary token account to transfer it's ownership to program and receive the trade of tokens
            if *senders_token_account.owner != spl_token::id() {
                return Err(ProgramError::IllegalOwner);
            }
            let escrow_wallet = next_account_info(accounts_iter)?;
            let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;
            if !rent.is_exempt(escrow_wallet.lamports(), escrow_wallet.data_len()) {
                return Err(ProgramError::AccountNotRentExempt);
            }
            let mut escrow_info = Escrow::try_from_slice(&escrow_wallet.data.borrow())?;
            let token_program = next_account_info(accounts_iter)?;
            if escrow_info.is_initialized {
                return Err(EscrowError::EscrowAlreadyInitialized.into());
            }
            escrow_info.user_sender = *user_sender.key;
            escrow_info.is_initialized = true;
            escrow_info.expected_amount = amount;
            escrow_info.escrow_token_account = *escrow_token_account.key;
            escrow_info.senders_token_receiver_account = *senders_token_account.key;
            // creating pda for escrow token account
            let (pda,_bump) = Pubkey::find_program_address(&[b"token"], &id());
            // transferring ownership of pda to program from sender to use this account as an escrow for sending Y tokens to Bob
            let ownership_inst = set_authority(token_program.key, escrow_token_account.key, Some(&pda), spl_token::instruction::AuthorityType::AccountOwner, &id(), &[&user_sender.key])?;
            invoke(&ownership_inst, 
                &[
                    token_program.clone(),
                    escrow_token_account.clone(),
                    user_sender.clone()
                ]
            )?;
            
            Ok(())
        }
    }
}