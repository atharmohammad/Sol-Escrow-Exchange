use solana_program :: {
    pubkey::Pubkey
};

pub struct Escrow {
    pub user_sender : Pubkey,
    pub is_initialized : bool,
    pub user_receiver : Pubkey,
    pub escrow_token_account : Pubkey,
    pub senders_token_receiver_account : Pubkey,
    pub expected_amount : u64
}