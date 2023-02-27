use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{check, OpenDepositsArgs, Vault, VaultType, LP_TOKEN_SEED};

#[derive(Accounts)]
#[instruction(args: OpenDepositsArgs)]
pub struct OpenDeposits<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        init,
        seeds = [
            LP_TOKEN_SEED,
            vault.key().as_ref()
        ],
        bump,
        payer = payer,
        mint::authority = vault.key(),
        mint::decimals = args.decimals,
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> OpenDeposits<'info> {
    /// We need to validate that this [`Vault`] is of [`VaultType::MultiToken`].
    pub fn validate(&self) -> Result<()> {
        check!(
            self.vault.vault_type == VaultType::MultiToken,
            InvalidVaultType
        );
        Ok(())
    }

    /// Resizes the [`Vault`] account to support new SPL Tokens.
    pub fn resize_vault(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(ctx: Context<OpenDeposits>, args: OpenDepositsArgs) -> Result<()> {
    Ok(())
}
