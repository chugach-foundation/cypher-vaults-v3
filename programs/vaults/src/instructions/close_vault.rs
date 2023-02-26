use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::state::Vault;

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        has_one = lp_mint,
        close = rent_destination
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        mint::authority = authority,
        close = rent_destination,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    pub authority: Signer<'info>,

    /// CHECK: There is no proper way to check this.
    /// Vault authority needs to be careful when closing [`Vault`].
    #[account(mut)]
    pub rent_destination: AccountInfo<'info>,
}

impl<'info> CloseVault<'info> {
    /// Validate that this [`Vault`] does not have outstanding deposits and LP tokens.
    pub fn validate(&self) -> Result<()> {
        // check that the deposits on the vault are zeroed

        // and the the supply of the LP token is also zero

        Ok(())
    }
}

pub fn handler(ctx: Context<CloseVault>) -> Result<()> {
    ctx.accounts.validate()
}
