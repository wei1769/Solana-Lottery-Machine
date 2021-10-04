use std::borrow::Borrow;
use base64::encode;
use std::fs;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use clap::{App, load_yaml};
use crate::util::{Lottery, get_pub};
mod util;
mod lottery;


fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    
    let mut key_pair = Keypair::new();

    if matches.is_present("private"){
        // read key from arg
        let private = matches.value_of("private").unwrap();
        key_pair = Keypair::from_base58_string(private);
    }
    else{
        // read key from storage
        key_pair = util::load_config_keypair();
    }
    

    


    
    let wallet_publickey = key_pair.pubkey();

    let mut ins:Vec<Instruction> = vec![]; 
    let fee_payer = Some(&wallet_publickey);
    let mut signer: Vec<&Keypair> = vec![&key_pair];
    
    // change RPC endpoint here
    let rpc_url: String = "https://api.devnet.solana.com".to_string();
    let commitment = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url, commitment);
    
    let mut token_mint = util::get_pub("So11111111111111111111111111111111111111112");
    // Change the token mint for the lottery
    
    
    
    let mut lottery_id = get_pub("DeaqeKSVpBo7gheD1h8cVDUoWbBQ116oZ45amzvp8eEu");
    // This is for buy, draw, withdraw
    let mut  instruction_signer = Keypair::new();
    if let Some(ref matches) = matches.subcommand_matches("init"){
        let lottery_max_amount:u64 = matches.value_of("max_amount").unwrap().parse().unwrap();
        let slot_last:u64 = matches.value_of("slot_last").unwrap().parse().unwrap();
        if matches.is_present("mint") {
            token_mint = util::get_pub(matches.value_of("mint").unwrap());
        }
        let (mut init_ins, mut lottery_signer) = lottery::init_lottery(slot_last, lottery_max_amount, &token_mint, &wallet_publickey);
        println!("Lottery initialized, id: {:?}",lottery_signer.pubkey().clone());
        instruction_signer= lottery_signer;
        ins.append(&mut init_ins);
        signer.push(&instruction_signer);
    }
    else if let Some(ref matches) = matches.subcommand_matches("buy") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());
        let value_amount = matches.value_of("amount").unwrap();
        let ticket_buying_amount = value_amount.parse().unwrap();
        let (mut buy_ins, ticket_signer) = lottery::buy(&lottery_id, ticket_buying_amount,&wallet_publickey , rpc_client.borrow());
        println!("ticket bought, id: {:?}",ticket_signer.pubkey().clone());
        instruction_signer= ticket_signer;
        ins.append(&mut buy_ins);
        signer.push(&instruction_signer);
    }
    else if let Some(ref matches) = matches.subcommand_matches("draw") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());
        // currently just set the winner to 1st ticket buyer
        let mut draw_ins = lottery::draw(&lottery_id, &wallet_publickey,);
        ins.append(&mut draw_ins);
    }
    else if let Some(ref matches) = matches.subcommand_matches("withdraw") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());
        let mut withdraw_ins = lottery::withdraw(&lottery_id, &wallet_publickey, rpc_client.borrow());
        ins.append(&mut withdraw_ins);
    }
    else if let Some(ref matches) = matches.subcommand_matches("find") {
        lottery_id = get_pub(matches.value_of("lottery_id").unwrap());

        let tickets = lottery::findtickets(&lottery_id, rpc_client.borrow());
    
        for data in tickets {
        println!("{:?},{:?},{:?},{:?}", data.0, data.1,data.2,data.3);
        }
    }

    if !ins.is_empty(){
        let mut tx = Transaction::new_with_payer(&ins, fee_payer);
        let (recent, fee) = rpc_client
            .get_recent_blockhash()
            .expect("failed to get recent blockhash");
        
        tx.sign(&signer, recent);


        let messagee = encode(tx.message_data());
        // this is the raw message of a tx, it's for debugging


        let send = rpc_client.send_and_confirm_transaction_with_spinner(&tx);
        println!("tx: {:?} \nresult:{:?}",messagee, send);
    }
    
    
}
