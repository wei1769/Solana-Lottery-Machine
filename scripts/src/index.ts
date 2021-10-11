import { struct, u64, u8 } from "@project-serum/borsh";
import {
  Account,
  AccountMeta,
  Connection,
  PublicKey,
  sendAndConfirmTransaction,
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

// lottery program ID
const LOTTERY_PUBLIC_KEY = new PublicKey(
  "42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv"
);
// the account to receive the lottery commission fee
const FEE_RECEIVER_PUBLIC_KEY = new PublicKey(
  "2wnEcArzCpX1QRdtpHRXxZ7k9b1UeK16mPt26LPWFZ6V"
);
// the base token used for buying lottery
const WSOL_PUBLIC_KEY_RAW = "So11111111111111111111111111111111111111112";

// devnet connection
const connection = new Connection(
  "https://api.devnet.solana.com",
  "singleGossip"
);

export async function init_lottery(
  _mintPublicKeyRaw: string,
  _slot: number,
  _max_amount: number
) {
  const lotteryProgramId = new PublicKey(LOTTERY_PUBLIC_KEY);
  const _mintPublicKey = new PublicKey(_mintPublicKeyRaw);
  // console.log("lotteryProgramId", lotteryProgramId);

  let feeRecieverPublicKey = new PublicKey(FEE_RECEIVER_PUBLIC_KEY);
  // console.log("lotteryPublicKey", lotteryPublicKey);
  var recent_blockhash = (await connection.getRecentBlockhash()).blockhash;

  const newAccount = new Account();
  let newAccountPublicKey = newAccount.publicKey;
  const transaction = new Transaction();

  transaction.recentBlockhash = recent_blockhash;

  // create lottery PDA
  const lottery_pda = await PublicKey.createProgramAddress(
    [newAccountPublicKey.toBuffer()],
    lotteryProgramId
  );

  // find associated token account address
  let lottery_ata = await utils.findAssociatedTokenAddress(
    lottery_pda,
    _mintPublicKey
  );
  let fee_ata = await utils.findAssociatedTokenAddress(
    feeRecieverPublicKey,
    _mintPublicKey
  );

  // prepare keys
  /// 0.`[writable,signer]` lottery id
  /// 1.`[writable,signer]` lottery authority
  /// 2.`[]` fee authority
  /// 3.`[writable]` lottery PDA
  /// 4.`[writable]` lottery associated token account
  /// 5.`[writable]` fee associated token account
  /// 6.`[]` Associated Token Program
  /// 7.`[]` token mint
  /// 8.`[]` token program
  /// 9.`[]` system program
  /// 10.`[]` Sysvar Clock
  /// 11.`[]` Sysvar Rent
  const keys: AccountMeta[] = [
    { pubkey: newAccountPublicKey, isSigner: true, isWritable: true },
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
    {
      pubkey: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: _mintPublicKey,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("SysvarC1ock11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("SysvarRent111111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
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
      instruction: 0, // for the instruction index, see instruction.rs
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

  transaction.feePayer = managerAccount.publicKey;

  // put the data into https://explorer.solana.com/tx/inspector
  console.log(transaction.serializeMessage().toString("base64"));

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

export async function buy(_lotteryPoolId: string, _amount: number) {
  // init
  const newAccount = new Account();
  let newAccountPublicKey = newAccount.publicKey;
  const transaction = new Transaction();
  const recent_blockhash = (await connection.getRecentBlockhash()).blockhash;
  transaction.recentBlockhash = recent_blockhash;

  const lotteryProgramId = new PublicKey(LOTTERY_PUBLIC_KEY);
  const lotteryPoolPublicKey = new PublicKey(_lotteryPoolId);

  // get token receiver account
  const lotteryAccountInfo = await connection.getAccountInfo(
    lotteryPoolPublicKey
  );
  if (!lotteryAccountInfo)
    throw new Error("Lottery pool id not found! " + lotteryPoolPublicKey);

  const lotteryPoolData = utils.parseLotteryPoolData(lotteryAccountInfo.data);
  // console.log("lottery pool info", lotteryPoolData);

  // get my token ata
  // const buyer_token_ata = await utils.findAssociatedTokenAddress(
  //   managerAccount.publicKey,
  //   lotteryPoolData.token_mint
  // );
  // console.log(lotteryPoolData.token_mint.toString());
  const buyer_token_ata = await connection.getTokenAccountsByOwner(
    managerAccount.publicKey,
    {
      mint: new PublicKey(WSOL_PUBLIC_KEY_RAW),
    }
  );
  // referenced from program/src/instruction.rs
  /// 0.`[writable]` lottery id
  /// 1.`[writable,signer]` ticket id
  /// 2.`[writable,signer]` buyer authority
  /// 3.`[writable]` token reciever (ATA owned by lottery PDA, Derived from mint,lottery PDA)
  /// 4.`[writable]` buyer token account
  /// 5.`[]` token program
  /// 6.`[]` Sysvar: Clock
  /// 7.`[]` system program
  /// 8.`[]` Sysvar Rent
  const keys: AccountMeta[] = [
    {
      pubkey: lotteryPoolPublicKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: newAccountPublicKey, // ticket id
      isSigner: true,
      isWritable: true,
    },
    { pubkey: managerAccount.publicKey, isSigner: true, isWritable: true },
    {
      pubkey: lotteryPoolData.token_reciever,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: buyer_token_ata.value[0].pubkey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("SysvarC1ock11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },

    {
      pubkey: new PublicKey("SysvarRent111111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
  ];

  // prepare data
  // referenced from program/src/instruction.rs
  const dataLayout = struct([u8("instruction"), u64("amount")]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: new BN(1), // for the instruction index, see instruction.rs
      amount: new BN(_amount),
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

  transaction.feePayer = managerAccount.publicKey;

  // put the data into https://explorer.solana.com/tx/inspector
  console.log(transaction.serializeMessage().toString("base64"));

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

export async function draw(_lotteryPoolId: string) {
  // init
  const transaction = new Transaction();
  const recent_blockhash = (await connection.getRecentBlockhash()).blockhash;
  transaction.recentBlockhash = recent_blockhash;

  const lotteryProgramId = new PublicKey(LOTTERY_PUBLIC_KEY);
  const lotteryPoolPublicKey = new PublicKey(_lotteryPoolId);

  /// 0.`[writable]` lottery id
  /// 1.`[signer]` lottery authority
  /// 2.`[]` Sysvar: Clock
  /// 3.`[]` Sysvar: Slot Hashes
  const keys: AccountMeta[] = [
    {
      pubkey: lotteryPoolPublicKey,
      isSigner: false,
      isWritable: true,
    },
    { pubkey: managerAccount.publicKey, isSigner: true, isWritable: false },
    {
      pubkey: new PublicKey("SysvarC1ock11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("SysvarS1otHashes111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
  ];

  // prepare data
  // referenced from program/src/instruction.rs
  const dataLayout = struct([u8("instruction")]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: new BN(2), // for the instruction index, see instruction.rs
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

  transaction.feePayer = managerAccount.publicKey;

  // put the data into https://explorer.solana.com/tx/inspector
  console.log(transaction.serializeMessage().toString("base64"));

  const tx = await sendAndConfirmTransaction(
    connection,
    transaction,
    [managerAccount],
    {
      skipPreflight: false,
      commitment: "recent",
      preflightCommitment: "recent",
    }
  );
  console.log(`Tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
}

export async function withdraw(_lotteryPoolId: string) {
  // init
  const transaction = new Transaction();
  const recent_blockhash = (await connection.getRecentBlockhash()).blockhash;
  transaction.recentBlockhash = recent_blockhash;

  const lotteryProgramId = new PublicKey(LOTTERY_PUBLIC_KEY);
  const lotteryPoolPublicKey = new PublicKey(_lotteryPoolId);

  // get token receiver account
  const lotteryAccountInfo = await connection.getAccountInfo(
    lotteryPoolPublicKey
  );
  if (!lotteryAccountInfo)
    throw new Error("Lottery pool id not found! " + lotteryPoolPublicKey);

  const lotteryPoolData = utils.parseLotteryPoolData(lotteryAccountInfo.data);
  // console.log("lottery pool info", lotteryPoolData);

  // get lottery pda
  const lottery_pda = await PublicKey.findProgramAddress(
    [lotteryPoolPublicKey.toBuffer()],
    lotteryProgramId
  );

  // get winning ticket number
  const winningTicketNumber = lotteryPoolData.lottery_number;

  // get winning ticket
  let winningTicketId = new PublicKey(LOTTERY_PUBLIC_KEY); // LOTTERY_PUBLIC_KEY is placeholder
  let winnerTokenAccount = new PublicKey(LOTTERY_PUBLIC_KEY);
  let winningBuyerAccount = new PublicKey(LOTTERY_PUBLIC_KEY);

  const programAccounts = await connection.getProgramAccounts(lotteryProgramId);
  programAccounts.forEach(async (value) => {
    const ticketData = utils.parseTicketData(value.account.data);
    if (
      new BN(ticketData.start_number).lten(winningTicketNumber.valueOf()) &&
      new BN(ticketData.end_number).gten(winningTicketNumber.valueOf())
    ) {
      winningTicketId = value.pubkey;
      winnerTokenAccount = await utils.findAssociatedTokenAddress(
        lottery_pda[0],
        winningTicketId
      );
      winningBuyerAccount = ticketData.buyer;
      console.log("ticket found: ", ticketData.end_number.toString());
    }
    console.log(
      "ticket: ",
      ticketData.start_number.toString(),
      winningTicketNumber.toString(),
      ticketData.end_number.toString()
    );
  });

  /// 0.`[writable]` lottery id
  /// 1.`[writable,signer]` lottery authority
  /// 2.`[writable]` token reciever (ATA owned by lottery PDA, Derived from mint,lottery PDA)
  /// 3.`[writable]` fee reciever (ATA owned by fee authority)
  /// 4.`[writable]` winner token account
  /// 5.`[]` winning ticket id
  /// 6.`[]` lottery PDA
  /// 7.`[]` token mint
  /// 8.`[]` token program
  /// 9.`[]` system program
  /// 10.`[]` Sysvar Rent
  /// 11.`[]` Associated Token Program
  /// 12.`[]` Winner account
  const keys: AccountMeta[] = [
    {
      pubkey: lotteryPoolPublicKey,
      isSigner: false,
      isWritable: true,
    },
    { pubkey: managerAccount.publicKey, isSigner: true, isWritable: true },
    {
      pubkey: lotteryPoolData.token_reciever,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: lotteryPoolData.fee_reciever,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: winnerTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: winningTicketId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: lottery_pda[0],
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: lotteryPoolData.token_mint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("11111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("SysvarRent111111111111111111111111111111111"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: winningBuyerAccount,
      isSigner: false,
      isWritable: false,
    },
  ];

  // prepare data
  // referenced from program/src/instruction.rs
  const dataLayout = struct([u8("instruction")]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: new BN(3), // for the instruction index, see instruction.rs
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

  transaction.feePayer = managerAccount.publicKey;

  // put the data into https://explorer.solana.com/tx/inspector
  console.log(transaction.serializeMessage().toString("base64"));

  const tx = await sendAndConfirmTransaction(
    connection,
    transaction,
    [managerAccount],
    {
      skipPreflight: false,
      commitment: "recent",
      preflightCommitment: "recent",
    }
  );
  console.log(`Tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
}
export async function get_lottery_info(_lotteryPoolId: string) {}

// -------- function execution ---------

// init lottery which users can buy tickets. Different base tokens need to init different pools.
const max_amount = 100; // the total amount of lottery tickets
const slot = 100000000; // how long this lottery lasts (slot is blocknumber is Solana)
const mint_public_key = WSOL_PUBLIC_KEY_RAW; // user can use WSOL to buy tickets
// init_lottery(mint_public_key, slot, max_amount); // we only need to init once

// buy 1 ticket with 1 WSOL
const lottery_lotteryPoolId = "97FjhMuEQz8PNSJN1hX9UNsgtwYE6mmXpKXGJaCXgnjS"; // obtain the lottery pool ID from the init function above
// buy(lottery_lotteryPoolId, 1);

// draw from the pool
// draw(lottery_lotteryPoolId);

// withdraw from the pool
withdraw(lottery_lotteryPoolId);
