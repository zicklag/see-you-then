// Mark this test as BPF-only
#![cfg(feature = "test-bpf")]

use std::str::FromStr;

use borsh::BorshDeserialize;
use see_you_then::{create_time_slot, schedule_meeting, Reservation, TimeSlot, TimeSlotTime};
use solana_program::{system_instruction, system_program};

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
    let time_slot_keypair = Keypair::new();

    let (mut banks_client, payer_keypair, recent_blockhash) = program_test().start().await;

    // Create a new time slot
    let mut transaction = Transaction::new_with_payer(
        &[create_time_slot(
            program_id(),
            payer_keypair.pubkey(),
            time_slot_keypair.pubkey(),
            TimeSlotTime {
                start: 0.0,
                end: 120.0,
            },
            "My meeting".to_string(),
        )],
        Some(&payer_keypair.pubkey()),
    );
    transaction.sign(&[&payer_keypair, &time_slot_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Confirm that the new time slot has been created
    let time_slot_account = banks_client
        .get_account(time_slot_keypair.pubkey())
        .await
        .expect("success")
        .expect("some account");
    assert_eq!(time_slot_account.owner, program_id());

    // Parse created time slot and verify data
    let parsed_time_slot = TimeSlot::try_from_slice(&time_slot_account.data).expect("Deserialize");
    assert_eq!(parsed_time_slot.is_scheduled, false);
    assert_eq!(parsed_time_slot.owner, payer_keypair.pubkey());
    assert_eq!(parsed_time_slot.time.start, 0.0);
    assert_eq!(parsed_time_slot.time.end, 120.0);
    assert_eq!(parsed_time_slot.meeting_id, "My meeting".to_string());

    // Now create a new account that will schedule a meeting
    let scheduling_keypair = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::create_account(
            &payer_keypair.pubkey(),
            &scheduling_keypair.pubkey(),
            10000000000,
            0,
            &system_program::ID,
        )],
        Some(&payer_keypair.pubkey()),
    );
    transaction.sign(&[&payer_keypair, &scheduling_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    eprintln!(
        "scheduling: {:?}\ntime_slot: {:?}\n",
        scheduling_keypair.pubkey(),
        time_slot_keypair.pubkey(),
    );

    // And a reservation keypair
    let reservation_keypair = Keypair::new();

    // Create a transaction to schedule the time slot with this new account
    let mut transaction = Transaction::new_with_payer(
        &[schedule_meeting(
            program_id(),
            scheduling_keypair.pubkey(),
            reservation_keypair.pubkey(),
            time_slot_keypair.pubkey(),
            "John Doe".to_string(),
        )],
        Some(&scheduling_keypair.pubkey()),
    );
    transaction.sign(
        &[&scheduling_keypair, &reservation_keypair],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // Get the new reservation account
    let reservation_account = banks_client
        .get_account(reservation_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let parsed_reservation =
        Reservation::try_from_slice(&reservation_account.data).expect("Deserialize");

    // Get the updated time slot
    let time_slot_account = banks_client
        .get_account(time_slot_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let parsed_time_slot = TimeSlot::try_from_slice(&time_slot_account.data).expect("Deserialize");

    // Verify that the registration succeeded
    assert_eq!(parsed_time_slot.is_scheduled, true);
    assert_eq!(
        parsed_time_slot.scheduled_with_account,
        scheduling_keypair.pubkey()
    );

    assert_eq!(
        parsed_reservation.time_slot_account,
        time_slot_keypair.pubkey()
    );
    assert_eq!(parsed_reservation.name, "John Doe".to_string());

    // Try to double-book the time slot
    let reservation_2_keypair = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[schedule_meeting(
            program_id(),
            scheduling_keypair.pubkey(),
            reservation_2_keypair.pubkey(),
            time_slot_keypair.pubkey(),
            "John Doe 2".to_string(),
        )],
        Some(&scheduling_keypair.pubkey()),
    );
    transaction.sign(
        &[&scheduling_keypair, &reservation_2_keypair],
        recent_blockhash,
    );
    let result = banks_client.process_transaction(transaction).await;

    // Make sure that it doesn't let us double-book
    assert_eq!(result.is_err(), true);
}
