// state.rs

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    pubkey::Pubkey,
};

// Contract for two competing predictions
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VersusContract {
    pub terms: String,      // 4 + length
    pub wallet_a: Pubkey,   // 32 bytes
    pub wallet_b: Pubkey,   // 32 bytes
    pub stake: u64,         // 8 bytes
}

/*
INIT = 0;
DEPOSIT_PAID= 1;
BELIEF_UPDATED = 2;
STATUS_LOCKED = 3;
APPROVAL_SET = 4;
PAYOUT_RENDERED = 5;
*/

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Wager {
    pub contract: VersusContract,
    pub paid_a: bool,                // 1 byte
    pub paid_b: bool,                // 1 byte
    pub belief_a: u8,                // 1 byte
    pub belief_b: u8,                // 1 byte
    pub locked_a: bool,              // 1 byte
    pub locked_b: bool,              // 1 byte
    pub decision_a: ApprovalState,   // 1 byte
    pub decision_b: ApprovalState,   // 1 byte
    pub payouts_rendered: bool,      // 1 byte
}

// Possible Wager states for each participant
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum ApprovalState {
    Pending,
    Landed,
    Missed,
    Push
}


