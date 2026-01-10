// processor.rs 

use std::cmp;

use borsh::{BorshSerialize, BorshDeserialize};

use crate::state::{Wager, Seat, Status, Judgment};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError, // TODO: Use custom error instead of ProgramError::Immutable
    pubkey::Pubkey,
    program::invoke,
    program::invoke_signed,
    sysvar::{rent::Rent, Sysvar},
    // hash::hash,
    system_program,
    system_instruction,
    msg,
    clock::Clock,
};

pub fn create_wager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wager: Wager,
    reserved_seats: Vec<Pubkey>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let mut seat_accounts = Vec::new();
    for _ in 0..reserved_seats.len() {
        seat_accounts.push(next_account_info(accounts_iter)?);
    }

    // Verify account ownership and signing
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !wager_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // create vault
    let vault_related_accounts = &[
        payer.clone(),
        wager_account.clone(),
        vault_account.clone(),
        system_program.clone(),
    ];

    create_vault(
        program_id,
        vault_related_accounts,
    )?;

    // create seat
    for (index, &seat_authority) in reserved_seats.iter().enumerate() {
        let seat_related_accounts = &[
            payer.clone(),
            wager_account.clone(),
            seat_accounts[index].clone(),
            system_program.clone(),
        ];

        create_seat(
            program_id,
            seat_related_accounts,
            &seat_authority,
            index as u8
        )?;
    }

    // Create wager account
    let rent = Rent::get()?;
    let space = Wager::SPACE;
    let required_lamports = rent.minimum_balance(space);

    invoke(
        &system_instruction::create_account(
            payer.key,
            wager_account.key,
            required_lamports,
            space as u64,
            program_id,
        ),
        &[
            payer.clone(), 
            wager_account.clone(), 
            system_program.clone(),
        ],
    )?;

    wager.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;
    
    msg!("Wager stored successfully!");

    Ok(())
}

pub fn create_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(0); // No data, just enough for rent exemption

    // create vault PDA
    let (pda, bump) = Pubkey::find_program_address(
        &[b"vault", wager_account.key.as_ref()],
        program_id
    );

    if pda != *vault_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    // let vault_seeds = &[b"vault", wager_key.as_ref(), &[vault_bump]];

    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            vault_account.key,
            lamports,
            0, // vault holds no data
            &system_program::ID, // formerly program_id ???
        ),
        &[
            payer.clone(), 
            vault_account.clone(), 
            system_program.clone()],
        &[&[
            b"vault", 
            wager_account.key.as_ref(), 
            &[bump]
        ]],
    )?;

    msg!("Vault created successfully!");
    Ok(())
}

pub fn create_seat(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    authority: &Pubkey,
    index: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;
    let seat_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let rent = Rent::get()?;
    let space = Seat::SPACE;
    let required_lamports = rent.minimum_balance(space);

    // create seat PDA
    let (pda, bump) = Pubkey::find_program_address(
        &[b"seat", wager_account.key.as_ref(), &index.to_le_bytes()],
        program_id
    );

    if pda != *seat_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    let seat = Seat::take(*wager_account.key, *authority, current_timestamp);

    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            seat_account.key,
            required_lamports,
            space as u64,
            program_id, // &system_program::ID,
        ),
        &[
            payer.clone(),
            seat_account.clone(),
            system_program.clone(),
        ],
        &[&[
            b"seat",
            wager_account.key.as_ref(),
            &index.to_le_bytes(),
            &[bump],
        ]],
    )?;

    seat.serialize(&mut &mut seat_account.data.borrow_mut()[..])?;
    msg!("Seat created successfully!");
    Ok(())
}

/*
pub fn process_deposit(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    
    // Verify accounts
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    if *system_program.key != system_program::ID {
        return Err(ProgramError::InvalidArgument);
    }
    
    // Get wager data
    let mut wager = Wager::try_from_slice(&wager_account.data.borrow())?;
    
    // Verify stake amount
    if amount < wager.stake {
        return Err(ProgramError::InsufficientFunds);
    }

    /*
    // TODO: rework seat logic
    if versus.seat_a.wallet == system_program::ID {
        versus.seat_a.wallet = *payer.key;
        msg!("Seat A claimed by: {}", payer.key);
    } else if versus.seat_b.wallet == system_program::ID {
        versus.seat_b.wallet = *payer.key;
        msg!("Seat B claimed by: {}", payer.key);
    } else {
        msg!("Both seats already taken!");
        return Err(ProgramError::InvalidAccountData);
    }

    // Find matching seat
    let seat = if payer.key == &versus.seat_a.wallet {
        &mut versus.seat_a
    } else if payer.key == &versus.seat_b.wallet {
        &mut versus.seat_b
    } else {
        msg!("no matching seat");
        return Err(ProgramError::InvalidArgument);
    };
     */

    // Verify wallet has not yet paid stake
    if seat.status != Status::Open {
        return Err(ProgramError::Immutable);
    }
    
    // Transfer funds from user to program account
    invoke(
        &system_instruction::transfer(
            payer.key,
            vault_account.key,
            amount,
        ),
        &[
            payer.clone(),
            vault_account.clone(),
            system_program.clone(),
        ],
    )?;

    seat.status = Status::Staked;
    
    // Serialize and save updated state
    versus.serialize(&mut *wager_account.data.borrow_mut())?;
    
    Ok(())
}

