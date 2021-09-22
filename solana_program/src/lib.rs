use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};

entrypoint!(process_instruction);

#[derive(BorshDeserialize, BorshSerialize)]
pub enum SeeYouThenInstruction {
    /// Expected accounts when calling this instruction are
    ///
    /// 0. `[writeable,signer]` Funding account and owner of the new time slot (must be a system account)
    /// 1. `[writeable]` Unallocated account for storing the time slot
    /// 2. `[]` System program
    CreateTimeSlot {
        time: TimeSlotTime,
        meeting_id: String,
    },
    /// Expected accounts when calling this instruction are
    ///
    /// 0. `[writeable,signer]` scheduling account: Funding account and the account that will be scheduling this meeting
    /// 1. `[writeable]` reservation account: Unallocated account for storing the the meeting reservation
    /// 2. `[writeable]` time slot account: The account of the time slot that we want to schedule the meeting with
    /// 3. `[]` System program
    ScheduleMeeting { user_name: String },
}

/// Get an instruction that will create a time slot
///
/// # Arguments
/// - `program_id`: This must be the program ID of the `SeeYouThen` program
/// - `owner_account`: The owner of the new time slot, and also the person that will be funding the
///   new time slot
/// - `time_slot_account`: An un-allocated account that we will create the time slot under
/// - `time_slot_time`: The time to create the time slot with
pub fn create_time_slot(
    program_id: Pubkey,
    owner_account: Pubkey,
    time_slot_account: Pubkey,
    time_slot_time: TimeSlotTime,
    meeting_id: String,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner_account, true),
            AccountMeta::new(time_slot_account, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: SeeYouThenInstruction::CreateTimeSlot {
            time: time_slot_time,
            meeting_id,
        }
        .try_to_vec()
        .expect("IO error"),
    }
}

