use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    rpc_request::TokenAccountsFilter,
};
use solana_program::{program_pack::Pack, system_program};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    sysvar::{clock, rent},
};
use spl_associated_token_account;
use spl_token;
use std::borrow::Borrow;

use crate::util::{get_pub, getkey, Lottery, Ticket};
pub fn lottery_program_id() -> Pubkey {
    get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv")
}
pub fn init_lottery(
    slot: u64,
    max_amount: u64,
    mint: &Pubkey,
    authority: &Pubkey,
) -> (Vec<Instruction>, Keypair) {
    let fee_receiver = get_pub("2wnEcArzCpX1QRdtpHRXxZ7k9b1UeK16mPt26LPWFZ6V");
    let mut ins: Vec<Instruction> = vec![];
    let mut lottery_key = Keypair::new();
    while Pubkey::create_program_address(
        &[&lottery_key.pubkey().to_bytes()],
        &self::lottery_program_id(),
    )
    .is_err()
    {
        lottery_key = Keypair::new();
    }
    let lottery_pda = Pubkey::create_program_address(
        &[&lottery_key.pubkey().to_bytes()],
        &self::lottery_program_id(),
    )
    .unwrap();
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
    keys.push(getkey(spl_associated_token_account::id(), false, false));
    keys.push(getkey(mint.clone(), false, false));
    keys.push(getkey(spl_token::id(), false, false));
    keys.push(getkey(system_program::id(), false, false));
    keys.push(getkey(clock::id(), false, false));

    keys.push(getkey(rent::id(), false, false));

    data.push(0);

    data.extend_from_slice(&max_amount.to_le_bytes());

    data.extend_from_slice(&slot.to_le_bytes());
    let init_lottery_ins = Instruction {
        program_id: self::lottery_program_id(),
        data: data,
        accounts: keys,
    };
    ins.push(init_lottery_ins);

    (ins, lottery_key)
}

pub fn buy(
    lottery_id: &Pubkey,
    amount: u64,
    authority: &Pubkey,
    rpc_client: &RpcClient,
) -> (Vec<Instruction>, Keypair) {
    let mut ins: Vec<Instruction> = vec![];
    let ticket_key = Keypair::new();
    let lottery_data = rpc_client.get_account_data(lottery_id.borrow()).unwrap();
    let lottery_info = Lottery::unpack_unchecked(&lottery_data).unwrap();
    let mint = rpc_client
        .get_token_account(lottery_info.token_reciever.borrow())
        .unwrap()
        .unwrap();
    let buyer_token_accounts = rpc_client
        .get_token_accounts_by_owner(
            authority.borrow(),
            TokenAccountsFilter::Mint(get_pub(&mint.mint)),
        )
        .unwrap();

    let buyer_token_account = get_pub(&(buyer_token_accounts[0].pubkey));
    let mut data: Vec<u8> = vec![];
    let mut keys: Vec<AccountMeta> = vec![];

    //println!("Lottery info current :{:?},  max:{:?}",lottery_info.current_amount, lottery_info.max_amount);

    data.push(1);
    data.extend_from_slice(&amount.to_le_bytes());

    keys.push(getkey(lottery_id.clone(), false, true));
    keys.push(getkey(ticket_key.pubkey(), true, true));
    keys.push(getkey(authority.clone(), true, false));
    keys.push(getkey(lottery_info.token_reciever.clone(), false, true));
    keys.push(getkey(buyer_token_account, false, true));
    keys.push(getkey(spl_token::id(), false, false));
    keys.push(getkey(clock::id(), false, false));

    keys.push(getkey(system_program::id(), false, false));

    keys.push(getkey(rent::id(), false, false));
    let buy_ins = Instruction {
        program_id: self::lottery_program_id(),
        data: data,
        accounts: keys,
    };
    ins.push(buy_ins);

    (ins, ticket_key)
}

pub fn findtickets(lottery_id: &Pubkey, connection: &RpcClient) -> Vec<(u64, u64, Pubkey, Pubkey)> {
    let mut ticket_data: Vec<(u64, u64, Pubkey, Pubkey)> = vec![];
    let ticket_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");

    let mut mem: Vec<u8> = vec![2];
    mem.extend_from_slice(&lottery_id.to_bytes());
    let memcmp = MemcmpEncodedBytes::Base58(bs58::encode(mem).into_string());
    //println!("memcmp: {:?}\n",memcmp);
    let filter = Some(vec![RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: memcmp,
        encoding: None,
    })]);

    let config = RpcProgramAccountsConfig {
        filters: filter,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        with_context: None,
    };

    let accounts = connection
        .get_program_accounts_with_config(&ticket_program_id, config)
        .unwrap();
    //println!("{:?}",accounts);
    for data in accounts {
        let account = data.1;
        let current_ticket = Ticket::unpack_unchecked(&account.data).unwrap();
        let strart_number = current_ticket.start_number;
        let end_number = current_ticket.end_number;
        let ticket_buyer = current_ticket.buyer;
        ticket_data.push((strart_number, end_number, ticket_buyer, data.0));
    }
    ticket_data.sort();

    ticket_data
}

