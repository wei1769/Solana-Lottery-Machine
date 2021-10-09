import { publicKey, struct, u64, u8 } from "@project-serum/borsh";
import {
  Account,
  AccountInfo,
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

const keyPairPath = os.homedir() + "/.config/solana/id.json";
const manager_private_key = JSON.parse(fs.readFileSync(keyPairPath, "utf-8"));

const managerAccount = new Account(manager_private_key);

const POOL_ID = "D9ioyVKEQkjbEpQFcQPDHQkTCfuKJU8QLzN6xcbr7LAe";
const TICKET_PUBLIC_KEY = "AUaGuQhpjttMdBmejoboMoUMrpcxNHZsT44C6jupLYNP";
const FEE_RECEIVER_PUBLIC_KEY = "2wnEcArzCpX1QRdtpHRXxZ7k9b1UeK16mPt26LPWFZ6V";

// devnet connection
const connection = new Connection(
  "https://api.devnet.solana.com",
  "singleGossip"
);

// async function getLamports() {
//   const accountInfo = await connection.getAccountInfo(
//     new PublicKey(POOL_MANAGER_PUBLIC_KEY)
//   );
//   const { lamports } = accountInfo;
//   console.log("lamports", lamports);
// }

// referenced from program/src/state.rs Pool struct
const POOL_LAYOUT = struct([
  u8("account_type"),
  publicKey("manager"),
  publicKey("fee_reciever"),
  u64("total_amount"),
  u64("price"),
  u8("fee"),
  u64("current_number"),
]);

// referenced from program/src/state.rs Ticket struct
const TICKET_LAYOUT = struct([
  u8("account_type"),
  publicKey("pool_id"),
  u64("ticketnumber"),
  publicKey("ticketbuyer"),
]);

// let { transaction, poolKeyPair } = initPool();
export async function initPool(
  managerAccount: Account,
  price: number,
  fee: number,
  total_amount: number
) {
  // ticket program Id is hard-coded
  const ticketProgramId = new PublicKey(TICKET_PUBLIC_KEY);
  // console.log("ticketProgramId", ticketProgramId);

  // create new public key for pool
  const newPoolAccount = new Account();
  let poolPublicKey = newPoolAccount.publicKey;
  // console.log("poolPublicKey", poolPublicKey);

  // add create account instruction to transaction
  const transaction = new Transaction();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: managerAccount.publicKey,
      newAccountPubkey: poolPublicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        POOL_LAYOUT.span
      ),
      space: POOL_LAYOUT.span,
      programId: ticketProgramId,
    })
  );

  // prepare keys
  const keys: AccountMeta[] = [
    { pubkey: poolPublicKey, isSigner: true, isWritable: true },
    { pubkey: managerAccount.publicKey, isSigner: true, isWritable: true },
    {
      pubkey: new PublicKey(FEE_RECEIVER_PUBLIC_KEY),
      isSigner: false,
      isWritable: true,
    },
  ];

  // prepare data
  // init pool layout, referenced from program/src/instruction.rs
  const dataLayout = struct([
    u8("instruction"),
    u64("price"),
    u8("fee"),
    u64("total_amount"),
  ]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: 0,
      price: new BN(price),
      fee: new BN(fee),
      total_amount: new BN(total_amount),
    },
    data
  );

  // add init pool instruction to transaction
  transaction.add(
    new TransactionInstruction({
      keys,
      programId: ticketProgramId,
      data,
    })
  );

  // return { transaction, newPoolAccount };

  const tx = await sendAndConfirmTransaction(
    connection,
    transaction,
    [managerAccount, newPoolAccount],
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
  } = POOL_LAYOUT.decode(data);
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
export async function buy(pool_id: string, buyer: Account) {
  const ticketProgramId = new PublicKey(TICKET_PUBLIC_KEY);

  // create new public key for ticket
  const newTicketAccount = new Account();
  let ticketPublicKey = newTicketAccount.publicKey;

  // add create account instruction to transaction
  const transaction = new Transaction();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: buyer.publicKey,
      newAccountPubkey: ticketPublicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        TICKET_LAYOUT.span
      ),
      space: TICKET_LAYOUT.span,
      programId: ticketProgramId,
    })
  );

  // get pool info
  let poolInfo: AccountInfo<Buffer> | null = await connection.getAccountInfo(
    new PublicKey(pool_id)
  );
  if (!poolInfo) throw new Error("Pool id not found! " + pool_id);
  else console.log("poolInfo", parsePoolInfoData(poolInfo.data));

  // add buy instruction to transaction
  const keys: AccountMeta[] = [
    { pubkey: new PublicKey(pool_id), isSigner: false, isWritable: true },
    {
      pubkey: parsePoolInfoData(poolInfo.data).manager,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: new PublicKey(FEE_RECEIVER_PUBLIC_KEY),
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: ticketPublicKey,
      isSigner: true,
      isWritable: true,
    },
    {
      pubkey: buyer.publicKey,
      isSigner: true,
      isWritable: true,
    },
    {
      pubkey: new PublicKey("11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
  ];

  const dataLayout = struct([u8("instruction")]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: 1,
    },
    data
  );
  transaction.add(
    new TransactionInstruction({
      keys,
      programId: ticketProgramId,
      data,
    })
  );

  // return { transaction, newTicketAccount };
  const tx = await sendAndConfirmTransaction(
    connection,
    transaction,
    [buyer, newTicketAccount],
    {
      skipPreflight: false,
      commitment: "recent",
      preflightCommitment: "recent",
    }
  );
  console.log(`Tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
}

// I'm the pool manager, so fill in my own public key
let manager = managerAccount;
let price = 696969;
let fee = 23;
let amount = 10;
initPool(manager, price, fee, amount);

let pool_id = POOL_ID;
let buyer = managerAccount;
buy(pool_id, buyer);
