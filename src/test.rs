#![cfg(test)]

use borsh::to_vec;
use crate::{process_instruction, SpaceInstruction, DualSpace};
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
};


#[tokio::test]
async fn test_dual_space() {

    let program_id = Pubkey::new_unique();
    let (banks_client, payer, recent_blockhash) = 
        ProgramTest::new("solana_god", program_id, processor!(process_instruction))
            .start()
            .await;
    
    // Create test space
    let dual_space = DualSpace {
        terms: "Trump switches to Regular Coke in 2025".to_string(),
        wallet_a: Pubkey::from_str_const("HWeDsoC6T9mCfaGKvoF7v6WdZyfEFhU2VaPEMzEjCq3J"),
        belief_a: 0.65,
        wallet_b: Pubkey::from_str_const("7V4wLNxUvejyeZ5Bmr2GpvfBL1mZxzQMhsyR7noiM3uD"),
        belief_b: 0.88,
    };

    // Hash terms with both wallets
    let terms_hash = hash(dual_space.terms.as_bytes()).to_bytes();
    let (space_pda, _bump) = Pubkey::find_program_address(
        &[
            &terms_hash[..],
            dual_space.wallet_a.as_ref(),
            dual_space.wallet_b.as_ref(),
        ], 
        &program_id
    );

    let instruction_data = SpaceInstruction::CreateSpace {
        space: dual_space,
    };

    let encoded_data = to_vec(&instruction_data).unwrap();

    // Create write instruction
    let write_instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(space_pda, false),
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

    
    // Step 2: Test reading the space
    // Create read instruction
    let read_instruction = Instruction::new_with_bytes(
        program_id,
        &[1], // 1 = get space instruction
        vec![AccountMeta::new_readonly(space_pda, false)],
    );

    // Create and send transaction
    let mut read_transaction = Transaction::new_with_payer(
        &[read_instruction],
        Some(&payer.pubkey())
    );
    read_transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(read_transaction).await.unwrap();

}