pub fn schedule_meeting(
    program_id: Pubkey,
    scheduling_account: Pubkey,
    reservation_account: Pubkey,
    time_slot_account: Pubkey,
    user_name: String,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(scheduling_account, true),
            AccountMeta::new(reservation_account, true),
            AccountMeta::new(time_slot_account, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: SeeYouThenInstruction::ScheduleMeeting { user_name }
            .try_to_vec()
            .expect("IO error"),
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct TimeSlot {
    pub owner: Pubkey,
    pub time: TimeSlotTime,
    pub is_scheduled: bool,
    /// The pubkey here only has meaning if `is_scheduled` is `true`.
    ///
    /// If `is_scheuled` is `false`, then the value of this will probably be `Pubkey::default()`, but
    /// either way it should be treated as if it is not present. The only reason it is not an
    /// `Option` is because we need the `PubKey` to reserve the space required to store it in the
    /// account when it is initially created.
    pub scheduled_with_account: Pubkey,
    pub meeting_id: String,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
pub struct Reservation {
    pub time_slot_account: Pubkey,
    pub name: String,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
/// The time slot range
pub struct TimeSlotTime {
    /// The start of the time slot in seconds relative to the unix epoch
    pub start: f64,
    /// The end of the time slot in seconds relative to the unix epoch
    pub end: f64,
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let processor = Processor::new(program_id, accounts, data);

    processor.process()?;

    Ok(())
}

struct Processor<'a, 'b> {
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'b>],
    data: &'a [u8],
}

impl<'a, 'b> Processor<'a, 'b> {
    fn new(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'b>], data: &'a [u8]) -> Self {
        Self {
            program_id,
            accounts,
            data,
        }
    }

    fn process(&self) -> ProgramResult {
        let transaction = SeeYouThenInstruction::try_from_slice(self.data)?;

        match transaction {
            SeeYouThenInstruction::CreateTimeSlot { time, meeting_id } => {
                self.create_time_slot(time, meeting_id)?
            }
            SeeYouThenInstruction::ScheduleMeeting { user_name } => {
                self.schedule_meeting(user_name)?
            }
        }

        Ok(())
    }

    fn create_time_slot(&self, time: TimeSlotTime, meeting_id: String) -> ProgramResult {
        msg!("Creating new time slot");
        let accounts = &mut self.accounts.iter();

        // Get our accounts
        let owner_account = next_account_info(accounts)?;
        let time_slot_account = next_account_info(accounts)?;
        let system_program_account = next_account_info(accounts)?;

        // Get the rent sysvar
        let rent = Rent::get()?;

        // Make sure that the owner of the account has signed this transaction
        if !owner_account.is_signer {
            msg!("The owner account must sign the transaction when creating a time slot");
            return Err(ProgramError::MissingRequiredSignature);
        }
        // Make sure that the owner is a system account ( not 100% sure this is necessary but I
        // can't think of a situation where it shouldn't be a system account )
        if owner_account.owner != &system_program::ID {
            msg!("The owner account must be a system program");
            return Err(ProgramError::InvalidAccountData);
        }

        // Create a timeslot with the given owner and time
        let time_slot = TimeSlot {
            owner: *owner_account.key,
            is_scheduled: false,
            scheduled_with_account: Pubkey::default(),
            time,
            meeting_id,
        };
        // Serialize the timeslot to it's raw bytes
        let mut time_slot_data = time_slot.try_to_vec()?;
        let time_slot_data_len = time_slot_data.len();

        // Create the time slot account
        invoke(
            &system_instruction::create_account(
                owner_account.key,
                time_slot_account.key,
                1.max(rent.minimum_balance(time_slot_data_len)),
                time_slot_data_len as u64,
                self.program_id,
            ),
            &[
                owner_account.clone(),
                time_slot_account.clone(),
                system_program_account.clone(),
            ],
        )?;
        // Make this program the owner of the new account
        invoke(
            &system_instruction::assign(time_slot_account.key, self.program_id),
            &[time_slot_account.clone(), system_program_account.clone()],
        )?;

        // Write the serialized data to the time slot account
        time_slot_data.swap_with_slice(*time_slot_account.try_borrow_mut_data()?);

        Ok(())
    }

    fn schedule_meeting(&self, user_name: String) -> ProgramResult {
        msg!("Scheduling a meeting");
        let accounts = &mut self.accounts.iter();

        // Get our accounts
        let scheduling_account = next_account_info(accounts)?;
        let reservation_account = next_account_info(accounts)?;
        let time_slot_account = next_account_info(accounts)?;
        let system_program_account = next_account_info(accounts)?;

        let rent = Rent::get()?;

        // Verify that the scheduling account has signed this transaction
        if !scheduling_account.is_signer {
            msg!("The transaction must be signed by the scheduling account");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Deserialize the time slot from the time slot account
        let mut time_slot = TimeSlot::try_from_slice(&time_slot_account.data.borrow())?;

        // Verify that this time slot has not already been scheduled
        if time_slot.is_scheduled {
            msg!("Time slot already scheduled with somebody else!");
            return Err(ProgramError::InvalidInstructionData);
        }

        // Create a reservation
        let reservation = Reservation {
            time_slot_account: *time_slot_account.key,
            name: user_name,
        };
        // Serialize the reservation to it's raw bytes
        let mut reservation_data = reservation.try_to_vec()?;
        let reservation_data_len = reservation_data.len();

        // Create the reservation account
        invoke(
            &system_instruction::create_account(
                scheduling_account.key,
                reservation_account.key,
                1.max(rent.minimum_balance(reservation_data_len)),
                reservation_data_len as u64,
                self.program_id,
            ),
            &[
                scheduling_account.clone(),
                reservation_account.clone(),
                system_program_account.clone(),
            ],
        )?;
        // Make this program the owner of the new reservation account
        invoke(
            &system_instruction::assign(reservation_account.key, self.program_id),
            &[reservation_account.clone(), system_program_account.clone()],
        )?;

        // Write the serialized reservation data to the reservation account
        reservation_data.swap_with_slice(*reservation_account.try_borrow_mut_data()?);

        // Schedule this time slot with the scheduling account
        time_slot.is_scheduled = true;
        time_slot.scheduled_with_account = *scheduling_account.key;

        // Serialize the updated time slot data to the time slot account
        let mut time_slot_data = time_slot.try_to_vec()?;
        time_slot_data.swap_with_slice(*time_slot_account.data.borrow_mut());

        Ok(())
    }
}
