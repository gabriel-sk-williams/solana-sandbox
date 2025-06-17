// processor.rs 

use std::cmp;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{PayoutStatus, ApprovalState, VersusContract, Wager};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError, // TODO: Use custom error instead of ProgramError::Immutable
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

    // Print SOL balance (in lamports)
    msg!("wager_account lamports: {}", wager_account.lamports());

    // Deserialize wager
    let data = &wager_account.data.borrow_mut();
    let wager_data = Wager::try_from_slice(&data);
    msg!("result {:?}", wager_data);

    Ok(())
}

pub fn create_wager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    contract: VersusContract,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let wager_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;


    // Verify account ownership and signing
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Create wager account
    let rent = Rent::get()?;
    let terms_allocation = 4 + contract.terms.len();
    let contract_allocation = terms_allocation + 32 + 32 + 8;
    let wager_allocation = contract_allocation + 2 + 2 + 2;
    let required_lamports = rent.minimum_balance(wager_allocation);

    // Derive PDA
    let terms_hash = hash(contract.terms.as_bytes()).to_bytes();
    let (wager_pda, wager_bump) = Pubkey::find_program_address(
        &[
            &terms_hash[..],
            contract.wallet_a.as_ref(),
            contract.wallet_b.as_ref(),
        ], 
        program_id
    );

    //msg!("wager: {:?} {:?}", wager_pda, wager_bump);

    if wager_pda != *wager_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"vault", wager_pda.as_ref()],
        program_id
    );

    //msg!("vault: {:?} {:?}", vault_pda, vault_bump);

    if vault_pda != *vault_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let vault_related_accounts = &[
        user_account.clone(),
        vault_account.clone(),
        system_program.clone(),
    ];

    create_vault(
        program_id,
        vault_related_accounts,
        &wager_pda,
        vault_bump
    )?;

    // Create wager account with PDA
    invoke_signed(
        &system_instruction::create_account(
            user_account.key,
            wager_account.key,
            required_lamports,
            wager_allocation as u64,
            program_id,
        ),
        &[
            user_account.clone(), 
            wager_account.clone(), 
            system_program.clone(),
        ],
        &[&[
            &terms_hash[..],
            contract.wallet_a.as_ref(),
            contract.wallet_b.as_ref(),
            &[wager_bump]
        ]],
    )?;

    let wager = Wager {
        contract: contract,
        status_a: PayoutStatus::NotStaked,
        status_b: PayoutStatus::NotStaked,
        belief_a: 255,
        belief_b: 255,
        decision_a: ApprovalState::Pending,
        decision_b: ApprovalState::Pending,
    };

    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;
    
    msg!("Wager stored successfully!");

    Ok(())
}

