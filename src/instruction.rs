// instruciion.rs

use crate::state::{VersusContract, ApprovalState};

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    msg,
    program_error::ProgramError,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum WagerInstruction {
    GetWager,
    CreateWager { contract: VersusContract },
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