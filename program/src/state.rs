
use solana_program :: {
    pubkey::Pubkey,
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
    msg
};
use borsh::{BorshDeserialize, BorshSerialize, BorshSchema};

#[derive(Clone,Copy,Debug,BorshSerialize,BorshDeserialize,BorshSchema,PartialEq)]
pub struct Escrow {
    pub user_sender : Pubkey, // 32
    pub is_initialized : u8, // 1
    pub escrow_token_account : Pubkey, //32
    pub senders_token_receiver_account : Pubkey, //32
    pub expected_amount : u64 // 8
}

impl Sealed for Escrow {}

impl Pack for Escrow {
    const LEN: usize = 32+1+32+32+8;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        Escrow::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize name record");
            ProgramError::InvalidAccountData
        })
    }
}
