
use std::convert::TryInto;

use crate::{
    check_program_account,
    check_fee_account,
    error::TicketMachineError,
    instruction::LotteryMachineInstructions,
    state::{Lottery, Ticket},
};
use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult,  msg, program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, system_instruction::{self},  sysvar::Sysvar};
use spl_token::state::Account as TokenAccount;
use spl_token::instruction as TokenIns ;
use spl_associated_token_account::{create_associated_token_account,get_associated_token_address} ;
use solana_program::rent::Rent;
use solana_program::clock;

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = LotteryMachineInstructions::unpack(instruction_data)?;
        check_program_account(program_id)?;
        match instruction {
            LotteryMachineInstructions::InitLottery {
                max_amount,
                slot
            } => {
                msg!("Instruction: InitPool");
                Self::process_init_lottery(accounts, max_amount, slot,program_id)
            }
            LotteryMachineInstructions::Buy {
                amount
            } => {
                msg!("Instruction: Buy");
                Self::process_buy(accounts,amount,program_id)
            }
            LotteryMachineInstructions::Draw {

            }=> {
                msg!("Instruction: Draw");
                Self::process_draw()

            }
            LotteryMachineInstructions::Withdraw {

            }=> {
                msg!("Instruction: Withdraw");
                Self::process_withdraw()

            }

        }
    }

    fn process_init_lottery(
        accounts: &[AccountInfo],
        max_amount: u64,
        slot: u64,
        program_id:&Pubkey,
    ) -> ProgramResult {
        msg!("init lottery process");
        let account_info_iter = &mut accounts.iter();
        let lottery_id = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let fee_authority = next_account_info(account_info_iter)?;
        let lottery_pda = next_account_info(account_info_iter)?;
        let lottery_ata = next_account_info(account_info_iter)?;
        let fee_ata = next_account_info(account_info_iter)?;
        let ata_program  = next_account_info(account_info_iter)?;
        let token_mint  = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let system_program_account  = next_account_info(account_info_iter)?;
        let clock_account  = next_account_info(account_info_iter)?;
        let rent  = next_account_info(account_info_iter)?;

        msg!("account all unpacked");
        
        let writable_accounts = vec![lottery_id,authority,lottery_ata,fee_ata];
        if Self::check_writable(writable_accounts) {
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("writable_accounts cheked");
        let rent_info = Rent::from_account_info(rent)?;
        let create_inx = system_instruction::create_account(authority.key,lottery_id.key, rent_info.minimum_balance(Lottery::LEN) , Lottery::LEN.try_into().unwrap(), program_id);
        msg!("Create Lottery accounts");

        invoke(&create_inx, 
            &[lottery_id.clone(),
                            authority.clone(),
                            ])?;
        let mut lottery_info = Lottery::unpack_unchecked(
            &lottery_id.data.borrow()
        )?; 
        check_program_account(lottery_id.owner)?;
        if lottery_info.account_type != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        if !authority.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        msg!("All account type is good");
        let pda = 
            Pubkey::create_program_address(&[&lottery_id.key.to_bytes().clone()], program_id)?;
        if lottery_pda.key.clone() != pda{
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("PDA created");
        check_fee_account(fee_authority.key)?;
        
        if get_associated_token_address(
            fee_authority.key, 
            token_mint.key) != fee_ata.key.clone(){
                return Err(ProgramError::InvalidAccountData);
        }
        msg!("fee_ata is right");
        if fee_ata.owner != token_program.key{
            let  fee_ata_ix = create_associated_token_account(authority.key, &fee_authority.key.clone(), token_mint.key);
            invoke(&fee_ata_ix, 
                &[
                    authority.clone(),
                    fee_authority.clone(),
                    fee_ata.clone(),
                    system_program_account.clone(),
                    rent.clone(),
                    token_program.clone(),
                    token_mint.clone(),
                ])?;
        }

        if lottery_ata.owner != token_program.key{
            let  lottery_ata_ix = create_associated_token_account(authority.key, lottery_pda.key, token_mint.key);
            invoke(&lottery_ata_ix, 
                &[
                    authority.clone(),
                    lottery_pda.clone(),
                    lottery_ata.clone(),
                    system_program_account.clone(),
                    rent.clone(),
                    token_program.clone(),
                    token_mint.clone(),
                ])?;
        }

        let fee_token_account =TokenAccount::unpack(&fee_ata.data.borrow())?;
        
        if fee_token_account.owner != fee_authority.key.clone(){
            return Err(ProgramError::InvalidAccountData);
        }

        let clock_info = clock::Clock::from_account_info(clock_account)?;

        let slot_ended = clock_info.slot.checked_add(slot).unwrap();

        msg!("writing data to lottery info");
        lottery_info.account_type = 1;
        lottery_info.authority = authority.key.clone();
        lottery_info.lottery_number = 0;
        lottery_info.ended_slot = slot_ended;
        lottery_info.max_amount = max_amount;
        lottery_info.token_reciever = lottery_ata.key.clone();
        lottery_info.fee_reciever = fee_ata.key.clone();
        lottery_info.current_amount = 0;
        Lottery::pack(lottery_info, &mut lottery_id.data.borrow_mut())?;
        msg!(&*format!("Pool initialized: {:?}", lottery_id.key));
        
        Ok(())
    }


    fn process_buy(
        accounts: &[AccountInfo],
        amount:u64,
        program_id:&Pubkey,
        ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let lottery_id = next_account_info(account_info_iter)?;
        let ticket_id = next_account_info(account_info_iter)?;
        let buy_authority = next_account_info(account_info_iter)?;
        let lottery_ata = next_account_info(account_info_iter)?;
        let buyer_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let clock_account  = next_account_info(account_info_iter)?;
        let system_program_account  = next_account_info(account_info_iter)?;
        
        let rent  = next_account_info(account_info_iter)?;

        msg!("accounts all unpacked");
        let writable_accounts = vec![
            lottery_id,ticket_id,buy_authority,lottery_ata,buyer_token_account];
        
        if Self::check_writable(writable_accounts) {
            return Err(ProgramError::InvalidAccountData);
        }
        
        
        if ticket_id.data_is_empty(){
            let rent_info = Rent::from_account_info(rent)?;
            let create_inx = system_instruction::create_account(buy_authority.key,ticket_id.key, rent_info.minimum_balance(Ticket::LEN) , Ticket::LEN.try_into().unwrap(), program_id);
            msg!("Create Lottery accounts");
    
            invoke(&create_inx, 
                &[ticket_id.clone(),
                                buy_authority.clone(),
                                ])?;    
        }
        check_program_account(lottery_id.owner)?;
        check_program_account(ticket_id.owner)?;
        msg!("writable accounts cheked");
        
        let mut lottery_info = Lottery::unpack_unchecked(&lottery_id.data.borrow())?;

        let mut ticket_info = Ticket::unpack_unchecked(&ticket_id.data.borrow())?;
        let clock = clock::Clock::from_account_info(clock_account)?;
        
        if lottery_info.ended_slot < clock.slot.clone(){
            msg!("This Lottery ends");
            return Err(ProgramError::InvalidAccountData);
        }
        if lottery_info.current_amount > lottery_info.max_amount{
            msg!("This Lottery is full");
            return Err(ProgramError::InvalidAccountData);
        }
        
        ticket_info.account_type = 2 ;
        if buy_authority.is_signer{
            ticket_info.buyer = buy_authority.key.clone();
        }
        else {
            msg!("Buyer isn't signer");
            return Err(ProgramError::InvalidAccountData);
        }
        ticket_info.lottery_id = lottery_id.key.clone();
        ticket_info.start_number = lottery_info.current_amount.clone().checked_add(1).unwrap() ;
        if lottery_ata.key.clone() == lottery_info.token_reciever{
            let transfer_ix = TokenIns::transfer(
                &token_program.key.clone(),
                &buyer_token_account.key.clone(),
                &lottery_info.token_reciever.clone(),
                &buy_authority.key.clone(),
                &[&buy_authority.key.clone()], amount)?;
            invoke(&transfer_ix, &[
                    buyer_token_account.clone(),
                    lottery_ata.clone(),
                    buy_authority.clone(),

                ])?;
        }

        let end_unmber = amount.checked_add( lottery_info.current_amount).unwrap();
        lottery_info.current_amount = end_unmber.clone();
        ticket_info.end_number = end_unmber.clone();
        Ticket::pack(ticket_info, &mut ticket_id.data.borrow_mut() )?;
        Lottery::pack(lottery_info, &mut lottery_id.data.borrow_mut())?;
        Ok(())
    }


    fn process_draw()->ProgramResult {
        


        Ok(())
    }



    fn process_withdraw() ->ProgramResult {
    
        Ok(())
    } 
    fn check_writable(accounts:Vec<&AccountInfo>) -> bool {
        for x in accounts.iter()
         {
            if x.is_writable {
                return false;
            }
        }
        return true;
    }   
}
