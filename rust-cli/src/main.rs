use std::borrow::Borrow;
use base64::encode;
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
use crate::util::{Lottery, get_pub};
mod util;
mod lottery;


fn main() {
    // Enter your Private key here
    let key_pair = Keypair::from_base58_string("");
    let wallet_publickey = key_pair.pubkey();

    let mut ins:Vec<Instruction> = vec![]; 
    let fee_payer = Some(&wallet_publickey);
    let mut signer: Vec<&Keypair> = vec![&key_pair];
    
    // change RPC endpoint here
    let rpc_url: String = "https://api.devnet.solana.com".to_string();
    let commitment = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url, commitment);
    
    let token_mint = util::get_pub("So11111111111111111111111111111111111111112");
    // Change the token mint for the lottery
    
    
    
    let lottery_id = get_pub("DeaqeKSVpBo7gheD1h8cVDUoWbBQ116oZ45amzvp8eEu");
    // This is for buy, draw, withdraw

     
    //initialize a lottery pool (remove /* */ in this block)
    /* 
    let lottery_max_amount = 100000;
    let slot_last = 100000000;
    //Change some arg here

    let (mut init_ins, lottery_signer) = lottery::init_lottery(1000000000, lottery_max_amount, &token_mint, &wallet_publickey);
    println!("Lottery initialize, id: {:?}",lottery_signer.pubkey().clone());
    ins.append(&mut init_ins);
    signer.push(&lottery_signer);
    */
    

    /* 
    //buy a ticket for a lottery id  (remove /* */ in this block)
    
    let ticket_buying_amount = 100000000;
    //Change some arg here
    
    let (mut buy_ins, ticket_signer) = lottery::buy(&lottery_id, ticket_buying_amount,&wallet_publickey , rpc_client.borrow());
    ins.append(&mut buy_ins);
    signer.push(&ticket_signer);
    
    */
    



    /* 
    //make a draw for a lottery id  (remove /* */ in this block) 
    // currently just set the winner to 1st ticket buyer
    let mut draw_ins = lottery::draw(&lottery_id, &wallet_publickey,);
    ins.append(&mut draw_ins);
    */


    /* 
    //withdraw the prize to the winner (remove /* */ in this block)
    
    let mut withdraw_ins = lottery::withdraw(&lottery_id, &wallet_publickey, rpc_client.borrow());
    ins.append(&mut withdraw_ins);

    */


    /* 
    //find tickets of a lottery (remove /* */ in this block)

    let tickets = lottery::findtickets(&lottery_id, rpc_client.borrow());
    
    for data in tickets {
        println!("{:?},{:?},{:?},{:?}", data.0, data.1,data.2,data.3);
    }
    */
    

    

    

    
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
