#![allow(unexpected_cfgs)]

pub mod instruction;
pub mod state;
pub mod processor;

use instruction::WagerInstruction;
use processor::{
    create_wager, 
    process_deposit,
    update_belief,
    lock_status,
    set_judgment,
    render_payout,
};

use solana_program::{
    account_info::{AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    entrypoint,
    msg,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    // Unpack instruction data
    let instruction = WagerInstruction::unpack(instruction_data)?;
    msg!("instruct {:?}", instruction);

    match instruction {
        WagerInstruction::CreateWager { wager } => {
            create_wager(program_id, accounts, wager)
        }
        WagerInstruction::ProcessDeposit { amount } => {
            process_deposit(program_id, accounts, amount)
        }
        WagerInstruction::UpdateBelief { belief } => {
            update_belief(program_id, accounts, belief)
        }
        WagerInstruction::LockStatus => {
            lock_status(program_id, accounts)
        }
        WagerInstruction::SetJudgment { judgment } => {
            set_judgment(program_id, accounts, judgment)
        }
        WagerInstruction::RenderPayouts => {
            render_payout(program_id, accounts)
        }
    }
}