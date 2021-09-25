use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    rpc_request::TokenAccountsFilter,
    
};
use solana_program::{program_pack::Pack, system_program};
use spl_associated_token_account;
use std::borrow::Borrow;
use spl_token;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    sysvar::{{clock, rent}},
};

use crate::util::{get_pub, getkey, Lottery, Ticket};
pub fn init_lottery(
    slot: u64,
    max_amount: u64,
    mint: &Pubkey,
    authority: &Pubkey,
) -> (Vec<Instruction>, Keypair) {
    let lottery_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");
    let fee_receiver = get_pub("2wnEcArzCpX1QRdtpHRXxZ7k9b1UeK16mPt26LPWFZ6V");
    let mut ins: Vec<Instruction> = vec![];
    let mut lottery_key = Keypair::new();
    while Pubkey::create_program_address(&[&lottery_key.pubkey().to_bytes()], &lottery_program_id).is_err() {

        lottery_key = Keypair::new();
    }
    let lottery_pda = Pubkey::create_program_address(&[&lottery_key.pubkey().to_bytes()], &lottery_program_id).unwrap();
    let mut data: Vec<u8> = vec![];
    let mut keys: Vec<AccountMeta> = vec![];
    keys.push(getkey(lottery_key.pubkey(), true, true));
    keys.push(getkey(authority.clone(), true, true));
    keys.push(getkey(fee_receiver, false, false));
    
    keys.push(getkey(lottery_pda, false, false));
    let lottery_ata =
        spl_associated_token_account::get_associated_token_address(&lottery_pda, mint);
    keys.push(getkey(lottery_ata, false, true));

    let fee_ata = spl_associated_token_account::get_associated_token_address(&fee_receiver, mint);
    keys.push(getkey(fee_ata, false, true));
    keys.push(getkey(
        spl_associated_token_account::id(),
        false,
        false,
    ));
    keys.push(getkey(mint.clone(), false, false));
    keys.push(getkey(
        spl_token::id(),
        false,
        false,
    ));
    keys.push(getkey(
        system_program::id(),
        false,
        false,
    ));
    keys.push(getkey(
        clock::id(),
        false,
        false,
    ));

    keys.push(getkey(
        rent::id(),
        false,
        false,
    ));

    data.push(0);

    data.extend_from_slice(&max_amount.to_le_bytes());

    data.extend_from_slice(&slot.to_le_bytes());
    let init_pool_ins = Instruction {
        program_id: lottery_program_id,
        data: data,
        accounts: keys,
    };
    ins.push(init_pool_ins);

    (ins, lottery_key)
}


pub fn buy(
    lottery_id:&Pubkey,
    amount:u64,
    authority: &Pubkey,
    rpc_client: &RpcClient,
    
) -> (Vec<Instruction>, Keypair){
    let lottery_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");

    let mut ins: Vec<Instruction> = vec![];
    let mut ticket_key = Keypair::new();
    let lottery_data = rpc_client.get_account_data(lottery_id.borrow()).unwrap();
    let lottery_info = Lottery::unpack_unchecked(&lottery_data).unwrap();
    let mint = rpc_client.get_token_account(lottery_info.token_reciever.borrow()).unwrap().unwrap();
    let buyer_token_accounts = rpc_client.get_token_accounts_by_owner(authority.borrow(), TokenAccountsFilter::Mint( get_pub(&mint.mint))).unwrap();
    
    let buyer_token_account = get_pub(&(buyer_token_accounts[0].pubkey));
    let mut data: Vec<u8> = vec![];
    let mut keys: Vec<AccountMeta> = vec![];
    
    println!("Lottery info current :{:?},  max:{:?}",lottery_info.current_amount, lottery_info.max_amount);

    data.push(1);
    data.extend_from_slice(&amount.to_le_bytes());

    keys.push(getkey(lottery_id.clone(), false, true));
    keys.push(getkey(ticket_key.pubkey(), true, true));
    keys.push(getkey(authority.clone(), true, false));
    keys.push(getkey(lottery_info.token_reciever.clone(), false, true));
    keys.push(getkey(buyer_token_account, false, true));
    keys.push(getkey(
        spl_token::id(),
        false,
        false,
    ));
    keys.push(getkey(
        clock::id(),
        false,
        false,
    ));

    keys.push(getkey(
        system_program::id(),
        false,
        false,
    ));
    
    keys.push(getkey(
        rent::id(),
        false,
        false,
    ));
    let buy_ins = Instruction{
        program_id: lottery_program_id,
        data:data,
        accounts:keys,

    };
    ins.push(buy_ins);


    (ins, ticket_key)
}