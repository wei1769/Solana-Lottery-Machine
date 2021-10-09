import { publicKey, struct, u64, u8 } from "@project-serum/borsh";
import {
  Account,
  AccountMeta,
  Connection,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import fs from "fs";
import os from "os";
import * as utils from "./utils";

const keyPairPath = os.homedir() + "/.config/solana/id.json";
const manager_private_key = JSON.parse(fs.readFileSync(keyPairPath, "utf-8"));

const managerAccount = new Account(manager_private_key);

const LOTTERY_PUBLIC_KEY = "42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv";
const FEE_RECEIVER_PUBLIC_KEY = "2wnEcArzCpX1QRdtpHRXxZ7k9b1UeK16mPt26LPWFZ6V";
const MINT_PUBLIC_KEY = "So11111111111111111111111111111111111111112"; // WSOL

// devnet connection
const connection = new Connection(
  "https://api.devnet.solana.com",
  "singleGossip"
);

// referenced from rust-cli/src/util.rs Lottery struct
const LOTTERY_LAYOUT = struct([
  u8("account_type"),
  publicKey("authority"),
  publicKey("token_reciever"),
  publicKey("fee_reciever"),
  u64("max_amount"),
  u64("ended_slot"),
  u64("lottery_number"),
  u64("current_amount"),
]);

// referenced from rust-cli/src/util.rs Ticket struct
const TICKET_LAYOUT = struct([
  u8("account_type"),
  publicKey("lottery_id"),
  publicKey("buyer"),
  u64("start_number"),
  u64("end_number"),
]);

export async function init_lottery(_slot: number, _max_amount: number) {
  const lotteryProgramId = new PublicKey(LOTTERY_PUBLIC_KEY);
  // console.log("lotteryProgramId", lotteryProgramId);

  const newAccount = new Account();
  let newAccountPublicKey = newAccount.publicKey;
  let feeRecieverPublicKey = new PublicKey(FEE_RECEIVER_PUBLIC_KEY);
  // console.log("lotteryPublicKey", lotteryPublicKey);

  const transaction = new Transaction();

  // create lottery account
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: managerAccount.publicKey,
      newAccountPubkey: newAccountPublicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        LOTTERY_LAYOUT.span
      ),
      space: LOTTERY_LAYOUT.span,
      programId: lotteryProgramId,
    })
  );

  // create lottery PDA
  const lottery_pda = await PublicKey.createProgramAddress(
    [],
    lotteryProgramId
  );

  // find associated token account address
  let lottery_ata = await utils.findAssociatedTokenAddress(
    managerAccount.publicKey,
    lotteryProgramId
  );
  let fee_ata = await utils.findAssociatedTokenAddress(
    managerAccount.publicKey,
    feeRecieverPublicKey
  );

  // prepare keys
  /// 0.`[writable,signer]` lottery id
  /// 1.`[writable,signer]` lottery authority
  /// 2.`[]` fee authority
  /// 3.`[writable]` lottery PDA
  /// 4.`[writable]` lottery associated token account
  /// 5.`[writable]` fee associated token account
  const keys: AccountMeta[] = [
    { pubkey: lotteryProgramId, isSigner: true, isWritable: true },
    { pubkey: managerAccount.publicKey, isSigner: true, isWritable: true },
    {
      pubkey: feeRecieverPublicKey,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: lottery_pda,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: lottery_ata,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: fee_ata,
      isSigner: false,
      isWritable: true,
    },
  ];

  // prepare data
  // referenced from program/src/instruction.rs
  const dataLayout = struct([
    u8("instruction"),
    u64("max_amount"),
    u64("slot"),
  ]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: 0,
      max_amount: new BN(_max_amount),
      slot: new BN(_slot),
    },
    data
  );

  // add init pool instruction to transaction
  transaction.add(
    new TransactionInstruction({
      keys,
      programId: lotteryProgramId,
      data,
    })
  );

  // return { transaction, newLotteryAccount };

  const tx = await sendAndConfirmTransaction(
    connection,
    transaction,
    [managerAccount, newAccount],
    {
      skipPreflight: false,
      commitment: "recent",
      preflightCommitment: "recent",
    }
  );
  console.log(`Tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
}

class PoolInfo {
  account_type: number;
  manager: PublicKey;
  fee_reciever: PublicKey;
  total_amount: number;
  price: number;
  fee: number;
  current_number: number;

  constructor(
    account_type: number,
    manager: PublicKey,
    fee_reciever: PublicKey,
    total_amount: number,
    price: number,
    fee: number,
    current_number: number
  ) {
    this.account_type = account_type;
    this.manager = manager;
    this.fee_reciever = fee_reciever;
    this.total_amount = total_amount;
    this.price = price;
    this.fee = fee;
    this.current_number = current_number;
  }
}

function parsePoolInfoData(data: any) {
  let {
    account_type,
    manager,
    fee_reciever,
    total_amount,
    price,
    fee,
    current_number,
  } = LOTTERY_LAYOUT.decode(data);
  return new PoolInfo(
    account_type,
    new PublicKey(manager),
    new PublicKey(fee_reciever),
    total_amount,
    price,
    fee,
    current_number
  );
}

// let { transaction, ticketKeyPair } = buy();
// export async function buy(pool_id: string, buyer: Account) {
//   const lotteryProgramId = new PublicKey(LOTTERY_PUBLIC_KEY);

//   // create new public key for ticket
//   const newTicketAccount = new Account();
//   let ticketPublicKey = newTicketAccount.publicKey;

//   // add create account instruction to transaction
//   const transaction = new Transaction();
//   // transaction.add(
//   //   SystemProgram.createAccount({
//   //     fromPubkey: buyer.publicKey,
//   //     newAccountPubkey: ticketPublicKey,
//   //     lamports: await connection.getMinimumBalanceForRentExemption(
//   //       TICKET_LAYOUT.span
//   //     ),
//   //     space: TICKET_LAYOUT.span,
//   //     programId: lotteryProgramId,
//   //   })
//   // );

//   // get pool info
//   // let poolInfo: AccountInfo<Buffer> | null = await connection.getAccountInfo(
//   //   new PublicKey(pool_id)
//   // );
//   // if (!poolInfo) throw new Error("Pool id not found! " + pool_id);
//   // else console.log("poolInfo", parsePoolInfoData(poolInfo.data));

//   // add buy instruction to transaction
//   const keys: AccountMeta[] = [
//     { pubkey: new PublicKey(pool_id), isSigner: false, isWritable: true },
//     {
//       pubkey: parsePoolInfoData(poolInfo.data).manager,
//       isSigner: false,
//       isWritable: true,
//     },
//     {
//       pubkey: new PublicKey(FEE_RECEIVER_PUBLIC_KEY),
//       isSigner: false,
//       isWritable: true,
//     },
//     {
//       pubkey: ticketPublicKey,
//       isSigner: true,
//       isWritable: true,
//     },
//     {
//       pubkey: buyer.publicKey,
//       isSigner: true,
//       isWritable: true,
//     },
//     {
//       pubkey: new PublicKey("11111111111111111111111111111111"),
//       isSigner: false,
//       isWritable: false,
//     },
//   ];

//   const dataLayout = struct([u8("instruction")]);
//   const data = Buffer.alloc(dataLayout.span);
//   dataLayout.encode(
//     {
//       instruction: 1,
//     },
//     data
//   );
//   transaction.add(
//     new TransactionInstruction({
//       keys,
//       programId: lotteryProgramId,
//       data,
//     })
//   );

//   // return { transaction, newTicketAccount };
//   const tx = await sendAndConfirmTransaction(
//     connection,
//     transaction,
//     [buyer, newTicketAccount],
//     {
//       skipPreflight: false,
//       commitment: "recent",
//       preflightCommitment: "recent",
//     }
//   );
//   console.log(`Tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
// }

// function execution
const max_amount = 100;
const slot = 100000000;
init_lottery(slot, max_amount);

// let pool_id = POOL_ID;
// let buyer = managerAccount;
// buy(pool_id, buyer);
