use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        pubkey::Pubkey,
        system_instruction,
    },
    spl_associated_token_account::instruction as ata_ix,
    spl_token::id as spl_token_program_id,
    spl_token::instruction as token_instruction,
};

const ESCROW_SEED: &[u8] = b"escrow";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BuyTokensArgs {
    pub quantity: u64,
    pub bump_seed: u8,
    pub price: u64, // SOL amount in lamports (buyer -> seller)
}

// Transfer token: PDA -> buyer
// get the instruction data has a prameter to get the bump_seed
// pass the bump seed correctly
pub fn transfer_pda_to_buyer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: BuyTokensArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let mint_account = next_account_info(accounts_iter)?; // token mint address
    let from_associated_token_account = next_account_info(accounts_iter)?; // PDA's ATA (source)
    let to_associated_token_account = next_account_info(accounts_iter)?; // buyer's ATA (destination)
    let owner = next_account_info(accounts_iter)?; // escrow PDA
    let recipient = next_account_info(accounts_iter)?; // buyer's pubkey
    let payer = next_account_info(accounts_iter)?; // buyer (signer for SOL transfer)
    let seller = next_account_info(accounts_iter)?; // seller (receives SOL)
    let system_program = next_account_info(accounts_iter)?; // system program
    let token_program = next_account_info(accounts_iter)?; // token program

    // ✅ Validate buyer is signer for SOL transfer
    if !payer.is_signer {
        return Err(solana_program::program_error::ProgramError::MissingRequiredSignature);
    }

    // ✅ Validate PDA matches expected derivation
    let mint = *mint_account.key;
    let seeds: &[&[u8]] = &[ESCROW_SEED, mint.as_ref()];
    let (expected_pda, bump) = Pubkey::find_program_address(seeds, program_id);

    if owner.key != &expected_pda {
        msg!("PDA mismatch: expected {}, got {}", expected_pda, owner.key);
        return Err(solana_program::program_error::ProgramError::InvalidArgument);
    }

    // Validate bump seed matches
    if bump != args.bump_seed {
        msg!(
            "Bump seed mismatch: expected {}, got {}",
            bump,
            args.bump_seed
        );
        return Err(solana_program::program_error::ProgramError::InvalidArgument);
    }

    // first phase
    if to_associated_token_account.lamports() == 0 {
        msg!("Creating associated token account for recipient...");

        // bump_seed is passed via instruciton data from client side
        // mentin the  recipient_pubkey.as_ref()
        // is pubkey needed here ?
        // let signer_seeds: &[&[u8]] = &[b"escrow", mint_account.key.as_ref(), &[args.bump_seed]];
        // let signer_seeds: &[&[&[u8]]] = &[&[b"escrow", mint_account.key.as_ref(), &[args.bump_seed]]];

        let transfer_ix = ata_ix::create_associated_token_account(
            payer.key,               // funding_address
            recipient.key,           // wallet_address (PDA as owner)
            mint_account.key,        // token_mint_address
            &spl_token_program_id(), // ✅ Correct Token Program ID
        );

        invoke(
            &transfer_ix,
            &[
                payer.clone(),
                to_associated_token_account.clone(),
                recipient.clone(),
                mint_account.clone(),
                system_program.clone(),
                token_program.clone(),
            ],
        )?;
    } else {
        msg!("Associated token account exists.");
    }
    msg!(
        "Recipient Associated Token Address: {}",
        to_associated_token_account.key
    );

    msg!("Transferring {} tokens...", args.quantity);
    msg!("Mint: {}", mint_account.key);
    msg!("Owner Token Address: {}", from_associated_token_account.key);
    msg!(
        "Recipient Token Address: {}",
        to_associated_token_account.key
    );

    // second phase - PDA -> buyer token transfer
    let token_pda_to_buyer_ix = token_instruction::transfer(
        token_program.key,
        from_associated_token_account.key, // PDA's ATA (source)
        to_associated_token_account.key,   // Buyer's ATA (destination)
        owner.key,                         // PDA is the authority
        &[],                               // Signers go in invoke_signed, not here
        args.quantity,
    )?;

    // ✅ Derive PDA seeds for signing
    let signer_seeds: &[&[&[u8]]] = &[&[ESCROW_SEED, mint.as_ref(), &[args.bump_seed]]];

    invoke_signed(
        &token_pda_to_buyer_ix,
        &[
            from_associated_token_account.clone(), // Source (PDA's ATA)
            to_associated_token_account.clone(),   // Destination (Buyer's ATA)
            owner.clone(),                         // Authority (PDA)
                                                   // token_program.clone(),                  // Token program
        ],
        signer_seeds,
    )?;

    // third phase - SOL transfer: buyer -> seller
    msg!(
        "Transferring {} lamports from buyer to seller...",
        args.price
    );

    let sol_transfer_ix = system_instruction::transfer(
        payer.key,  // From: buyer (signer)
        seller.key, // To: seller
        args.price, // Amount in lamports
    );

    invoke(
        &sol_transfer_ix,
        &[
            payer.clone(),          // Buyer (signer)
            seller.clone(),         // Seller (destination)
            system_program.clone(), // System program
        ],
    )?;

    msg!(
        "SOL transfer successful. {} lamports transferred to seller.",
        args.price
    );

    msg!("Token and SOL transfers completed successfully.");

    Ok(())
}
