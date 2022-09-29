use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Copy, Clone, Debug)]
pub enum EscrowError {
    #[error("Invalid Data")]
    InvalidData,
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Escrow Already Initialized")]
    EscrowAlreadyInitialized,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
