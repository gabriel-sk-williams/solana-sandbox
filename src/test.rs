#[cfg(test)]

use super::*;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

#[tokio::test]
async fn test_counter_program() {
    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "solana_god",
        program_id,
        processor!(process_instruction),
    )
    .start()
    .await;

    // Create a new keypair to use as the address for our counter account
    let counter_keypair = Keypair::new();
    let initial_value: u64 = 42;

    // Step 1: Initialize the counter
    println!("Testing counter initialization...");

    // Create initialization instruction
    let mut init_instruction_data = vec![0]; // 0 = initialize instruction
    init_instruction_data.extend_from_slice(&initial_value.to_le_bytes());

    let initialize_instruction = Instruction::new_with_bytes(
        program_id,
        &init_instruction_data,
        vec![
            AccountMeta::new(counter_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Send transaction with initialize instruction
    let mut transaction = Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &counter_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Check account data
    let account = banks_client
        .get_account(counter_keypair.pubkey())
        .await
        .expect("Failed to get counter account");

    if let Some(account_data) = account {
        let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
            .expect("Failed to deserialize counter data");
        assert_eq!(counter.count, 42);
        println!(
            "✅ Counter initialized successfully with value: {}",
            counter.count
        );
    }

    // Step 2: Increment the counter
    println!("Testing counter increment...");

    // Create increment instruction
    let increment_instruction = Instruction::new_with_bytes(
        program_id,
        &[1], // 1 = increment instruction
        vec![AccountMeta::new(counter_keypair.pubkey(), true)],
    );

    // Send transaction with increment instruction
    let mut transaction =
        Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &counter_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Check account data
    let account = banks_client
        .get_account(counter_keypair.pubkey())
        .await
        .expect("Failed to get counter account");

    if let Some(account_data) = account {
        let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
            .expect("Failed to deserialize counter data");
        assert_eq!(counter.count, 43);
        println!("✅ Counter incremented successfully to: {}", counter.count);
    }
}
