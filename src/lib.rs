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
    // program::invoke,
    program::invoke_signed,
    // system_program,
    sysvar::{rent::Rent, Sysvar},
};

// Riverboat space for two competing predictions
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DualSpace {
    pub terms: String,      // 4 + length
    pub wallet_a: Pubkey,   // 32 bytes
    pub belief_a: f64,      // 8 bytes
    pub wallet_b: Pubkey,   // 32 bytes
    pub belief_b: f64,      // 8 bytes
}

// Define instruction types
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SpaceInstruction {
    CreateSpace { space: DualSpace },
    GetSpace,
}

impl SpaceInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Get the instruction variant from the first byte
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        msg!("input {:?}", input);
 
        // Match instruction type and parse the remaining bytes based on the variant
        match variant {
            0 => {
                let dual_space = DualSpace::try_from_slice(
                    &rest).map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::CreateSpace { space: dual_space })
            }
            1 => { // No additional data needed
                Ok(Self::GetSpace)
            }
            _ => {
                Err(ProgramError::InvalidInstructionData)
            }
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
    let instruction = SpaceInstruction::unpack(instruction_data)?;
    msg!("instruct {:?}", instruction);

    match instruction {
        SpaceInstruction::CreateSpace { space } => {
            create_space(program_id, accounts, space)
        }
        SpaceInstruction::GetSpace => {
            get_space(program_id, accounts)
        }
    }
}

fn create_space(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    space: DualSpace,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let space_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify account ownership and signing
    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Create the message account
    let rent = Rent::get()?;

    let space_allocation = 32 + 32 + 8 + 8 + 4 + space.terms.len(); // Allocate fixed space for the message

    let required_lamports = rent.minimum_balance(space_allocation);

    // Derive PDA
    let space_seed = b"gerben";
    // let (space_pda, bump) = Pubkey::find_program_address(&[space_seed, user.key.as_ref()], program_id);
    let (space_pda, bump) = Pubkey::find_program_address(&[space_seed], program_id);

    if space_pda != *space_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    // Create account with the program as owner
    invoke_signed(
        &system_instruction::create_account(
            user.key,
            space_account.key,
            required_lamports,
            space_allocation as u64,
            program_id,
        ),
        &[
            user.clone(), 
            space_account.clone(), 
            system_program.clone(),
        ],
        &[&[b"gerben", &[bump]]],
        // &[&[b"gerben", user.key.as_ref(), &[bump]]],
    )?;

    space.serialize(&mut &mut space_account.data.borrow_mut()[..])?;
    
    msg!("Space stored successfully!");
    Ok(())
}

fn get_space(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("id {:?}", program_id);

    let accounts_iter = &mut accounts.iter();
    let space_account = next_account_info(accounts_iter)?;

    // Deserialize the message
    let data = &space_account.data.borrow_mut();
    let message_data = DualSpace::try_from_slice(&data);
    msg!("result {:?}", message_data);

    Ok(())
}

#[cfg(test)]
mod test;