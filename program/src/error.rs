use solana_program::program_error::ProgramError;
use thiserror::Error;
#[derive(Error, Debug, Copy, Clone)]
pub enum LotteryError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    ///Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    #[error("Account Not Writable")]
    AccountNotWritable,
    #[error("Ticket has been sold out")]
    PoolSoldOut,
}

impl From<LotteryError> for ProgramError {
    fn from(e: LotteryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
