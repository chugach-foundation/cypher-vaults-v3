use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{check, state::Vault};

#[derive(Accounts)]
pub struct CloseDeposits<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        mint::authority = vault,
        close = rent_destination,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    pub authority: Signer<'info>,

    /// CHECK: There is no proper way to check this.
    /// Vault authority needs to be careful when closing [`Vault`].
    #[account(mut)]
    pub rent_destination: AccountInfo<'info>,
}

impl<'info> CloseDeposits<'info> {
    /// Validate that this [`Vault`] does not have outstanding deposits and LP tokens.
    pub fn validate(&self, token_mint: Pubkey) -> Result<()> {
        let token_info = self.vault.get_token_info(token_mint).unwrap();

        // check that the deposits on the vault are zeroed
        check!(token_info.deposits == 0, TokenWithDeposits);
        // and the the supply of the LP token is also zero
        check!(token_info.token_supply == 0, TokenWithLpSupply);

        Ok(())
    }
}

pub fn handler(ctx: Context<CloseDeposits>, token_mint: Pubkey) -> Result<()> {
    ctx.accounts.validate(token_mint)
}
