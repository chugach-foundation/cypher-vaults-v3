use anchor_lang::prelude::*;

use crate::{check, state::Vault};

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        close = rent_destination
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub authority: Signer<'info>,

    /// CHECK: There is no proper way to check this.
    /// Vault authority needs to be careful when closing [`Vault`].
    #[account(mut)]
    pub rent_destination: AccountInfo<'info>,
}

impl<'info> CloseVault<'info> {
    /// Validate that this [`Vault`] does not have outstanding deposits and LP tokens.
    pub fn validate(&self) -> Result<()> {
        for token_info in self.vault.token_infos.iter() {
            // check that the deposits for this SPL Token are zeroed
            check!(token_info.deposits == 0, TokenWithDeposits);
            // and the the supply of the corresponding LP token is also zero
            check!(token_info.token_supply == 0, TokenWithLpSupply);
        }
        Ok(())
    }
}

pub fn handler(ctx: Context<CloseVault>) -> Result<()> {
    ctx.accounts.validate()
}
