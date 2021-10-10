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

// pub fn buy(
//     lottery_id:&Pubkey,
//     amount:u64,
//     authority: &Pubkey,
//     rpc_client: &RpcClient,

// )

export async function buy(_lotteryPoolId: string, _amount: number) {
  // init
  const newAccount = new Account();
  let newAccountPublicKey = newAccount.publicKey;
  const transaction = new Transaction();

  const lotteryPoolPublicKey = new PublicKey(_lotteryPoolId);

  // get token receiver account
  const lotteryAccountInfo = await connection.getAccountInfo(
    lotteryPoolPublicKey
  );
  if (!lotteryAccountInfo)
    throw new Error("Lottery pool id not found! " + lotteryPoolPublicKey);

  const lotteryPoolData = utils.parseLotteryPoolData(lotteryAccountInfo.data);
  console.log("lottery pool info", lotteryPoolData);

  // referenced from program/src/instruction.rs
  /// 0.`[writable]` lottery id
  /// 1.`[writable,signer]` ticket id
  /// 2.`[writable,signer]` buyer authority
  /// 3.`[writable]` token receiver account
  /// 4.`[writable]` authority token account
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
      pubkey: feeRecieverPublicKey,
      isSigner: false,
      isWritable: false,
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
}

export async function get_lottery_info(_lotteryPoolId: string) {}

// -------- function execution ---------

// init lottery which users can buy tickets. Different base tokens need to init different pools.
const max_amount = 100; // the total amount of lottery tickets
const slot = 100000000; // how long this lottery lasts (slot is blocknumber is Solana)
const mint_public_key = WSOL_PUBLIC_KEY_RAW; // user can use WSOL to buy tickets
init_lottery(mint_public_key, slot, max_amount);

// buy 1 ticket with 1 WSOL
const lottery_lotteryPoolId = "HimNHAmWUMK5ez5BfR5SG8725aWnWf9o9p6SWzwnkEU7"; // obtain the lottery pool ID from the init function above
buy(lottery_lotteryPoolId, 1);
