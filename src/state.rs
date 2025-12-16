// state.rs

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    pubkey::Pubkey,
    system_program::ID,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VersusWager {
    pub wager: Wager,               // 73 bytes
    pub seat_a: Seat,               // 43 bytes
    pub seat_b: Seat,               // 43 bytes
}

impl VersusWager {
    pub const SPACE: usize = 8 + 32 + 32 + 1 + 43 + 43;
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Wager {
    pub stake: u64,                 // 8 bytes
    pub contract: Pubkey,           // 32 bytes
    pub vault: Pubkey,              // 32 bytes
    pub vault_bump: u8,             // 1 byte
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Seat {
    pub wallet: Pubkey,             // 32 bytes
    pub belief: u8,                 // 1 byte
    pub status: Status,             // 1 byte
    pub judgment: Judgment,         // 1 byte
    pub last_change_at: i64,        // 8 bytes
}

impl Seat {
    pub fn open(timestamp: i64) -> Self {
        Seat {
            wallet: ID,
            belief: 255,
            status: Status::Open,
            judgment: Judgment::Pending,
            last_change_at: timestamp
        }
    }

    pub fn reserved(wallet: Pubkey, timestamp: i64) -> Self {
        Seat {
            wallet: wallet,
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

// Ouctome, decided by participants
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum Judgment {
    Pending,
    Landed,
    Missed,
    Push,
}



