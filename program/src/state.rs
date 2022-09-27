use solana_program :: {
    pubkey::Pubkey
};
use borsh::{BorshDeserialize, BorshSerialize, BorshSchema};

#[derive(Clone,Copy,Debug,BorshSerialize,BorshDeserialize,BorshSchema,PartialEq)]
pub struct Escrow {
    pub user_sender : Pubkey,
    pub is_initialized : bool,
    pub escrow_token_account : Pubkey,
    pub senders_token_receiver_account : Pubkey,
    pub expected_amount : u64
}

