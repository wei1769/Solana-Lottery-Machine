import { publicKey, struct, u64, u8 } from "@project-serum/borsh";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";

const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

// referenced from program/src/state.rs Lottery struct
const LOTTERY_LAYOUT = struct([
  u8("account_type"),
  publicKey("authority"),
  publicKey("token_reciever"),
  publicKey("fee_reciever"),
  u64("max_amount"),
  u64("ended_slot"),
  u64("lottery_number"),
  u64("current_amount"),
  publicKey("token_mint"),
]);

const TICKET_LAYOUT = struct([
  u8("account_type"),
  publicKey("lottery_id"),
  publicKey("buyer"),
  u64("start_number"),
  u64("end_number"),
]);

class Lottery {
  account_type: number;
  authority: PublicKey;
  token_reciever: PublicKey;
  fee_reciever: PublicKey;
  max_amount: number;
  ended_slot: number;
  lottery_number: number;
  current_amount: number;
  token_mint: PublicKey;

  constructor(
    account_type: number,
    authority: PublicKey,
    token_reciever: PublicKey,
    fee_reciever: PublicKey,
    max_amount: number,
    ended_slot: number,
    lottery_number: number,
    current_amount: number,
    token_mint: PublicKey
  ) {
    this.account_type = account_type;
    this.authority = authority;
    this.token_reciever = token_reciever;
    this.fee_reciever = fee_reciever;
    this.max_amount = max_amount;
    this.ended_slot = ended_slot;
    this.lottery_number = lottery_number;
    this.current_amount = current_amount;
    this.token_mint = token_mint;
  }
}

class Ticket {
  account_type: number;
  lottery_id: PublicKey;
  buyer: PublicKey;
  start_number: number;
  end_number: number;

  constructor(
    account_type: number,
    lottery_id: PublicKey,
    buyer: PublicKey,
    start_number: number,
    end_number: number
  ) {
    this.account_type = account_type;
    this.lottery_id = lottery_id;
    this.buyer = buyer;
    this.start_number = start_number;
    this.end_number = end_number;
  }
}

export async function findAssociatedTokenAddress(
  walletAddress: PublicKey,
  tokenMintAddress: PublicKey
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        walletAddress.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        tokenMintAddress.toBuffer(),
      ],
      SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    )
  )[0];
}

export function parseLotteryPoolData(data: any) {
  let {
    account_type,
    authority,
    token_reciever,
    fee_reciever,
    max_amount,
    ended_slot,
    lottery_number,
    current_amount,
    token_mint,
  } = LOTTERY_LAYOUT.decode(data);
  return new Lottery(
    account_type,
    authority,
    token_reciever,
    fee_reciever,
    max_amount,
    ended_slot,
    lottery_number,
    current_amount,
    token_mint
  );
}

export function parseTicketData(data: any) {
  let { account_type, lottery_id, buyer, start_number, end_number } =
    TICKET_LAYOUT.decode(data);
  return new Ticket(account_type, lottery_id, buyer, start_number, end_number);
}