pub fn draw(lottery_id: &Pubkey, authority: &Pubkey) -> Vec<Instruction> {
    let mut ins: Vec<Instruction> = vec![];
    let mut data: Vec<u8> = vec![];
    let mut keys: Vec<AccountMeta> = vec![];
    keys.push(getkey(lottery_id.clone(), false, true));
    keys.push(getkey(authority.clone(), true, true));
    keys.push(getkey(clock::id(), false, false));
    keys.push(getkey(
        solana_program::sysvar::slot_hashes::id(),
        false,
        false,
    ));
    data.push(2);
    let draw_ins = Instruction {
        program_id: self::lottery_program_id(),
        data: data,
        accounts: keys,
    };
    ins.push(draw_ins);
    ins
}

pub fn withdraw(
    lottery_id: &Pubkey,
    authority: &Pubkey,
    connection: &RpcClient,
) -> Vec<Instruction> {
    let lottery_info = self::get_lottery_info(lottery_id, connection);

    let mut ins: Vec<Instruction> = vec![];
    let mut data: Vec<u8> = vec![];
    let mut keys: Vec<AccountMeta> = vec![];

    let (winner_ticket_id, winning_buyer) = self::find_winning_ticket(lottery_id, connection);
    let lottery_ata_info = connection
        .get_account_data(&lottery_info.token_reciever)
        .unwrap();
    let mint = spl_token::state::Account::unpack_unchecked(&lottery_ata_info)
        .unwrap()
        .mint;
    println!("{:?}", winner_ticket_id);
    let lottery_pda =
        Pubkey::create_program_address(&[&lottery_id.to_bytes()], &self::lottery_program_id())
            .unwrap();
    let winner_ata =
        spl_associated_token_account::get_associated_token_address(&winning_buyer, &mint);
    keys.push(getkey(lottery_id.clone(), false, true));
    keys.push(getkey(authority.clone(), true, true));
    keys.push(getkey(lottery_info.token_reciever.clone(), false, true));
    keys.push(getkey(lottery_info.fee_reciever.clone(), false, true));
    keys.push(getkey(winner_ata.clone(), false, true));
    keys.push(getkey(winner_ticket_id.clone(), false, false));
    keys.push(getkey(lottery_pda.clone(), false, false));
    keys.push(getkey(mint.clone(), false, false));
    keys.push(getkey(spl_token::id().clone(), false, false));
    keys.push(getkey(system_program::id().clone(), false, false));
    keys.push(getkey(rent::id(), false, false));
    keys.push(getkey(spl_associated_token_account::id(), false, false));
    keys.push(getkey(winning_buyer.clone(), false, false));

    data.push(3);
    let withdraw_ins = Instruction {
        program_id: self::lottery_program_id(),
        data: data,
        accounts: keys,
    };
    ins.push(withdraw_ins);
    ins
}

pub fn close(ticket_id: &Pubkey, lottery_id: &Pubkey, buyer: &Pubkey) -> Vec<Instruction> {
    let mut ins: Vec<Instruction> = vec![];
    let mut data: Vec<u8> = vec![];
    let mut keys: Vec<AccountMeta> = vec![];
    keys.push(getkey(lottery_id.clone(), false, false));
    keys.push(getkey(ticket_id.clone(), false, true));
    keys.push(getkey(buyer.clone(), false, true));

    data.push(4);
    let withdraw_ins = Instruction {
        program_id: self::lottery_program_id(),
        data: data,
        accounts: keys,
    };
    ins.push(withdraw_ins);
    ins
}

pub fn find_winning_ticket(lottery_id: &Pubkey, connection: &RpcClient) -> (Pubkey, Pubkey) {
    let lottery_info = self::get_lottery_info(lottery_id, connection);
    let tickets = self::findtickets(lottery_id, connection);
    let mut winningticket = Pubkey::default();
    let mut winningticket_buyer = Pubkey::default();
    for data in tickets {
        if lottery_info.lottery_number >= data.0 && lottery_info.lottery_number <= data.1 {
            winningticket_buyer = data.2;
            winningticket = data.3;
        }
    }
    (winningticket, winningticket_buyer)
}

pub fn get_lottery_info(lottery_id: &Pubkey, connection: &RpcClient) -> Lottery {
    let lottery_data = connection.get_account_data(lottery_id.borrow()).unwrap();
    let lottery_info = Lottery::unpack_unchecked(&lottery_data).unwrap();
    lottery_info
}

