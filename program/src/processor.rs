use crate::state::*;
use crate::{error::EscrowError, instructions::*};
use crate::id;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token::instruction::close_account;
use spl_token::instruction::{set_authority, transfer};

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    msg!("program starts!");
    let instruction =
        Payload::try_from_slice(input).map_err(|_| ProgramError::InvalidInstructionData)?;
    msg!("deserialize instruction!");

    match instruction.variant {
        0 => {
            msg!("Instruction : Initialize the Escrow");
            let accounts_iter = &mut accounts.iter();
            let user_sender = next_account_info(accounts_iter)?;
            msg!("check is sender a signer");
            if !user_sender.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let senders_token_account = next_account_info(accounts_iter)?;
            let escrow_token_account = next_account_info(accounts_iter)?; // senders temporary token account to transfer it's ownership to program and receive the trade of tokens

            msg!("check is senders token account owned by token program");
            if *senders_token_account.owner != spl_token::id() {
                return Err(ProgramError::IllegalOwner);
            }
            let escrow_wallet = next_account_info(accounts_iter)?;
            let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;
            msg!("check is rent exempt on escrow wallet");
            if !rent.is_exempt(escrow_wallet.lamports(), escrow_wallet.data_len()) {
                return Err(ProgramError::AccountNotRentExempt);
            }
            msg!("deserialize escrow account");
            let mut escrow_info = Escrow::unpack_unchecked(*escrow_wallet.data.borrow())?;
            let token_program = next_account_info(accounts_iter)?;
            msg!("check is escrow initialized !");
            if escrow_info.is_initialized == 1 {
                return Err(EscrowError::EscrowAlreadyInitialized.into());
            }
            escrow_info.user_sender = *user_sender.key;
            escrow_info.is_initialized = 1;
            escrow_info.expected_amount = instruction.arg1;
            escrow_info.escrow_token_account = *escrow_token_account.key;
            escrow_info.senders_token_receiver_account = *senders_token_account.key;
            msg!("serializing the escrow account !");
            escrow_info.serialize(&mut &mut escrow_wallet.data.borrow_mut()[..])?;
            msg!("creating pda for escrow token account !");

            let (pda, _bump) = Pubkey::find_program_address(&[b"token"], &id());
            msg!("transferring ownership of pda to program from sender to use this account as an escrow for sending Y tokens to Bob !");
            let ownership_inst = set_authority(
                &token_program.key,
                &escrow_token_account.key,
                Some(&pda),
                spl_token::instruction::AuthorityType::AccountOwner,
                &user_sender.key,
                &[&user_sender.key],
            )?;
            invoke(
                &ownership_inst,
                &[
                    token_program.clone(),
                    escrow_token_account.clone(),
                    user_sender.clone(),
                ],
            )?;

            Ok(())
        }
        1 => {
            msg!("Instruction : Compelete the escrow payment !");
            let account_iter = &mut accounts.iter();
            let user_receiver = next_account_info(account_iter)?;
            msg!("recevier is a signer to compelete the escrow trade !");
            if !user_receiver.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let token_account = next_account_info(account_iter)?; // this token account will send tokens to alice
            msg!("is token account owned by spl token program !");
            if *token_account.owner != spl_token::id() {
                return Err(ProgramError::IllegalOwner);
            }
            let receving_token_account = next_account_info(account_iter)?; // this token account will receive tokens from escrow
            msg!("is receiver token account owned by spl token program !");
            if *receving_token_account.owner != spl_token::id() {
                return Err(ProgramError::IllegalOwner);
            }
            let (pda, _bump) = Pubkey::find_program_address(&[b"token"], &id());
            let escrow_wallet = next_account_info(account_iter)?;
            msg!("Deserailize the escrow wallet state !");
            let escrow_info = Escrow::unpack_unchecked(&escrow_wallet.try_borrow_data()?)?;
            let token_program = next_account_info(account_iter)?;
            let token_pda = next_account_info(account_iter)?; // pda account for cpi calls
            msg!("check is pda account passed is same as defined while initializing the escrow");
            if *token_pda.key != pda {
                msg!("{} {}",*token_pda.key,pda);
                return Err(ProgramError::IncorrectProgramId);
            }
            let escrow_token_account = next_account_info(account_iter)?;
            msg!("check is escrow token account account passed is same as defined while initializing the escrow");
            if *escrow_token_account.key != escrow_info.escrow_token_account {
                return Err(ProgramError::IncorrectProgramId);
            }
            let senders_token_receiver_account = next_account_info(account_iter)?;
            msg!("check is senders token account passed is same as defined while initializing the escrow");
            if *senders_token_receiver_account.key != escrow_info.senders_token_receiver_account {
                return Err(ProgramError::IncorrectProgramId);
            }
            let user_sender = next_account_info(account_iter)?;
            msg!("check is user sender account passed is same as defined while initializing the escrow");
            if *user_sender.key != escrow_info.user_sender {
                return Err(ProgramError::IncorrectProgramId);
            }

            msg!("transfer token from bob to alice token receiver account");
            let transfer_to_alice = transfer(
                &token_program.key,
                &token_account.key,
                &escrow_info.senders_token_receiver_account,
                &user_receiver.key,
                &[&user_receiver.key],
                escrow_info.expected_amount,
            )?;

            invoke(
                &transfer_to_alice,
                &[
                    token_program.clone(),
                    token_account.clone(),
                    senders_token_receiver_account.clone(),
                    user_receiver.clone(),
                ],
            )?;

            msg!("transfer token from escrow to bob");
            let transfer_to_bob = transfer(
                &token_program.key,
                &escrow_info.escrow_token_account,
                &receving_token_account.key,
                &pda,
                &[&pda],
                instruction.arg1,
            )?;
            invoke_signed(
                &transfer_to_bob,
                &[
                    token_program.clone(),
                    token_pda.clone(),
                    receving_token_account.clone(),
                    escrow_token_account.clone(),
                ],
                &[&[&b"token"[..], &[_bump]]],
            )?;

            msg!("close the escrow token account and pda has the authorization and rent fee should be sent to alice");
            let close_account_inst = close_account(
                &token_program.key,
                &escrow_info.escrow_token_account,
                &escrow_info.user_sender,
                &pda,
                &[&pda],
            )?;
            invoke_signed(
                &close_account_inst,
                &[
                    token_program.clone(),
                    token_pda.clone(),
                    user_sender.clone(),
                    escrow_token_account.clone(),
                ],
                &[&[&b"token"[..], &[_bump]]],
            )?;
            // msg!("closing escrow account that is holding state info");
            // let close_escrow_inst = close_account(
            //     &token_program.key,
            //     &escrow_wallet.key,
            //     &user_sender.key,
            //     &user_sender.key,
            //     &[&user_sender.key],
            // )?;
            // invoke(
            //     &close_escrow_inst,
            //     &[
            //         token_program.clone(),
            //         escrow_wallet.clone(),
            //         user_sender.clone(),
            //     ],
            // )?;
            Ok(())
        }
        _ => return Err(ProgramError::InvalidArgument),
    }
}
