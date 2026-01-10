// state.rs

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    pubkey::Pubkey,
    // system_program::ID,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Wager {
    pub contract: Pubkey,           // 32 bytes
    pub vault: Pubkey,              // 32 bytes
    pub vault_bump: u8,             // 1 byte
    pub seat_count: u8,             // 1 byte
    pub capacity: u8,               // 1 byte
    pub stake: u64,                 // 8 bytes
}

impl Wager {
    pub const SPACE: usize = 32 + 32 + 1 + 1 + 1 + 8;
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Seat {
    pub wager: Pubkey,              // 32 bytes
    pub authority: Pubkey,          // 32 bytes
    pub belief: u8,                 // 1 byte
    pub status: Status,             // 1 byte
    pub judgment: Judgment,         // 1 byte
    pub last_change_at: i64,        // 8 bytes
}

impl Seat {
    pub const SPACE: usize = 32 + 32 + 1 + 1 + 1 +8;

    pub fn take(wager: Pubkey, authority: Pubkey, timestamp: i64) -> Self {
        Seat {
            wager: wager,
            authority: authority,
            belief: 255,
            status: Status::Open,
            judgment: Judgment::Pending,
            last_change_at: timestamp
        }
    }
}

// Game states for a given seat or participant
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum Status {
    Open,
    Staked,
    Locked,
}

// Outcome, decided by participants
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum Judgment {
    Pending,
    Landed,
    Missed,
    Push,
}

/*
pub fn open(timestamp: i64) -> Self {
    Seat {
        authority: ID,
        belief: 255,
        status: Status::Open,
        judgment: Judgment::Pending,
        last_change_at: timestamp
    }
}
*/

