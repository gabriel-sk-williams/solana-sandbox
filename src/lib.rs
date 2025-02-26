#![allow(unexpected_cfgs)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::invoke, // , invoke_signed
    // system_program,
    sysvar::{rent::Rent, Sysvar},
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MessageAccount {
    pub message: String,
}

// Define instruction types
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MessageInstruction {
    CreateMessage { message: String },
    ReadMessage,
    ParseFloat { float: f64 }, 
}

// working towards this
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DualSpace {
    pub terms: String,      // 4 + length
    pub wallet_a: Pubkey,   // 32 bytes
    pub belief_a: f64,      // 8 bytes
    pub wallet_b: Pubkey,   // 32 bytes
    pub belief_b: f64,      // 8 bytes
}

impl MessageInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Get the instruction variant from the first byte
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        msg!("input {:?}", input);
 
        // Match instruction type and parse the remaining bytes based on the variant
        match variant {
            0 => {
                // Parse String
                let message = String::try_from_slice(
                    &rest).map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::CreateMessage { message })
            }
            1 => { // No additional data needed
                Ok(Self::ReadMessage)
            }
            2 => {
                // Parse f64
                let float = f64::from_le_bytes(
                    rest.try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?
                );
                Ok(Self::ParseFloat { float })
            } 
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    // Unpack instruction data
    let instruction = MessageInstruction::unpack(instruction_data)?;

    match instruction {
        MessageInstruction::CreateMessage { message } => {
            create_message(program_id, accounts, message)
        }
        MessageInstruction::ReadMessage => {
            read_message(program_id, accounts)
        }
        MessageInstruction::ParseFloat { float } => {
            parse_float(float)
        }
    }
}

fn parse_float(float: f64) -> ProgramResult {
    msg!("We got that mfin float: {:?}", float);
    Ok(())
}

fn create_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    message: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let message_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify account ownership and signing
    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Create the message account
    let rent = Rent::get()?;
    let space = 4 + message.len(); // Allocate fixed space for the message
    let required_lamports = rent.minimum_balance(space);

    // Create account with the program as owner
    invoke(
        &system_instruction::create_account(
            user.key,
            message_account.key,
            required_lamports,
            space as u64,
            program_id,
        ),
        &[
            user.clone(), 
            message_account.clone(), 
            system_program.clone()
        ],
    )?;

    // Store the message
    let message_data = MessageAccount { message };

    // attempt
    let mut account_data = &mut message_account.data.borrow_mut()[..];
    message_data.serialize(&mut account_data)?;
    
    msg!("Message stored successfully!");
    Ok(())
}

fn read_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("id {:?}", program_id);

    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;

    // Deserialize the message
    let data = &message_account.data.borrow_mut();
    let message_data = MessageAccount::try_from_slice(&data);
    msg!("result {:?}", message_data);

    Ok(())
}

#[cfg(test)]
mod test;