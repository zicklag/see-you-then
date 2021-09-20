use borsh::{BorshDeserialize, BorshSerialize};
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
    /// 3. `[]` Rent sysvar
    CreateTimeSlot { time: TimeSlotTime },
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TimeSlot {
    pub owner: Pubkey,
    pub scheduled_with: Option<Pubkey>,
    pub time: TimeSlotTime,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TimeSlotTime {
    pub day_time_start_secs: u32,
    pub day_time_end_secs: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
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
        }
        .try_to_vec()
        .expect("IO error"),
    }
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
            SeeYouThenInstruction::CreateTimeSlot { time } => self.create_time_slot(time)?,
        }

        Ok(())
    }

    fn create_time_slot(&self, time: TimeSlotTime) -> ProgramResult {
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
            scheduled_with: None,
            time,
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
}
