// instruciion.rs

use crate::state::{VersusContract, ApprovalState};

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    program_error::ProgramError,
    msg,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum WagerInstruction {
    GetWager,
    CreateWager { contract: VersusContract },
    ProcessDeposit { amount: u64 },
    UpdateBelief { belief: u8 },
    LockStatus,
    SetApproval { decision: ApprovalState },
}

impl WagerInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Get the instruction variant from the first byte
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        msg!("input {:?}", input);
 
        // Match instruction type and parse the remaining bytes based on the variant
        match variant {
            0 => { // No additional data needed
                Ok(Self::GetWager)
            }
            1 => {
                let versus_contract = VersusContract::try_from_slice(
                    &rest).map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::CreateWager { contract: versus_contract })
            }
            2 => {
                let amount = u64::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(Self::ProcessDeposit { amount })
            }
            3 => {
                let belief = u8::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(Self::UpdateBelief { belief })
            }
            4 => {
                Ok(Self::LockStatus)
            }
            5 => {
                let (&decision_byte, _) = rest
                    .split_first()
                    .ok_or(ProgramError::InvalidInstructionData)?;

                let decision = match decision_byte {
                    0 => ApprovalState::Pending,
                    1 => ApprovalState::Landed,
                    2 => ApprovalState::Missed,
                    3 => ApprovalState::Push,
                    _ => return Err(ProgramError::InvalidInstructionData),
                };

                Ok(Self::SetApproval { decision })
            }
            _ => {
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}