pub fn create_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wager_pda: &Pubkey,
    vault_bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let user_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(0); // No data, just enough for rent exemption

    let vault_seeds = &[b"vault", wager_pda.as_ref(), &[vault_bump]];

    invoke_signed(
        &system_instruction::create_account(
            user_account.key,
            vault_account.key,
            lamports,
            0, // vault holds no data
            program_id,
        ),
        &[user_account.clone(), vault_account.clone(), system_program.clone()],
        &[vault_seeds],
    )?;

    msg!("Vault created successfully!");
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
    let vault_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    
    // Verify accounts
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    if *system_program.key != system_program::ID {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Get wager data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;
    
    // Verify stake amount
    if amount < wager.contract.stake {
        return Err(ProgramError::InsufficientFunds);
    }
    
    // Transfer funds from user to program account
    invoke(
        &system_instruction::transfer(
            user_account.key,
            vault_account.key,
            amount,
        ),
        &[
            user_account.clone(),
            vault_account.clone(),
            system_program.clone(),
        ],
    )?;
    
    // Update payment status
    if user_account.key == &wager.contract.wallet_a {

        // Verify wallet has not yet paid stake
        if wager.status_a != PayoutStatus::NotStaked {
            return Err(ProgramError::Immutable);
        }
        wager.status_a = PayoutStatus::Staked;

    } else if user_account.key == &wager.contract.wallet_b {

        // Verify wallet has not yet paid stake
        if wager.status_b != PayoutStatus::NotStaked {
            return Err(ProgramError::Immutable);
        }
        wager.status_b = PayoutStatus::Staked;

    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize and save updated state
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
    
    // Deserialize account data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;
    
    // Verify signer is an authorized wallet
    if signer.key == &wager.contract.wallet_a {

        // Verify wallet has paid stake
        if wager.status_a != PayoutStatus::Staked {
            return Err(ProgramError::Immutable);
        } else {
            wager.belief_a = belief;
        }

    } else if signer.key == &wager.contract.wallet_b {

        // Verify wallet has paid stake
        if wager.status_b != PayoutStatus::Staked {
            return Err(ProgramError::Immutable);
        } else {
            wager.belief_b = belief;
        }

    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize updated data back to account
    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    msg!("Belief Updated!");

    Ok(())
}

pub fn lock_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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

    // Deserialize account data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;

    // Verify signer is an authorized wallet and update appropriate approval
    if signer.key == &wager.contract.wallet_a {
        wager.status_a = PayoutStatus::Locked;
    } else if signer.key == &wager.contract.wallet_b {
        wager.status_b = PayoutStatus::Locked;
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    // Serialize updated data back to account
    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

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
    
    // Deserialize account data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;
    
    // Verify signer is an authorized wallet and update appropriate approval
    if signer.key == &wager.contract.wallet_a {
        wager.decision_a = decision;
    } else if signer.key == &wager.contract.wallet_b {
        wager.decision_b = decision;
    } else {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Serialize updated data back to account
    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn render_payouts(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {

    msg!("Rendering Payouts!");

    let accounts_iter = &mut accounts.iter();
    let wager_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify account ownership
    if wager_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Verify signer
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize account data
    let wager = Wager::try_from_slice(&wager_account.data.borrow())?;

    let is_player_a = user_account.key == &wager.contract.wallet_a;
    let _is_player_b = user_account.key == &wager.contract.wallet_b;

    // Check if we need to execute payout logic
    if wager.decision_a == ApprovalState::Landed && 
       wager.decision_b == ApprovalState::Landed {
        
        // Execute payout logic
        let payouts = calc_risk(wager.contract.stake, wager.belief_a as u64, wager.belief_b as u64);
        let (payout_a, _payout_b) = payouts;

        msg!("rendering {:?}", payout_a);
        
        // Transfer payouts
        if is_player_a && payout_a > 0 {
            invoke(
                &system_instruction::transfer(
                    user_account.key,
                    vault_account.key,
                    payout_a,
                ),
                &[
                    user_account.clone(),
                    vault_account.clone(),
                    system_program.clone(),
                ],
            )?;
            
        }

        msg!("Wager Landed!")
    }

    /*
    // Check if we need to execute payout logic
    if wager.decision_a == ApprovalState::Missed && 
       wager.decision_b == ApprovalState::Missed {

        // Execute payout logic
        //let payouts = calc_risk(wager.contract.stake, wager.belief_a as u64, wager.belief_b as u64);
        //let (payout_a, payout_b) = payouts;

        msg!("Wager Missed!")
    }

    // Check if we need to execute payout logic
    if wager.decision_a == ApprovalState::Push && 
       wager.decision_b == ApprovalState::Push {

        // Return original stakes to both players

        msg!("Wager Pushed!")
    }
    */

    Ok(())
}

fn calc_risk(stake: u64, belief_a: u64, belief_b: u64) -> (u64, u64) {

    if belief_a == belief_b { return (0, 0) };

    let p = cmp::max(belief_a, belief_b);
    let q = 100 - cmp::min(belief_a, belief_b);

    // max 10000
    let p_sqd = p * p;
    let q_sqd = q * q;

    let p_surprise = 100 - p;
    let q_surprise = 100 - q;

    // max 10000
    let p_surprise_sqd = p_surprise * p_surprise;
    let q_surprise_sqd = q_surprise * q_surprise;

    let portion_a = p_sqd - q_surprise_sqd;
    let portion_b = q_sqd - p_surprise_sqd;

    let scale = stake / 10_000; // 100^2

    let risk_a = scale * portion_a;
    let risk_b = scale * portion_b;

    return (risk_a, risk_b);
}