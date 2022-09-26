use solana_program::{
    entrypoint::{ProgramResult},
    pubkey::Pubkey,
    account_info::{AccountInfo,next_account_info},
    msg,
    program_error::ProgramError, rent::Rent, sysvar::Sysvar
};
use crate::{instructions::*, error::EscrowError};
use crate::state::*;

pub fn process_instruction(
    program_id:&Pubkey,
    accounts:&[AccountInfo],
    input:&[u8]
) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(input);
    match instruction {
        EscrowInstruction::Initialize { amount } =>{
            msg!("Instruction : Initialize the Escrow");
            let accounts_iter = &mut accounts.iter();
            let user_sender = next_account_info(accounts_iter)?;
            if !user_sender.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let senders_token_receiver_account = next_account_info(accounts_iter)?;
            let escrow_token_account = next_account_info(accounts_iter)?;
            if *senders_token_receiver_account.owner != spl_token::id() {
                return Err(ProgramError::IllegalOwner);
            }
            let escrow_wallet = next_account_info(accounts_iter)?;
            let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;
            if !rent.is_exempt(escrow_wallet.lamports(), escrow_wallet.data_len()) {
                return Err(ProgramError::AccountNotRentExempt);
            }
            let mut escrow_info = Escrow::unpack_unchecked(escrow_wallet.try_borrow_data()?)?;
            let token_program = next_account_info(accounts_iter)?;
            if escrow_info.is_initialized {
                return Err(EscrowError::EscrowAlreadyInitialized.into());
            }
            Ok(())
        }
    }
}