pub fn update_belief(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    belief: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let signer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;
    
    // Verify account ownership
    if wager_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify signer
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Deserialize account data
    let mut versus = Wager::try_from_slice(&wager_account.data.borrow())?;

    // Find matching seat
    let seat = if signer.key == &versus.seat_a.wallet {
        &mut versus.seat_a
    } else if signer.key == &versus.seat_b.wallet {
        &mut versus.seat_b
    } else {
        return Err(ProgramError::InvalidArgument);
    };

    // Verify wallet has paid stake before setting belief
    if seat.status != Status::Staked {
        return Err(ProgramError::Immutable);
    } else {
        seat.belief = belief;
    }

    // Serialize updated data back to account
    versus.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    msg!("Belief Updated!");

    Ok(())
}

pub fn lock_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let signer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;

    // Verify account ownership
    if wager_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Verify signer
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize account data
    let mut versus = Wager::try_from_slice(&wager_account.data.borrow())?;

    // Find matching seat
    let seat = if signer.key == &versus.seat_a.wallet {
        &mut versus.seat_a
    } else if signer.key == &versus.seat_b.wallet {
        &mut versus.seat_b
    } else {
        return Err(ProgramError::InvalidArgument);
    };

    seat.status = Status::Locked;

    // Serialize updated data back to account
    versus.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn set_judgment(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    judgment: Judgment,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let signer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;

    // Verify account ownership
    if wager_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify signer
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Deserialize account data
    let mut versus = Wager::try_from_slice(&wager_account.data.borrow())?;

    // Find matching seat
    let seat = if signer.key == &versus.seat_a.wallet {
        &mut versus.seat_a
    } else if signer.key == &versus.seat_b.wallet {
        &mut versus.seat_b
    } else {
        return Err(ProgramError::InvalidArgument);
    };

    seat.judgment = judgment;
    
    // Serialize updated data back to account
    versus.serialize(&mut &mut wager_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn render_payout(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let signer = next_account_info(accounts_iter)?;
    let wager_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify account ownership
    if wager_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Verify signer
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if vault_account.owner != &system_program::ID {
        msg!("Vault owner is wrong!");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize account data
    let wager = Wager::try_from_slice(&wager_account.data.borrow())?;

    let payouts = calc_risk(wager.stake, seat.belief as u64, seat.belief as u64);
    let (risk_a, risk_b) = payouts;

    /*
    // Find matching wallet
    let player_wallet = if signer.key == &versus.seat_a.wallet {
        &versus.seat_a.wallet
    } else if signer.key == &versus.seat_b.wallet {
        &versus.seat_b.wallet
    } else {
        return Err(ProgramError::InvalidArgument);
    };
    */
    
    // Determine if players agree on wager outcome
    let _judgment = if 
        versus.seat_a.judgment == Judgment::Landed &&  
        versus.seat_b.judgment == Judgment::Landed {
        msg!("Players agree: Wager Landed!");
        Judgment::Landed
    } else if
        versus.seat_a.judgment == Judgment::Missed && 
        versus.seat_b.judgment == Judgment::Missed {
        msg!("Players agree: Wager Missed!");
        Judgment::Missed
    } else if 
        versus.seat_a.judgment == Judgment::Push && 
        versus.seat_b.judgment == Judgment::Push {
        msg!("Players agree to push");
        Judgment::Push
    } else {
        return Err(ProgramError::InvalidAccountData)
    };

    msg!("Player {} {} {}", player_wallet, risk_a, risk_b);

    invoke_signed(
        &system_instruction::transfer(
            vault_account.key,
            signer.key,
            versus.wager.stake,
        ),
        &[
            vault_account.clone(),
            signer.clone(),
            system_program.clone(),
        ],
        &[&[b"vault", wager_account.key.as_ref(), &[versus.wager.vault_bump]]]
    )?;

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

*/