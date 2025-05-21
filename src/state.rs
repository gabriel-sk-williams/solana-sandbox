// state.rs

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    pubkey::Pubkey,
};

// TODO: change to Option<f64> which is 1 + 1 bytes
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Wager {
    pub contract: VersusContract,
    pub wallet_a_decision: ApprovalState,   // 1 byte
    pub wallet_b_decision: ApprovalState,   // 1 byte
    pub belief_a: u8,                       // 1 byte (+1 Option)
    pub belief_b: u8,                       // 1 byte (+1 Option)
    pub paid_a: bool,                       // 1 byte
    pub paid_b: bool,                       // 1 byte
}

// Riverboat parlor for two competing predictions
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VersusContract {
    pub terms: String,      // 4 + length
    pub wallet_a: Pubkey,   // 32 bytes
    pub wallet_b: Pubkey,   // 32 bytes
    pub stake: u64,         // 8 bytes
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum ApprovalState {
    Pending,
    Landed,
    Missed,
    Push
}