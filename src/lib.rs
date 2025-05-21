#![allow(unexpected_cfgs)]

pub mod instruction;
pub mod state;
pub mod processor;

use instruction::WagerInstruction;
use processor::{get_wager, create_wager, set_approval};

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
        WagerInstruction::GetWager => {
            get_wager(program_id, accounts)
        }
        WagerInstruction::CreateWager { contract } => {
            create_wager(program_id, accounts, contract)
        }
        WagerInstruction::SetApproval { decision } => {
            set_approval(program_id, accounts, decision)
        }
    }
}


#[cfg(test)]
mod test;