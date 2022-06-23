use {
    anchor_lang::{prelude::*, solana_program::program::invoke},
    anchor_spl::{
        token,
        token::{MintTo, Token},
    },
    mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v2},
};

// Our struct to mint NFT
// This is the struct we will pass in as instructions

#[derive(Accounts)]
pub struct MintNFT<'info> {
    /// CHECK: master edition
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: metadata
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: mint
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: mint authority
    #[account(mut)]
    pub mint_authority: Signer<'info>,

    /// CHECK: payer
    #[account(mut)]
    pub payer: AccountInfo<'info>,

    /// CHECK: rent
    pub rent: AccountInfo<'info>,
    pub system_program: Program<'info, System>,

    /// CHECK: token account
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,

    /// CHECK: token metadata program
    pub token_metadata_program: UncheckedAccount<'info>,
}

declare_id!("EKWmd8gcme68CteJFfXXYtm4vCcPHGGLBtuUnp7DWHJM");

#[program]
pub mod nft_marketplace {

    use super::*;

    pub fn mint_nft(
        context: Context<MintNFT>,
        collection_key: Pubkey,
        metadata_uri: String,
        metadata_title: String,
    ) -> Result<()> {
        msg!("Initializing NFT minting...");

        // Configure Cross-Program-Invokation (CPI) context

        let token_program = context.accounts.token_program.to_account_info();
        let token_mint = context.accounts.mint.to_account_info();
        let token_mint_id = token_mint.key;
        let accounts = MintTo {
            mint: token_mint,
            to: context.accounts.token_account.to_account_info(),
            authority: context.accounts.payer.to_account_info(),
        };

        let cpi_context = CpiContext::new(token_program, accounts);
        token::mint_to(cpi_context, 1)?;

        msg!("Your NFT has been minted!");
        msg!(" Token ID {}", token_mint_id);

        // Set Up the associated metadata
        let creators = vec![
            mpl_token_metadata::state::Creator {
                address: collection_key,
                verified: false,
                share: 100,
            },
            mpl_token_metadata::state::Creator {
                address: context.accounts.mint_authority.key(),
                verified: false,
                share: 0,
            },
        ];

        let token_symbol = std::string::ToString::to_string("ARTIST");

        // Invoke the solana program to create the metadata accounts
        // Wrapped nicely in anchor instructions

        invoke(
            &create_metadata_accounts_v2(
                context.accounts.token_program.key(),
                context.accounts.metadata.key(),
                context.accounts.mint.key(),
                context.accounts.mint_authority.key(),
                context.accounts.payer.key(),
                context.accounts.payer.key(),
                metadata_title,
                token_symbol,
                metadata_uri,
                Some(creators),
                1,
                true,
                false,
                None,
                None,
            ),
            &[
                context.accounts.metadata.to_account_info(),
                context.accounts.mint.to_account_info(),
                context.accounts.mint_authority.to_account_info(),
                context.accounts.payer.to_account_info(),
                context.accounts.token_metadata_program.to_account_info(),
                context.accounts.token_program.to_account_info(),
                context.accounts.system_program.to_account_info(),
                context.accounts.rent.to_account_info(),
            ],
        )?;

        msg!("Metadata account created successfully!");

        invoke(
            &create_master_edition_v3(
                context.accounts.token_metadata_program.key(),
                context.accounts.master_edition.key(),
                context.accounts.mint.key(),
                context.accounts.payer.key(),
                context.accounts.mint_authority.key(),
                context.accounts.metadata.key(),
                context.accounts.payer.key(),
                Some(0),
            ),
            &[
                context.accounts.metadata.to_account_info(),
                context.accounts.mint.to_account_info(),
                context.accounts.mint_authority.to_account_info(),
                context.accounts.payer.to_account_info(),
                context.accounts.token_metadata_program.to_account_info(),
                context.accounts.token_program.to_account_info(),
                context.accounts.system_program.to_account_info(),
                context.accounts.rent.to_account_info(),
            ],
        )?;

        msg!("NFT delivered successfully. Check you wallet");

        Ok(())
    }
}
