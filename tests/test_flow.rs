use borsh::{BorshDeserialize, to_vec}; // BorshSerialize

use solana_god::{process_instruction};
use solana_god::instruction::{WagerInstruction};
use solana_god::state::{Wager, Seat, Judgment};

use solana_program::{
    pubkey::Pubkey,
    system_instruction,
    system_program,
    // hash::hash,
    // solana_system_interface,
};

// use solana_system_interface::instruction;
// use solana_sdk_ids::system_program;

use solana_program_test::*;

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Signer,
    transaction::Transaction,
    signer::keypair::Keypair
};

#[tokio::test]
async fn wager_flow() {

    //
    // Create Program Test
    //

    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = 
        ProgramTest::new("solana_god", program_id, processor!(process_instruction))
            .start()
            .await;

    let contract_pubkey = Pubkey::new_unique();

    let wallet_a = Keypair::new();
    let wallet_b = Keypair::new();
    let seed_amount: u64 = 1_000_000_000; // 1.0 SOL
    let stake_amount: u64 = 100_000_000; // 0.1 SOL
    let belief_a: u8 = 65;
    let belief_b: u8 = 15;
    let wager_account = Keypair::new();

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::transfer(&payer.pubkey(), &wallet_a.pubkey(), seed_amount),
            system_instruction::transfer(&payer.pubkey(), &wallet_b.pubkey(), seed_amount),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    //
    // STEP ONE: Create test wager and vault
    //

    // Derive vault PDA
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"vault", &wager_account.pubkey().to_bytes()],
        &program_id,
    );

    let wager = Wager {
        contract: contract_pubkey,
        vault: vault_pda,
        vault_bump: vault_bump,
        seat_count: 0,
        capacity: 2,
        stake: stake_amount,
    };

    let reserved_seats = vec!(wallet_a.pubkey(), wallet_b.pubkey());

    // Derive seat PDAs
    let mut seat_pdas = Vec::new();
    for i in 0..reserved_seats.len() {
        let (seat_pda, _seat_bump) = Pubkey::find_program_address(
            &[
                b"seat",
                &wager_account.pubkey().to_bytes(),
                &(i as u8).to_le_bytes(),
            ],
            &program_id,
        );
        seat_pdas.push(seat_pda);
    }

    // Build and encode wager
    let contract_data = WagerInstruction::CreateWager { wager, reserved_seats };
    let encoded_data = to_vec(&contract_data).unwrap();

    let mut accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(wager_account.pubkey(), true),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    for seat_pda in &seat_pdas {
        accounts.push(AccountMeta::new(*seat_pda, false));
    }

    let write_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        accounts,
    );

    // Create and send transaction
    let mut write_transaction = Transaction::new_with_payer(
        &[write_instruction], 
        Some(&payer.pubkey())
    );

    write_transaction.sign(&[&payer, &wager_account], recent_blockhash);
    banks_client.process_transaction(write_transaction).await.unwrap();

    /*
    //
    // STEP TWO: Process Deposits
    //

    // wallet_a
    let deposit_data = WagerInstruction::ProcessDeposit { amount: stake_amount };
    let encoded_data = to_vec(&deposit_data).unwrap();

    let deposit_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wallet_a.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Create and send transaction
    let mut deposit_transaction = Transaction::new_with_payer(
        &[deposit_instruction],
        Some(&payer.pubkey())
    );
    deposit_transaction.sign(&[&payer, &wallet_a], recent_blockhash);
    banks_client.process_transaction(deposit_transaction).await.unwrap();

    // wallet_b
    let deposit_data = WagerInstruction::ProcessDeposit { amount: stake_amount };
    let encoded_data = to_vec(&deposit_data).unwrap();

    let deposit_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wallet_b.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Create and send transaction
    let mut deposit_transaction = Transaction::new_with_payer(
        &[deposit_instruction],
        Some(&payer.pubkey())
    );
    deposit_transaction.sign(&[&payer, &wallet_b], recent_blockhash);
    banks_client.process_transaction(deposit_transaction).await.unwrap();

    
    //
    // STEP THREE: Update beliefs
    //

    // wallet_a
    let update_data = WagerInstruction::UpdateBelief { belief: belief_a };
    let encoded_data = to_vec(&update_data).unwrap();

    let update_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new_readonly(wallet_a.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
        ],
    );

    // Create and send transaction
    let mut update_transaction = Transaction::new_with_payer(
        &[update_instruction],
        Some(&payer.pubkey())
    );
    update_transaction.sign(&[&payer, &wallet_a], recent_blockhash);
    banks_client.process_transaction(update_transaction).await.unwrap();

    // wallet_b
    let update_data = WagerInstruction::UpdateBelief { belief: belief_b };
    let encoded_data = to_vec(&update_data).unwrap();

    let update_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new_readonly(wallet_b.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
        ],
    );

    // Create and send transaction
    let mut update_transaction = Transaction::new_with_payer(
        &[update_instruction],
        Some(&payer.pubkey())
    );
    update_transaction.sign(&[&payer, &wallet_b], recent_blockhash);
    banks_client.process_transaction(update_transaction).await.unwrap();

    //
    // STEP FOUR: Lock status
    //

    // wallet_a
    let lock_data = WagerInstruction::LockStatus;
    let encoded_data = to_vec(&lock_data).unwrap();

    let lock_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new_readonly(wallet_a.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
        ],
    );

    // Create and send transaction
    let mut lock_transaction = Transaction::new_with_payer(
        &[lock_instruction],
        Some(&payer.pubkey())
    );
    lock_transaction.sign(&[&payer, &wallet_a], recent_blockhash);
    banks_client.process_transaction(lock_transaction).await.unwrap();

    // wallet_b
    // TODO: check that payment does not worked until wallet_b is locked
    // TODO: reset Lock status for both when an account is 

    
    //
    // STEP FIVE: Set approval status
    //

    // wallet_a
    let approval_data = WagerInstruction::SetJudgment { judgment: Judgment::Landed };
    let encoded_data = to_vec(&approval_data).unwrap();

    let approval_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new_readonly(wallet_a.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
        ],
    );

    // Create and send transaction
    let mut approval_transaction = Transaction::new_with_payer(
        &[approval_instruction],
        Some(&payer.pubkey())
    );
    approval_transaction.sign(&[&payer, &wallet_a], recent_blockhash);
    banks_client.process_transaction(approval_transaction).await.unwrap();


    // wallet_b
    let approval_data = WagerInstruction::SetJudgment { judgment: Judgment::Landed };
    let encoded_data = to_vec(&approval_data).unwrap();

    let approval_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new_readonly(wallet_b.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false), 
        ],
    );

    // Create and send transaction
    let mut approval_transaction = Transaction::new_with_payer(
        &[approval_instruction],
        Some(&payer.pubkey())
    );
    approval_transaction.sign(&[&payer, &wallet_b], recent_blockhash);
    banks_client.process_transaction(approval_transaction).await.unwrap();

    //
    // STEP SIX: Render payouts
    //

    // wallet_a
    let variant = WagerInstruction::RenderPayouts;
    let encoded_data = to_vec(&variant).unwrap();

    let payout_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wallet_a.pubkey(), true),
            AccountMeta::new(wager_account.pubkey(), false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Create and send transaction
    let mut payout_transaction = Transaction::new_with_payer(
        &[payout_instruction],
        Some(&wallet_a.pubkey())
    );
    payout_transaction.sign(&[&wallet_a], recent_blockhash);
    banks_client.process_transaction(payout_transaction).await.unwrap();

    let vault = banks_client
        .get_account(vault_pda)
        .await
        .unwrap()
        .expect("vault should exist");

    println!("Vault: {:?}", &vault);
    */

    //
    // FINAL STEP: Get accounts and print!
    //

    let account = banks_client
        .get_account(wager_account.pubkey())
        .await
        .unwrap()
        .expect("account should exist");

    let account_data = Wager::try_from_slice(&account.data)
        .expect("Failed to deserialize account data");

    print_wager(&account_data);

    let vault = banks_client
        .get_account(vault_pda)
        .await
        .unwrap()
        .expect("vault should exist");

    println!("Vault: {:?}", &vault);

    let seat = banks_client
        .get_account(seat_pdas[0])
        .await
        .unwrap()
        .expect("account should exist");

    let seat_data = Seat::try_from_slice(&seat.data)
        .expect("Failed to deserialize account data");

    print_seat(&seat_data);
}


pub fn print_wager(wager: &Wager) {
    println!("📋 WAGER INFO:");
    println!("  Contract:    {}", wager.contract);
    println!("  Vault:       {}", wager.vault);
    println!("  Vault Bump:  {}", wager.vault_bump);
    println!("  Seat Count:  {}", wager.seat_count);
    println!("  Capacity:    {}", wager.capacity);
    println!("  Stake:       {} lamports", wager.stake);
    println!("\n");
}

pub fn print_seat(seat: &Seat) {
    println!("👤 SEAT INFO:");
    println!("  Wager:       {}", seat.wager);
    println!("  Authority:   {}", seat.authority);
    println!("  Belief:      {}", seat.belief);
    println!("  Status:      {:?}", seat.status);
    println!("  Judgment:    {:?}", seat.judgment);
    println!("  Last Change: {}", seat.last_change_at);
    println!("\n");
}