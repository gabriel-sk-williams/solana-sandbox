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
    hash::hash,
};

// Riverboat parlor for two competing predictions
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DualSpace {
    pub terms: String,      // 4 + length
    pub wallet_a: Pubkey,   // 32 bytes
    pub wallet_b: Pubkey,   // 32 bytes
    pub belief_a: f64,      // 8 bytes
    pub belief_b: f64,      // 8 bytes
    // pub stake: f64,         // 8 bytes
}

// Define instruction types
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SpaceInstruction {
    CreateSpace { space: DualSpace },
    GetSpace,
    SetApproval { decision: ApprovalState },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum ApprovalState {
    Pending,
    Landed,
    Missed,
    Push
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Wager {
    pub parlor: DualSpace,
    pub wallet_a_decision: ApprovalState, // 1 byte
    pub wallet_b_decision:  ApprovalState, // 1 byte
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
            2 => {
                let decision_byte: u8 = 1;

                let decision = match decision_byte {
                    0 => ApprovalState::Pending,
                    1 => ApprovalState::Landed,
                    2 => ApprovalState::Missed,
                    3 => ApprovalState::Push,
                    _ => return Err(ProgramError::InvalidInstructionData),
                };

                Ok(Self::SetApproval { decision })
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
        SpaceInstruction::SetApproval { decision } => {
            set_approval(program_id, accounts, decision)
        }
    }
}

fn create_space(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    dual_space: DualSpace,
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
    let space_allocation = 32 + 32 + 8 + 8 + 4 + 1 + 1 + dual_space.terms.len(); // Allocate fixed space for the message
    let required_lamports = rent.minimum_balance(space_allocation);

    // Derive PDA
    let terms_hash = hash(dual_space.terms.as_bytes()).to_bytes();
    let (space_pda, bump) = Pubkey::find_program_address(
        &[
            &terms_hash[..],
            dual_space.wallet_a.as_ref(),
            dual_space.wallet_b.as_ref(),
        ], 
        program_id
    );

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
        &[&[
            &terms_hash[..],
            dual_space.wallet_a.as_ref(),
            dual_space.wallet_b.as_ref(),
            &[bump]
        ]],
    )?;

    let wager = Wager {
        parlor: dual_space,
        wallet_a_decision: ApprovalState::Pending,
        wallet_b_decision:  ApprovalState::Pending,
    };

    wager.serialize(&mut &mut space_account.data.borrow_mut()[..])?;
    
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
    let message_data = Wager::try_from_slice(&data);
    msg!("result {:?}", message_data);

    Ok(())
}

fn set_approval(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    decision: ApprovalState,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let space_account = next_account_info(accounts_iter)?;
    let signer = next_account_info(accounts_iter)?;

    // Verify account ownership
    if space_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify signer
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Deserialize the account data
    let mut wager = Wager::try_from_slice(&space_account.data.borrow())?;
    
    
    // Verify signer is an authorized wallet and update the appropriate approval
    if signer.key == &wager.parlor.wallet_a {
        wager.wallet_a_decision = decision;
    } else if signer.key == &wager.parlor.wallet_b {
        wager.wallet_b_decision = decision;
    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize the updated data back to the account
    wager.serialize(&mut &mut space_account.data.borrow_mut()[..])?;

    
    // Check if we need to execute payout logic
    if wager.wallet_a_decision == ApprovalState::Landed && 
       wager.wallet_b_decision == ApprovalState::Landed {
        // Execute payout logic
        msg!("Landed")
    }

    Ok(())
}

#[cfg(test)]
mod test;