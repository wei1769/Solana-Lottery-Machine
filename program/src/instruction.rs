use crate::{error::TicketMachineError::InvalidInstruction};
use solana_program::{msg, program_error::ProgramError};
use std::convert::TryInto;
use std::fmt::format;
pub enum LotteryMachineInstructions {
    /// 0.`[writable]` lottery id
    /// 1.`[writable,signer]` lottery authority
    /// 2.`[]` fee authority
    /// 3.`[]` lottery PDA
    /// 4.`[writable]` lottery associated token account
    /// 5.`[]` fee associated token account
    /// 6.`[]` Associated Token Program
    /// 7.`[]` token mint
    /// 8.`[]` token program
    /// 9.`[]` system program
    /// 10.`[]` Sysvar Clock
    /// 11.`[]` Sysvar Rent
    
    InitLottery {
        max_amount:u64,
        slot: u64, //slot ended
    },
    /// 0.`[]` lottery id
    /// 1.`[writable]` ticket id
    /// 2.`[signer]` buyer authority
    /// 3.`[writable]` lottery associated token account
    /// 4.`[writable]` authority token account
    /// 5.`[]` token program
    /// 6.`[]` Sysvar: Clock
    Buy {
        amount: u64,// amount to participate
    },
    /// 0.`[writable]` lottery id
    /// 1.`[signer]` pool authority
    /// 2.`[]` lottery associated token account
    /// 3.`[]` Sysvar: Clock
    Draw{

    },
    /// 0.`[writable]` lottery id
    /// 1.`[writablesigner]` lottery authority
    /// 2.`[writable]` lottery associated token account
    /// 3.`[writable]` fee associated token account
    /// 4.`[writable]` winner token account
    Withdraw{

    },
}
impl LotteryMachineInstructions {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
                //First byte in data:
                //0:Initialize lottery
                //1:Buy

            0 => { 
                let (max , rest ) = Self::unpack_u64(rest).unwrap();
                let slot  = Self::unpack_u64(rest).unwrap().0;
                let message = format(format_args!(
                    "slot_ended: {:?}",
                     slot 
                ));
                msg!(&message);
                Self::InitLottery {
                    max_amount:max,
                    slot: slot,
                }
            },
            1 => {
                let amount = Self::unpack_u64(rest).unwrap().0;
                Self::Buy{
                    amount: amount,
                }
            },
            2 => {
                Self::Draw{}
            },
            3 => {
                Self::Withdraw{}
            }

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
    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        let (amount, rest) = input.split_at(1);
        let amount = amount
            .try_into()
            .ok()
            .map(u8::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok((amount, rest))
    }
}
