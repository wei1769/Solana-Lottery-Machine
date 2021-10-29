use crate::util::get_pub;
use base64::encode;
use clap::{load_yaml, App};
use solana_account_decoder::parse_token::spl_token_v2_0_native_mint;
use solana_client::rpc_client::RpcClient;
use solana_program::system_instruction;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::borrow::Borrow;
mod lottery;
mod util;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let key_pair;

    if matches.is_present("private") {
        // read key from arg
        let private = matches.value_of("private").unwrap();
        key_pair = Keypair::from_base58_string(private);
    } else {
        // read key from storage
        key_pair = util::load_config_keypair();
    }

    let wallet_publickey = key_pair.pubkey();

    let mut ins: Vec<Instruction> = vec![];
    let fee_payer = Some(&wallet_publickey);
    let mut signer: Vec<&Keypair> = vec![&key_pair];

    // change RPC endpoint here
    let mut rpc_url: String = "https://api.testnet.solana.com".to_string();
    if matches.is_present("mainnet") {
        // read key from arg
        rpc_url = "https://api.mainnet-beta.solana.com".to_string();
    }
    let commitment = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url, commitment);

    let mut token_mint = util::get_pub("So11111111111111111111111111111111111111112");
    // Change the token mint for the lottery

    if matches.is_present("wrap") {
        let wrapped_amount: u64 = matches.value_of("wrap").unwrap().parse().unwrap();
        let wsol_ata = spl_associated_token_account::get_associated_token_address(
            &wallet_publickey,
            &spl_token_v2_0_native_mint(),
        );
        let wallet_info = rpc_client.get_account(&wallet_publickey).expect("cool");

        let ata = match rpc_client.get_account(&wsol_ata) {
            Ok(ata) => ata,
            Err(_err) => wallet_info,
        };

        if !spl_token::check_id(&ata.owner) {
            let create_ata_ins = spl_associated_token_account::create_associated_token_account(
                &wallet_publickey,
                &wallet_publickey,
                &spl_token_v2_0_native_mint(),
            );
            ins.push(create_ata_ins);
        }

        let transfer_ins =
            system_instruction::transfer(&wallet_publickey, &wsol_ata, wrapped_amount);
        ins.push(transfer_ins);
        let sync_ins = spl_token::instruction::sync_native(&spl_token::id(), &wsol_ata).unwrap();
        ins.push(sync_ins);
    }

    let lottery_id;
    // This is for buy, draw, withdraw
    let instruction_signer;

    if let Some(ref matches) = matches.subcommand_matches("init") {
        let lottery_max_amount: u64 = matches.value_of("max_amount").unwrap().parse().unwrap();
        let slot_last: u64 = matches.value_of("slot_last").unwrap().parse().unwrap();
        if matches.is_present("mint") {
            token_mint = util::get_pub(matches.value_of("mint").unwrap());
        }
        let (mut init_ins, lottery_signer) = lottery::init_lottery(
            slot_last,
            lottery_max_amount,
            &token_mint,
            &wallet_publickey,
        );
        println!(
            "Lottery initialized, id: {:?}",
            lottery_signer.pubkey().clone()
        );
        instruction_signer = lottery_signer;
        ins.append(&mut init_ins);
        signer.push(&instruction_signer);
    } else if let Some(ref matches) = matches.subcommand_matches("buy") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());
        let value_amount = matches.value_of("amount").unwrap();
        let ticket_buying_amount = value_amount.parse().unwrap();
        let (mut buy_ins, ticket_signer) = lottery::buy(
            &lottery_id,
            ticket_buying_amount,
            &wallet_publickey,
            rpc_client.borrow(),
        );
        println!("ticket bought, id: {:?}", ticket_signer.pubkey().clone());
        instruction_signer = ticket_signer;
        ins.append(&mut buy_ins);
        signer.push(&instruction_signer);
    } else if let Some(ref matches) = matches.subcommand_matches("draw") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());
        let mut draw_ins = lottery::draw(&lottery_id, &wallet_publickey);
        ins.append(&mut draw_ins);
    } else if let Some(ref matches) = matches.subcommand_matches("withdraw") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());
        let mut withdraw_ins =
            lottery::withdraw(&lottery_id, &wallet_publickey, rpc_client.borrow());
        ins.append(&mut withdraw_ins);
    } else if let Some(ref _matches) = matches.subcommand_matches("find") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());

        let tickets = lottery::findtickets(&lottery_id, rpc_client.borrow());

        for data in tickets {
            println!("{:?},{:?},{:?},{:?}", data.0, data.1, data.2, data.3);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("draw_all") {
        let ended_lotterys = lottery::get_ended_lotterys(&wallet_publickey, rpc_client.borrow());
        for id in ended_lotterys {
            let mut draw_ins = lottery::draw(&id, &wallet_publickey);
            ins.append(&mut draw_ins);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("withdraw_all") {
        let withdrawble_lottery =
            lottery::get_withdrable_lotterys(&wallet_publickey, rpc_client.borrow());
        for id in withdrawble_lottery {
            let mut withdraw_ins = lottery::withdraw(&id, &wallet_publickey, rpc_client.borrow());
            ins.append(&mut withdraw_ins);
            if ins.len() > 2 {
                break;
            }
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("close_all") {
        let tickets = lottery::find_closable_tickets(&wallet_publickey, rpc_client.borrow());
        println!("\n{:?}\n", tickets);
        for i in tickets {
            let mut close_ins = lottery::close(&i.0, &i.1, &wallet_publickey);
            ins.append(&mut close_ins);
            if ins.len() > 10 {
                break;
            }
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("close_every") {
        let tickets = lottery::find_all_closable_tickets(rpc_client.borrow());
        //println!("\n{:?}\n",tickets);
        for i in tickets {
            let mut close_ins = lottery::close(&i.0, &i.1, &i.2);
            ins.append(&mut close_ins);
            if ins.len() > 10 {
                break;
            }
        }
    }

    //print!("{:?},{:?}",get_ended_lotterys(&wallet_publickey, rpc_client.borrow()),wallet_publickey);
    if !ins.is_empty() {
        let mut tx = Transaction::new_with_payer(&ins, fee_payer);
        let (recent, _fee) = rpc_client
            .get_recent_blockhash()
            .expect("failed to get recent blockhash");

        tx.sign(&signer, recent);

        let messagee = encode(tx.message_data());
        // this is the raw message of a tx, it's for debugging

        let send = rpc_client.send_and_confirm_transaction_with_spinner(&tx);
        println!("result:{:?}\n {:?} \n", send, messagee);
    }
}
