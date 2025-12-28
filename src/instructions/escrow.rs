// ✅ Core Solana imports (minimal for this function)
use solana_program::{
    account_info::AccountInfo,           // ✅ For AccountInfo type
    program::{invoke_signed},    // ✅ For calling other programs
    pubkey::Pubkey,                      // ✅ For Pubkey type
    system_instruction,                  // ✅ For create_account instruction
    sysvar::{rent::Rent, Sysvar},        // ✅ For rent calculation
    msg,                                 // ✅ For logging
    program_error::ProgramError,         // ✅ For error handling
    entrypoint::ProgramResult,           // ✅ For ProgramResult type
};

// Constants
const ESCROW_SEED: &[u8] = b"escrow";
const DISCRIMINATOR_SIZE: usize = 8;
const SELLER_SIZE: usize = 32;
const MINT_SIZE: usize = 32;
const PRICE_SIZE: usize = 8;
const ESCROW_STATE_SIZE: usize = DISCRIMINATOR_SIZE + SELLER_SIZE + MINT_SIZE + PRICE_SIZE;


// ✅ Data serialization (if needed)

pub fn escrow_pda_create(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult { 
    // ✅ Use indices instead of iteration
    let seller = &accounts[0];        // seller (signer) - will be the payer
    let mint_account = &accounts[1];  // mint account
    let pda_account = &accounts[2];   // escrow PDA account (to be created)
    // let system_program = &accounts[3]; // system program


    // ✅ Check seller is signer (only once)
    if !seller.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // ✅ Extract mint Pubkey from mint account
    let mint = *mint_account.key;  // ✅ Get the Pubkey value
    
    // ✅ Create unique PDA using mint as seed
    let seeds: &[&[u8]] = &[ESCROW_SEED, mint.as_ref()];
    let (expected_pda, bump) = Pubkey::find_program_address(seeds, program_id);
    
    // ✅ Validate that the passed PDA matches what we derive
    if pda_account.key != &expected_pda {
        msg!("PDA mismatch: expected {}, got {}", expected_pda, pda_account.key);
        return Err(ProgramError::InvalidArgument);
    }

    // ✅ Derive escrow ATA address
    // no need to derive the ata address
    //  internally the ata address is derived using the same algorithm
    // let escrow_ata_address = spl_associated_token_account::get_associated_token_address(
    //     &pda,  // ✅ PDA is the owner
    //     &mint, // ✅ NFT mint
    // );

    // rent + space for PDA
    let rent = Rent::get()?;
    // let space: usize = 8 + 32 + 32 + 8; // EscrowState size
    // 8 (discriminator) + 32 (seller) + 32 (mint) + 8 (price)
    let lamports = rent.minimum_balance(ESCROW_STATE_SIZE);

    msg!("Creating PDA: {} for mint: {}", expected_pda, mint);

    // ✅ Step 1: Create PDA account (seller pays for it)
    invoke_signed(
        &system_instruction::create_account(
            seller.key,           // ✅ Seller pays for account creation
            pda_account.key,      // ✅ PDA address (being created)
            lamports,
            ESCROW_STATE_SIZE as u64,
            program_id,
        ),
        &[
            seller.clone(),
            pda_account.clone()// ✅ PDA account (must be included)
        ],
        &[&[b"escrow", mint.as_ref(), &[bump]]], // ✅ Use mint in seeds for signing
    )?;

    Ok(()) 
}