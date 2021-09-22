import { connection } from './solana';
import { Buffer } from 'buffer';
import {
  Keypair,
  Transaction,
  TransactionInstruction,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { Notify } from 'quasar';
import { selectedWallet } from './wallet';

const programId = new PublicKey('F1EdFSDRttLzmcKvwkF6VTL4fSyDTyo3xx68kxSCdSwE');

export async function createTimeSlot(
  startDate: Date,
  endDate: Date,
  meetingId?: string
) {
  if (!meetingId || meetingId == '') {
    meetingId = btoa(Math.random().toString());
  }

  const wallet = selectedWallet.value;

  if (!wallet || !wallet.adapter.publicKey) {
    Notify.create({
      type: 'negative',
      message: 'Could not create time slot: wallet not connected',
    });
    return;
  }

  const ownerKey = new PublicKey(wallet.adapter.publicKey.toString());

  // TODO: Take out for production
  // Airdrop 1 sol so that we have enough for the transaction below
  await connection.requestAirdrop(ownerKey, LAMPORTS_PER_SOL);

  // Create a new account keypair to store the time slot in
  const timeSlotKeypair = new Keypair();
  const timeSlotCreateTransaction = new Transaction({
    feePayer: ownerKey,
    recentBlockhash: (await connection.getRecentBlockhash()).blockhash,
  });

  // Get the number of seconds since the unix epoch. We divide by 1000 because JavaScript dates are
  // recorded as the number of _miliseconds_ since the epoch, not seconds.
  const startTime = startDate.valueOf() / 1000;
  const endTime = endDate.valueOf() / 1000;

  // Borsh serialize the time slot creation instruction. Ironically this is easier than using
  // borsh.js 🙄
  const strLen = Buffer.byteLength(meetingId);
  const buffer = Buffer.alloc(
    // Instruction discriminant ( u8 )
    1 +
      // Size of start time ( f64 )
      8 +
      // Size of end time ( f64 )
      8 +
      // Size of the string size ( u32 )
      4 +
      // Size of the string
      strLen
  );
  let cursor = 0;
  cursor = buffer.writeUInt8(0, cursor);
  cursor = buffer.writeDoubleLE(startTime, cursor);
  cursor = buffer.writeDoubleLE(endTime, cursor);
  cursor = buffer.writeUInt32LE(strLen, cursor);
  buffer.write(meetingId, cursor);

  // Create transaction
  timeSlotCreateTransaction.add(
    new TransactionInstruction({
      programId,
      keys: [
        {
          pubkey: ownerKey,
          isWritable: true,
          isSigner: true,
        },
        {
          pubkey: timeSlotKeypair.publicKey,
          isWritable: true,
          isSigner: true,
        },
        {
          pubkey: SystemProgram.programId,
          isWritable: true,
          isSigner: false,
        },
      ],
      data: buffer,
    })
  );
  timeSlotCreateTransaction.sign(timeSlotKeypair);

  // Submit transaction
  const transactionId = await wallet.adapter.sendTransaction(
    timeSlotCreateTransaction,
    connection
  );
  console.log('Completed transaction', transactionId);
}

export async function scheduleMeeting(timeSlot: PublicKey, username: string) {
  const wallet = selectedWallet.value;

  if (!wallet || !wallet.adapter.publicKey) {
    Notify.create({
      type: 'negative',
      message: 'Could not create time slot: wallet not connected',
    });
    return;
  }

  const ownerKey = new PublicKey(wallet.adapter.publicKey.toString());

  // TODO: Take out for production
  // Airdrop 1 sol so that we have enough for the transaction below
  await connection.requestAirdrop(ownerKey, LAMPORTS_PER_SOL);

  // Create a new account keypair to store the reservation in
  const reservationKeypair = new Keypair();
  const reservationCreateTransaction = new Transaction({
    feePayer: ownerKey,
    recentBlockhash: (await connection.getRecentBlockhash()).blockhash,
  });

  // Borsh serialize the reservation creation instruction. Ironically this is easier than using
  // borsh.js 🙄
  const strLen = Buffer.byteLength(username);
  const buffer = Buffer.alloc(
    // Instruction discriminant ( u8 )
    1 +
      // Size of the string size ( u32 )
      4 +
      // Size of the string
      strLen
  );
  let cursor = 0;
  cursor = buffer.writeUInt8(1, cursor);
  cursor = buffer.writeUInt32LE(strLen, cursor);
  buffer.write(username, cursor);

  // Create transaction
  reservationCreateTransaction.add(
    new TransactionInstruction({
      programId,
      keys: [
        {
          pubkey: ownerKey,
          isWritable: true,
          isSigner: true,
        },
        {
          pubkey: reservationKeypair.publicKey,
          isWritable: true,
          isSigner: true,
        },
        {
          pubkey: timeSlot,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: SystemProgram.programId,
          isWritable: true,
          isSigner: false,
        },
      ],
      data: buffer,
    })
  );
  reservationCreateTransaction.sign(reservationKeypair);

  // Submit transaction
  const transactionId = await wallet.adapter.sendTransaction(
    reservationCreateTransaction,
    connection
  );
  console.log('Completed transaction', transactionId);

  Notify.create({
    type: 'positive',
    message: 'Successfully scheduled meeting! 🎉',
  });
}

export interface TimeSlot {
  id: PublicKey;
  owner: PublicKey;
  time: {
    start: Date;
    end: Date;
  };
  scheduledWith?: PublicKey;
  meetingId: string;
}

export async function getTimeSlots(owner: PublicKey): Promise<TimeSlot[]> {
  const timeSlots: TimeSlot[] = [];

  const programAccounts = await connection.getProgramAccounts(programId, {
    filters: [
      {
        memcmp: {
          offset: 0,
          bytes: owner.toBase58(),
        },
      },
    ],
  });

  for (const account of programAccounts) {
    timeSlots.push(deserializeTimeSlot(account.pubkey, account.account.data));
  }

  return timeSlots;
}

export function subscribeToTimeSlots(
  owner: PublicKey,
  callback: (slot: TimeSlot) => void
): number {
  const subscription = connection.onProgramAccountChange(
    programId,
    (update) => {
      callback(deserializeTimeSlot(update.accountId, update.accountInfo.data));
    },
    'confirmed',
    [
      {
        memcmp: {
          offset: 0,
          bytes: owner.toBase58(),
        },
      },
    ]
  );

  return subscription;
}

function deserializeTimeSlot(accountId: PublicKey, data: Buffer): TimeSlot {
  let cursor = 0;
  const nextBytes = (num: number) => {
    cursor += num;
    return num;
  };
  const ownerBytes = data.subarray(cursor, nextBytes(32));
  const owner = new PublicKey(ownerBytes);

  const start = new Date(data.readDoubleLE(cursor) * 1000);
  nextBytes(8);

  const end = new Date(data.readDoubleLE(cursor) * 1000);
  nextBytes(8);

  const isScheduled = data.readUInt8(cursor) == 1;
  nextBytes(1);

  const scheduledWithBytes = data.subarray(cursor, cursor + nextBytes(32));
  const scheduledWith = new PublicKey(scheduledWithBytes);

  const meetingIdStrLen = data.readUInt32LE(cursor);
  nextBytes(4);

  const meetingIdBytes = data.subarray(cursor, cursor + meetingIdStrLen);
  const meetingId = meetingIdBytes.toString();

  return {
    id: accountId,
    meetingId,
    owner,
    time: {
      start,
      end,
    },
    scheduledWith: isScheduled ? scheduledWith : undefined,
  };
}
