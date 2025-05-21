// processor.rs 

use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{Wager, VersusContract, ApprovalState};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed,
    sysvar::{rent::Rent, Sysvar},
    hash::hash,
    system_instruction,
    msg,
};

pub fn get_wager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("id {:?}", program_id);

    let accounts_iter = &mut accounts.iter();
    let wager_account = next_account_info(accounts_iter)?;

    // Deserialize the message
    let data = &wager_account.data.borrow_mut();
    let message_data = Wager::try_from_slice(&data);
    msg!("result {:?}", message_data);

    Ok(())
}

pub fn create_wager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    contract: VersusContract,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let wager_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify account ownership and signing
    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Create the message account
    let rent = Rent::get()?;
    let terms_allocation = 4 + contract.terms.len();
    let contract_allocation = terms_allocation + 32 + 32 + 8;
    let wager_allocation = contract_allocation + 2 + 2 + 2;
    let required_lamports = rent.minimum_balance(wager_allocation);

    // Derive PDA
    let terms_hash = hash(contract.terms.as_bytes()).to_bytes();
    let (wager_pda, bump) = Pubkey::find_program_address(
        &[
            &terms_hash[..],
            contract.wallet_a.as_ref(),
            contract.wallet_b.as_ref(),
        ], 
        program_id
    );

    if wager_pda != *wager_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    // Create account with the program as owner
    invoke_signed(
        &system_instruction::create_account(
            user.key,
            wager_account.key,
            required_lamports,
            wager_allocation as u64,
            program_id,
        ),
        &[
            user.clone(), 
            wager_account.clone(), 
            system_program.clone(),
        ],
        &[&[
            &terms_hash[..],
            contract.wallet_a.as_ref(),
            contract.wallet_b.as_ref(),
            &[bump]
        ]],
    )?;

    let wager = Wager {
        contract: contract,
        wallet_a_decision: ApprovalState::Pending,
        wallet_b_decision: ApprovalState::Pending,
        belief_a: 0,
        belief_b: 0,
        paid_a: false,
        paid_b: false,
    };

    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;
    
    msg!("Wager stored successfully!");
    Ok(())
}

pub fn set_approval(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    decision: ApprovalState,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let wager_account = next_account_info(accounts_iter)?;
    let signer = next_account_info(accounts_iter)?;

    // Verify account ownership
    if wager_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify signer
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Deserialize the account data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;
    
    // Verify signer is an authorized wallet and update the appropriate approval
    if signer.key == &wager.contract.wallet_a {
        wager.wallet_a_decision = decision;
    } else if signer.key == &wager.contract.wallet_b {
        wager.wallet_b_decision = decision;
    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize the updated data back to the account
    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    
    // Check if we need to execute payout logic
    if wager.wallet_a_decision == ApprovalState::Landed && 
       wager.wallet_b_decision == ApprovalState::Landed {
        // Execute payout logic
        msg!("Wager Landed!")
    }

    // Check if we need to execute payout logic
    if wager.wallet_a_decision == ApprovalState::Missed && 
       wager.wallet_b_decision == ApprovalState::Missed {
        // Execute payout logic
        msg!("Wager Missed!")
    }

    Ok(())
}
