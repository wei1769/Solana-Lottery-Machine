use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
pub struct Lottery  {
    pub account_type: u8,//1 is lottery ,3 is ended Lottery size:1
    pub authority: Pubkey,//size:32
    pub token_reciever: Pubkey,//size:32
    pub fee_reciever: Pubkey,//size:32
    pub max_amount: u64,//size:8
    pub ended_slot:  u64,//size:8
    pub lottery_number: u64,//size:8
    pub current_amount: u64,//size:8

    //Lottery account size should be 129 Bytes
}
pub struct Ticket{
    pub account_type: u8,//2 is Ticket size:1
    pub lottery_id: Pubkey,//size:32
    pub buyer: Pubkey,//size:32
    pub start_number:u64,//size:8
    pub end_number: u64,//size:32

    //Ticket account size should be 81 Bytes
    
}



impl Sealed for Ticket {}

impl IsInitialized for Ticket{
    fn is_initialized(&self) -> bool {
        self.account_type != 0 
    }
}

impl Pack for Ticket {
    const LEN: usize = 81;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        
        let src = array_ref![src, 0, Ticket::LEN];
        let(_account_type,
            _lottery_id,
            _buyer,
            _start_number,
            _end_number,
        ) = array_refs![src, 1, 32, 32, 8, 8];
        
        let account_type =u8::from_le_bytes(*_account_type);
        
        let lottery_id = Pubkey::new(_lottery_id);
        let buyer = Pubkey::new(_buyer);
        let start_number = u64::from_le_bytes(*_start_number);
        let end_number = u64::from_le_bytes(*_end_number);
        
        Ok(Ticket{
            account_type,
            lottery_id,
            buyer,
            start_number,
            end_number,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Ticket::LEN];
        let(_account_type_dst,
            _lottery_id_dst,
            _buyer_dst,
            _start_number_dst,
            _end_number_dst,
            
        ) = mut_array_refs![dst,  1, 32, 32, 8, 8];

        let Ticket{
            account_type,
            lottery_id,
            buyer,
            start_number,
            end_number,
        } = self;
        _account_type_dst[0] = *account_type as u8;
        _lottery_id_dst.copy_from_slice(lottery_id.as_ref());
        _buyer_dst.copy_from_slice(buyer.as_ref());
        *_start_number_dst = start_number.to_le_bytes();
        *_end_number_dst = end_number.to_le_bytes();
    }
}

impl Sealed for Lottery {}

impl IsInitialized for Lottery{
    fn is_initialized(&self) -> bool {
        self.account_type != 0 
    }
}


impl Pack for Lottery {
    const LEN: usize = 129;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        
        let src = array_ref![src, 0, Lottery::LEN];
        let(_account_type,
            _authority,
            _token_reciever,
            _fee_reciever,
            _max_amount,
            _ended_slot,
            _lottery_number,
            _current_amount,
        ) = array_refs![src, 1, 32,32,32,8,8,8,8];
        
        let account_type =u8::from_le_bytes(*_account_type);
        
        let authority = Pubkey::new(_authority);
        let token_reciever = Pubkey::new(_token_reciever);
        let fee_reciever = Pubkey::new(_fee_reciever);
        let max_amount = u64::from_le_bytes(*_max_amount);
        let ended_slot = u64::from_le_bytes(*_ended_slot);
        let lottery_number = u64::from_le_bytes(*_lottery_number);
        let current_amount = u64::from_le_bytes(*_current_amount);
        Ok(Lottery{
            account_type,
            authority,
            token_reciever,
            fee_reciever,
            max_amount,
            ended_slot,
            lottery_number,
            current_amount,

        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Lottery::LEN];
        
        let(_account_type_dst,
            _authority_dst,
            _token_reciever_dst,
            _fee_reciever_dst,
            _max_amount_dst,
            _ended_slot_dst,
            _lottery_number_dst,
            _current_amount_dst,
        ) = mut_array_refs![dst, 1, 32,32,32,8,8,8,8];

        let Lottery{
            account_type,
            authority,
            token_reciever,
            fee_reciever,
            max_amount,
            ended_slot,
            lottery_number,
            current_amount,            
        } = self;
        _account_type_dst[0] = *account_type as u8;
        _authority_dst.copy_from_slice(authority.as_ref());
        _token_reciever_dst.copy_from_slice(token_reciever.as_ref());
        _fee_reciever_dst.copy_from_slice(fee_reciever.as_ref());

        *_max_amount_dst = max_amount.to_le_bytes();
        *_ended_slot_dst = ended_slot.to_le_bytes();
        *_lottery_number_dst = lottery_number.to_le_bytes();
        *_current_amount_dst = current_amount.to_le_bytes();



    }
}



