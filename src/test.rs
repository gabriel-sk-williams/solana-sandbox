#![cfg(test)]

use crate::process_instruction;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};

#[tokio::test]
async fn test_solana_message() {
    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = 
        ProgramTest::new("solana_god", program_id, processor!(process_instruction))
            .start()
            .await;

    // Create the test message
    let test_message = "Receive THIS";
    
    // Create the instruction with the message as instruction data
    let instruction = solana_program::instruction::Instruction {
        program_id,
        accounts: vec![],
        data: test_message.as_bytes().to_vec(),  // Convert message to bytes
    };

    // Add the instruction to a new transaction
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // Process the transaction
    let transaction_result = banks_client.process_transaction(transaction).await;
    assert!(transaction_result.is_ok());
}
