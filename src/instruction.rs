// instruction.rs

use crate::state::{Wager, Judgment};

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum WagerInstruction {
    CreateWager { wager: Wager, reserved_seats: Vec<Pubkey> },
    //ProcessDeposit { amount: u64 },
    //UpdateBelief { belief: u8 },
    //LockStatus,
    //SetJudgment { judgment: Judgment },
    //RenderPayouts,
}

impl WagerInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        
        // Get instruction variant from first byte
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        msg!("input {:?}", input);
 
        // Match instruction type and parse remaining bytes based on variant
        match variant {
            0 => {
                const WAGER_SIZE: usize = Wager::SPACE;

                let wager = Wager::try_from_slice(&rest[..WAGER_SIZE])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
        
                let reserved_seats = Vec::<Pubkey>::try_from_slice(&rest[WAGER_SIZE..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::CreateWager { wager, reserved_seats, })
            }
            /* 
            1 => {
                let amount = u64::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(Self::ProcessDeposit { amount })
            }
            2 => {
                let belief = u8::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(Self::UpdateBelief { belief })
            }
            3 => {
                Ok(Self::LockStatus)
            }
            4 => {
                let (&decision_byte, _) = rest
                    .split_first()
                    .ok_or(ProgramError::InvalidInstructionData)?;

                let judgment = match decision_byte {
                    0 => Judgment::Pending,
                    1 => Judgment::Landed,
                    2 => Judgment::Missed,
                    3 => Judgment::Push,
                    _ => return Err(ProgramError::InvalidInstructionData),
                };

                Ok(Self::SetJudgment { judgment })
            }
            5 => {
                Ok(Self::RenderPayouts)
            }
            */
            _ => {
                Err(ProgramError::InvalidInstructionData)
            }
            
        }
    }
}