// Mark this test as BPF-only
#![cfg(feature = "test-bpf")]

use std::str::FromStr;

use borsh::BorshDeserialize;
use see_you_then::{create_time_slot, TimeSlot, TimeSlotTime};

use {
    see_you_then::process_instruction,
    solana_program::pubkey::Pubkey,
    solana_program_test::*,
    solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
};

/// Dummy program ID for tests
fn program_id() -> Pubkey {
    Pubkey::from_str("SeeYouThen111111111111111111111111111111111").unwrap()
}

fn program_test() -> ProgramTest {
    ProgramTest::new(
        "see_you_then",
        program_id(),
        processor!(process_instruction),
    )
}

#[tokio::test]
async fn time_slot_create_and_schedule() {
    let time_slot = Keypair::new();

    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    // Create a new feature proposal
    let mut transaction = Transaction::new_with_payer(
        &[create_time_slot(
            program_id(),
            payer.pubkey(),
            time_slot.pubkey(),
            TimeSlotTime {
                day_time_start_secs: 0,
                day_time_end_secs: 120,
                year: 2021,
                month: 10,
                day: 3,
            },
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &time_slot], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Confirm that the new time slot has been created
    let time_slot_account = banks_client
        .get_account(time_slot.pubkey())
        .await
        .expect("success")
        .expect("some account");
    assert_eq!(time_slot_account.owner, program_id());

    // Parse created time slot and verify data
    let parsed_time_slot = TimeSlot::try_from_slice(&time_slot_account.data).expect("Deserialize");
    assert_eq!(parsed_time_slot.scheduled_with, None);
    assert_eq!(parsed_time_slot.owner, payer.pubkey());
    assert_eq!(parsed_time_slot.time.day_time_start_secs, 0);
    assert_eq!(parsed_time_slot.time.day_time_end_secs, 120);
    assert_eq!(parsed_time_slot.time.month, 10);
    assert_eq!(parsed_time_slot.time.year, 2021);
    assert_eq!(parsed_time_slot.time.day, 3);
}
