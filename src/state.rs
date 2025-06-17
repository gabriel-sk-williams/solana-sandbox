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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Wager {
    pub contract: VersusContract,
    pub status_a: PayoutStatus,      // 1 byte
    pub status_b: PayoutStatus,      // 1 byte
    pub belief_a: u8,                // 1 byte
    pub belief_b: u8,                // 1 byte
    pub decision_a: ApprovalState,   // 1 byte
    pub decision_b: ApprovalState,   // 1 byte
}

// Payout states for a participant
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum PayoutStatus {
    NotStaked,
    Staked,
    Locked,
    ClaimedPartial,
    Settled
}

// Wager states, decided by participants
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum ApprovalState {
    Pending,
    Landed,
    Missed,
    Push
}




