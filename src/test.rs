#![cfg(test)]

use borsh::to_vec;
use crate::{process_instruction, MessageInstruction};
use solana_program::{system_program, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_message_storage() {
    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = 
        ProgramTest::new("solana_god", program_id, processor!(process_instruction))
            .start()
            .await;

    // Create a new keypair for the message account
    let message_account = Keypair::new();
    
    // Step 1: Create test message and instruction
    let test_message = "gg".to_string();
    let instruction_data = MessageInstruction::CreateMessage {
        message: test_message.clone(),
    };

    let encoded_data = to_vec(&instruction_data).unwrap();

    // Create write instruction
    let write_instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(message_account.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: encoded_data,
    };

    // Create and send transaction
    let mut write_transaction = Transaction::new_with_payer(
        &[write_instruction], 
        Some(&payer.pubkey())
    );
    write_transaction.sign(&[&payer, &message_account], recent_blockhash);
    banks_client.process_transaction(write_transaction).await.unwrap();

    // Step 2: Test reading the message
    
    // Create read instruction
    let read_instruction = Instruction::new_with_bytes(
        program_id,
        &[1], // 1 = increment instruction
        vec![AccountMeta::new(message_account.pubkey(), true)],
    );

    // Create and send transaction
    let mut read_transaction = Transaction::new_with_payer(
        &[read_instruction],
        Some(&payer.pubkey())
    );
    read_transaction.sign(&[&payer, &message_account], recent_blockhash);
    banks_client.process_transaction(read_transaction).await.unwrap();
}

#[tokio::test]
async fn test_float_send() {
    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = 
        ProgramTest::new("solana_god", program_id, processor!(process_instruction))
            .start()
            .await;

    // Create a new keypair for the message account
    let message_account = Keypair::new();
    
    // Step 1: Create test message and instruction
    let test_float: f64 = 0.32;
    let instruction_data = MessageInstruction::ParseFloat {
        float: test_float,
    };

    let encoded_data = to_vec(&instruction_data).unwrap();

    // Create write instruction
    let write_instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(message_account.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: encoded_data,
    };

    // Create and send transaction
    let mut write_transaction = Transaction::new_with_payer(
        &[write_instruction], 
        Some(&payer.pubkey())
    );
    write_transaction.sign(&[&payer, &message_account], recent_blockhash);
    banks_client.process_transaction(write_transaction).await.unwrap();
}

// Check account data
/*
let account = banks_client
    .get_account(message_account.pubkey())
    .await
    .expect("Failed to get message account");


if let Some(account_data) = account {
    let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
        .expect("Failed to deserialize counter data");
    assert_eq!(counter.count, 43);
    println!("✅ Counter incremented successfully to: {}", counter.count);
}
*/