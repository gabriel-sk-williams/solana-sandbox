use borsh::to_vec;

use solana_god::{process_instruction};
use solana_god::instruction::{WagerInstruction};
use solana_god::state::{ApprovalState, VersusContract};

use solana_program::{
    pubkey::Pubkey,
    hash::hash,
    system_instruction,
    system_program,
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
    let stake_amount: u64 = 100_000_000; // 0.1 SOL
    let belief_a: u8 = 65;
    let belief_b: u8 = 15;

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::transfer(&payer.pubkey(), &wallet_a.pubkey(), 1_000_000_000),
            system_instruction::transfer(&payer.pubkey(), &wallet_b.pubkey(), 1_000_000_000),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    //
    // STEP ONE: Create test wager
    //

    let versus_contract = VersusContract {
        terms: "Trump switches to Regular Coke in 2025".to_string(),
        wallet_a: wallet_a.pubkey(),
        wallet_b: wallet_b.pubkey(),
        stake: stake_amount,
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

    // Build and encode wager
    let contract_data = WagerInstruction::CreateWager { contract: versus_contract };
    let encoded_data = to_vec(&contract_data).unwrap();

    let write_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Create and send transaction
    let mut write_transaction = Transaction::new_with_payer(
        &[write_instruction], 
        Some(&payer.pubkey())
    );
    write_transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(write_transaction).await.unwrap();

    //
    // STEP TWO: Process Deposits
    //

    let deposit_data = WagerInstruction::ProcessDeposit { amount: stake_amount };
    let encoded_data = to_vec(&deposit_data).unwrap();

    let deposit_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new(wallet_a.pubkey(), true),
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
    let update_data = WagerInstruction::UpdateBelief { belief: belief_b };
    let encoded_data = to_vec(&update_data).unwrap();

    let update_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
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
    let result = banks_client.process_transaction(update_transaction).await; // .unwrap();
    assert!(result.is_err());

    //
    // STEP FOUR: Lock status
    //

    // wallet_a
    let update_data = WagerInstruction::LockStatus;
    let encoded_data = to_vec(&update_data).unwrap();

    let lock_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new_readonly(wallet_a.pubkey(), true),
        ],
    );

    // Create and send transaction
    let mut lock_transaction = Transaction::new_with_payer(
        &[lock_instruction],
        Some(&payer.pubkey())
    );
    lock_transaction.sign(&[&payer, &wallet_a], recent_blockhash);
    banks_client.process_transaction(lock_transaction).await.unwrap();

    //
    // STEP FIVE: Set approval status
    //

    // wallet_a
    let approval_data = WagerInstruction::SetApproval { decision: ApprovalState::Landed };
    let encoded_data = to_vec(&approval_data).unwrap();

    let approval_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new_readonly(wallet_a.pubkey(), true),
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
    let approval_data = WagerInstruction::SetApproval { decision: ApprovalState::Landed };
    let encoded_data = to_vec(&approval_data).unwrap();

    let approval_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data,
        vec![
            AccountMeta::new(wager_pda, false),
            AccountMeta::new_readonly(wallet_b.pubkey(), true),
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
    // STEP FIVE: Read the wager
    //

    let variant = WagerInstruction::GetWager;
    let encoded_data = to_vec(&variant).unwrap();

    let read_instruction = Instruction::new_with_bytes(
        program_id,
        &encoded_data, // &[0]
        vec![AccountMeta::new_readonly(wager_pda, false)],
    );

    // Create and send transaction
    let mut read_transaction = Transaction::new_with_payer(
        &[read_instruction],
        Some(&payer.pubkey())
    );
    read_transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(read_transaction).await.unwrap();

}