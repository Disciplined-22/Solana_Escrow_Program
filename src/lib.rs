use solana_program::{
    account_info::{AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
};

// ✅ Add these imports
// use solana_system_interface::system_instruction;
use borsh::{BorshDeserialize};

pub mod instructions;
use instructions::escrow::escrow_pda_create;
use instructions::nfttransfer::{transfer_tokens, TransferTokensArgs};
use instructions::pdatobuyer::{transfer_pda_to_buyer, BuyTokensArgs};

use solana_program::declare_id;

// replace with your actual program pubkey
declare_id!("4Aya5LDquudbRn6JPMSAxNM7eSqhZ1sm2gtm4ba1ndxj");



// every Solana program needs an entrypoint
entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    match instruction_data[0] {
        // off chian logic with this 
        // first hit 0  create escrow pda
        //  and then again derive the pda 
        // Hit and Pass thme to 1 => transfer_tokens(
        0 => escrow_pda_create(program_id, accounts),
        1 => {
            // ✅ Transfer: seller -> PDA (token)
            let args = TransferTokensArgs::try_from_slice(&instruction_data[1..])?;
            transfer_tokens(accounts, args)
        }
        2 => {
            // ✅ Transfer: PDA -> buyer (token)
            let args = BuyTokensArgs::try_from_slice(&instruction_data[1..])?;
            transfer_pda_to_buyer(program_id, accounts, args)
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }
}

