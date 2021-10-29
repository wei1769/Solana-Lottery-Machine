use crate::error::LotteryError::InvalidInstruction;
use solana_program::{ program_error::ProgramError};
use std::convert::TryInto;
pub enum LotteryMachineInstructions {
    /// 0.`[writable,signer]` lottery id
    /// 1.`[writable,signer]` lottery authority
    /// 2.`[]` fee authority
    /// 3.`[writable]` lottery PDA
    /// 4.`[writable]` token reciever (ATA owned by lottery PDA, Derived from mint,lottery PDA) 
    /// 5.`[writable]` fee reciever (ATA owned by fee authority)
    /// 6.`[]` Associated Token Program
    /// 7.`[]` token mint
    /// 8.`[]` token program
    /// 9.`[]` system program
    /// 10.`[]` Sysvar Clock
    /// 11.`[]` Sysvar Rent
    InitLottery {
        max_amount: u64,
        slot: u64, //how mant slot this Lottery last
    },
    /// 0.`[writable]` lottery id
    /// 1.`[writable,signer]` ticket id
    /// 2.`[writable,signer]` buyer authority
    /// 3.`[writable]` token reciever (ATA owned by lottery PDA, Derived from mint,lottery PDA)
    /// 4.`[writable]` buyer token account
    /// 5.`[]` token program
    /// 6.`[]` Sysvar: Clock
    /// 7.`[]` system program
    /// 8.`[]` Sysvar Rent
    Buy {
        amount: u64, // amount to participate
    },
    /// 0.`[writable]` lottery id
    /// 1.`[signer]` lottery authority
    /// 2.`[]` Sysvar: Clock
    /// 3.`[]` Sysvar: Slot Hashes
    Draw {},
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
    Withdraw {},

    Close {},
}
impl LotteryMachineInstructions {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            //First byte in data:
            //0:Initialize lottery
            //1:Buy
            0 => {
                let (max, rest) = Self::unpack_u64(rest).unwrap();
                let slot = Self::unpack_u64(rest).unwrap().0;
                //let message = format(format_args!("slot_ended: {:?}, max", slot));
                //msg!(&message);
                Self::InitLottery {
                    max_amount: max,
                    slot: slot,
                }
            }
            1 => {
                let amount = Self::unpack_u64(rest).unwrap().0;
                Self::Buy { amount: amount }
            }
            2 => Self::Draw {},
            3 => Self::Withdraw {},
            4 => Self::Close {},
            _ => return Err(InvalidInstruction.into()),
        })
    }
    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        let (amount, rest) = input.split_at(8);
        let amount = amount
            .try_into()
            .ok()
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok((amount, rest))
    }
    
}
