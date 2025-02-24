#![allow(unexpected_cfgs)]

use solana_program::{
    account_info::AccountInfo, 
    entrypoint, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
    program_error::ProgramError,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    // Convert the instruction data (bytes) into a string
    let message = String::from_utf8(instruction_data.to_vec())
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Print the user's message
    msg!("User's message: {}", message);

    Ok(())
}

#[cfg(test)]
mod test;