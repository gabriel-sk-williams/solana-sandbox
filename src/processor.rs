// processor.rs 

use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{ApprovalState, VersusContract, Wager};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke,
    program::invoke_signed,
    sysvar::{rent::Rent, Sysvar},
    hash::hash,
    system_program,
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

    // Print the SOL balance (in lamports)
    msg!("wager_account lamports: {}", wager_account.lamports());

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
        decision_a: ApprovalState::Pending,
        decision_b: ApprovalState::Pending,
        belief_a: 255,
        belief_b: 255,
        paid_a: false,
        paid_b: false,
    };

    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;
    
    msg!("Wager stored successfully!");
    Ok(())
}

pub fn process_deposit(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    msg!("Processing Deposit...");
    
    // Get accounts
    let wager_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    
    // Verify accounts
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    /*
    if user_account.key != &user {
        return Err(ProgramError::InvalidArgument);
    }
    */
    
    if *system_program.key != system_program::ID {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Get dual space data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;
    
    // Verify stake amount
    if amount < wager.contract.stake {
        return Err(ProgramError::InsufficientFunds);
    }
    
    // Transfer funds from user to the program account
    invoke(
        &system_instruction::transfer(
            user_account.key,
            wager_account.key,
            amount,
        ),
        &[
            user_account.clone(),
            wager_account.clone(),
            system_program.clone(),
        ],
    )?;
    
    // Update payment status
    if user_account.key == &wager.contract.wallet_a {
        wager.paid_a = true;
    } else if user_account.key == &wager.contract.wallet_b {
        wager.paid_b = true;
    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize and save the updated state
    wager.serialize(&mut *wager_account.data.borrow_mut())?;
    
    Ok(())
}

pub fn update_belief(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    belief: u8,
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
    
    // Verify signer is an authorized wallet
    if signer.key == &wager.contract.wallet_a {

        // Verify wallet has paid stake
        if !wager.paid_a {
            return Err(ProgramError::Immutable);
        } else {
            wager.belief_a = belief;
        }

    } else if signer.key == &wager.contract.wallet_b {

        // Verify wallet has paid stake
        if !wager.paid_b {
            return Err(ProgramError::Immutable);
        } else {
            wager.belief_b = belief;
        }

    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize the updated data back to the account
    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    msg!("Belief Updated!");

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
        wager.decision_a = decision;
    } else if signer.key == &wager.contract.wallet_b {
        wager.decision_b = decision;
    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize the updated data back to the account
    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    
    // Check if we need to execute payout logic
    if wager.decision_a == ApprovalState::Landed && 
       wager.decision_b == ApprovalState::Landed {
        // Execute payout logic
        msg!("Wager Landed!")
    }

    // Check if we need to execute payout logic
    if wager.decision_a == ApprovalState::Missed && 
       wager.decision_b == ApprovalState::Missed {
        // Execute payout logic
        msg!("Wager Missed!")
    }

    Ok(())
}