pub fn get_ended_lotterys(pool_authority: &Pubkey, connection: &RpcClient) -> Vec<Pubkey> {
    let ticket_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");
    let mut mem: Vec<u8> = vec![1];
    mem.extend_from_slice(&pool_authority.to_bytes());
    let memcmp = MemcmpEncodedBytes::Base58(bs58::encode(mem).into_string());
    //println!("memcmp: {:?}\n",memcmp);
    let filter = Some(vec![
        RpcFilterType::Memcmp(Memcmp {
            offset: 0,
            bytes: memcmp,
            encoding: None,
        }),
        RpcFilterType::DataSize(161),
    ]);

    let config = RpcProgramAccountsConfig {
        filters: filter,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        with_context: None,
    };
    let slot = connection.get_slot().unwrap();
    let mut lotteys: Vec<Pubkey> = vec![];
    let accounts = connection
        .get_program_accounts_with_config(&ticket_program_id, config)
        .unwrap();
    for data in accounts {
        let account = data.1;
        let current_ticket = Lottery::unpack_unchecked(&account.data).unwrap();
        let max_amount = current_ticket.max_amount;
        let current_amount = current_ticket.current_amount;
        let ended_slot = current_ticket.ended_slot;
        // print!("{:?}, {:?} , {:?}\n",data.0,max_amount,current_amount);
        if slot > ended_slot || current_amount > max_amount {
            lotteys.push(data.0);
        }
    }
    lotteys.sort();
    lotteys
}

pub fn get_withdrable_lotterys(pool_authority: &Pubkey, connection: &RpcClient) -> Vec<Pubkey> {
    let ticket_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");
    let mut mem: Vec<u8> = vec![3];
    mem.extend_from_slice(&pool_authority.to_bytes());
    let memcmp = MemcmpEncodedBytes::Base58(bs58::encode(mem).into_string());
    //println!("memcmp: {:?}\n",memcmp);
    let filter = Some(vec![
        RpcFilterType::Memcmp(Memcmp {
            offset: 0,
            bytes: memcmp,
            encoding: None,
        }),
        RpcFilterType::DataSize(161),
    ]);

    let config = RpcProgramAccountsConfig {
        filters: filter,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        with_context: None,
    };
    let mut lotteys: Vec<Pubkey> = vec![];
    let accounts = connection
        .get_program_accounts_with_config(&ticket_program_id, config)
        .unwrap();
    for data in accounts {
        let account = data.1;
        let current_ticket = Lottery::unpack_unchecked(&account.data).unwrap();
        let max_amount = current_ticket.max_amount;
        let current_amount = current_ticket.current_amount;
        print!("{:?}, {:?} , {:?}\n", data.0, max_amount, current_amount);

        lotteys.push(data.0);
    }
    lotteys.sort();
    lotteys
}

pub fn find_closable_tickets(buyer: &Pubkey, connection: &RpcClient) -> Vec<(Pubkey, Pubkey)> {
    let ticket_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");
    let mut mem: Vec<u8> = vec![];
    let account_type: Vec<u8> = vec![2];
    mem.extend_from_slice(&buyer.to_bytes());
    let memcmp = MemcmpEncodedBytes::Base58(bs58::encode(mem).into_string());
    let account_type_cmp = MemcmpEncodedBytes::Base58(bs58::encode(account_type).into_string());
    //println!("memcmp: {:?} , type :{:?}\n",memcmp, account_type_cmp);
    let filter = Some(vec![
        RpcFilterType::Memcmp(Memcmp {
            offset: 0,
            bytes: account_type_cmp,
            encoding: None,
        }),
        RpcFilterType::Memcmp(Memcmp {
            offset: 33,
            bytes: memcmp,
            encoding: None,
        }),
    ]);

    let config = RpcProgramAccountsConfig {
        filters: filter,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        with_context: None,
    };
    let mut tickets: Vec<(Pubkey, Pubkey)> = vec![];
    let accounts = connection
        .get_program_accounts_with_config(&ticket_program_id, config)
        .unwrap();
    //println!("accounts: {:?}",accounts);
    for data in accounts {
        let account = data.1;
        let current_ticket = Ticket::unpack_unchecked(&account.data).unwrap();
        let ended_lottery_id = current_ticket.lottery_id;
        let ended_lottery_info = self::get_lottery_info(&ended_lottery_id, connection);

        //print!("{:?}, {:?} \n",data.0,ended_lottery_id);
        if ended_lottery_info.account_type == 4 {
            tickets.push((data.0, ended_lottery_id));
        }
    }
    tickets.sort();
    tickets
}

pub fn find_all_closable_tickets(connection: &RpcClient) -> Vec<(Pubkey, Pubkey, Pubkey)> {
    let ticket_program_id = get_pub("42hrGQzkPQMXTmtpsE9hb9D7dTffzYXgqC4DHUHubJSv");
    let filter = Some(vec![RpcFilterType::DataSize(81)]);

    let config = RpcProgramAccountsConfig {
        filters: filter,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        with_context: None,
    };
    let mut tickets: Vec<(Pubkey, Pubkey, Pubkey)> = vec![];
    let accounts = connection
        .get_program_accounts_with_config(&ticket_program_id, config)
        .unwrap();
    //println!("accounts: {:?}",accounts);
    for data in accounts {
        let account = data.1;
        let current_ticket = Ticket::unpack_unchecked(&account.data).unwrap();
        let ended_lottery_id = current_ticket.lottery_id;
        let ended_lottery_info = self::get_lottery_info(&ended_lottery_id, connection);

        // print!("{:?}, {:?} \n",data.0,ended_lottery_id);
        if ended_lottery_info.account_type == 4 {
            tickets.push((data.0, ended_lottery_id, current_ticket.buyer));
        }
    }
    tickets.sort();
    tickets
}
