use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke},
    },
    spl_token::instruction as token_instruction, // passing this to invoke of transfer function which is not allowed
    spl_associated_token_account::instruction as ata_ix,
    spl_token::id as spl_token_program_id,
};


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TransferTokensArgs {
    pub quantity: u64,
    pub bump_seed: u8,
}

// get the instruction data has a prameter to get the bump_seed
// pass the bump seed correctly 
pub fn transfer_tokens(accounts: &[AccountInfo], args: TransferTokensArgs) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let mint_account = next_account_info(accounts_iter)?;
    let from_associated_token_account = next_account_info(accounts_iter)?;
    let to_associated_token_account = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let recipient = next_account_info(accounts_iter)?; // change to pda later  recipient = escrow_pda_account.key
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    // let associated_token_program = next_account_info(accounts_iter)?;  // expected ata from client side using ATA = find_program_address(
    // let rent_sysvar = next_account_info(accounts_iter)?;

    // use pda here
    // every time u gotta check
    // be sure how would you pass it from client side 

    // first phase 
    if to_associated_token_account.lamports() == 0 {
        msg!("Creating associated token account for recipient...");

        // bump_seed is passed via instruciton data from client side 
        // mentin the  recipient_pubkey.as_ref()  
        // is pubkey needed here ?
        // let signer_seeds: &[&[u8]] = &[b"escrow", mint_account.key.as_ref(), &[args.bump_seed]];
        // let signer_seeds: &[&[&[u8]]] = &[&[b"escrow", mint_account.key.as_ref(), &[args.bump_seed]]];
      

        let transfer_ix = ata_ix::create_associated_token_account(
            payer.key,           // funding_address
            recipient.key,       // wallet_address (PDA as owner)
            mint_account.key,    // token_mint_address
            &spl_token_program_id(),   // ✅ Correct Token Program ID
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



// second phase 
    let token_seller_to_buyer_ix = token_instruction::transfer(
        token_program.key,
        from_associated_token_account.key,
        to_associated_token_account.key,
        owner.key,   
        &[&owner.key], // its seller --> pda 
        args.quantity,
    )?;
    // here use pda here too 
    // from seller --> user 
    invoke(
        &token_seller_to_buyer_ix,
        &[
            from_associated_token_account.clone(),  // Source
            to_associated_token_account.clone(),    // Destination
            owner.clone(),                          // Authority (signer)
            owner.clone(),     
        ],
    )?;

    // but how are you doing it for teh pda buyer 

    msg!("Tokens transferred successfully.");

    Ok(())
}