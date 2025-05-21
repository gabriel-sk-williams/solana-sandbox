#![cfg(test)]

use borsh::to_vec;
use crate::{process_instruction, ApprovalState, VersusContract, WagerInstruction};
use solana_program::{
    system_program, 
    pubkey::Pubkey,
    hash::hash,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Signer,
    transaction::Transaction,
    signer::keypair::Keypair,
};


#[tokio::test]
async fn test_versus_contract() {

    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = 
        ProgramTest::new("solana_god", program_id, processor!(process_instruction))
            .start()
            .await;

    let wallet_a = Keypair::new();
    let wallet_b = Keypair::new();

    //
    // Step 1: Create test wager
    //
    let versus_contract = VersusContract {
        terms: "Trump switches to Regular Coke in 2025".to_string(),
        wallet_a: wallet_a.pubkey(),
        wallet_b: wallet_b.pubkey(),
        stake: 100000000, // 0.1 SOL 100_000_000
    };

    // Hash terms with both wallets
    let terms_hash = hash(versus_contract.terms.as_bytes()).to_bytes();
    let (wager_pda, _bump) = Pubkey::find_program_address(
        &[
            &terms_hash[..],
            versus_contract.wallet_a.as_ref(),
            versus_contract.wallet_b.as_ref(),
        ], 
        &program_id
    );

    let instruction_data = WagerInstruction::CreateWager {
        contract: versus_contract,
    };

    let encoded_data = to_vec(&instruction_data).unwrap();

    // Create write instruction
    let write_instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(wager_pda, false),
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
    write_transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(write_transaction).await.unwrap();

    
    //
    // Step 2: Test reading the wager
    //
    let read_instruction = Instruction::new_with_bytes(
        program_id,
        &[0], // get wager
        vec![AccountMeta::new_readonly(wager_pda, false)],
    );

    // Create and send transaction
    let mut read_transaction = Transaction::new_with_payer(
        &[read_instruction],
        Some(&payer.pubkey())
    );
    read_transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(read_transaction).await.unwrap();
    
    
    //
    // Step 3: Update status
    //
    
    // wallet_a
    let decision_a = ApprovalState::Landed as u8;

    let update_instruction = Instruction::new_with_bytes(
        program_id,
        &[2, decision_a],
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new_readonly(wallet_a.pubkey(), true),
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
    let decision_b = ApprovalState::Landed as u8;

    let update_instruction = Instruction::new_with_bytes(
        program_id,
        &[2, decision_b],
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new_readonly(wallet_b.pubkey(), true),
        ],
    );

    // Create and send transaction
    let mut update_transaction = Transaction::new_with_payer(
        &[update_instruction],
        Some(&payer.pubkey())
    );
    update_transaction.sign(&[&payer, &wallet_b], recent_blockhash);
    banks_client.process_transaction(update_transaction).await.unwrap();

}   