use anchor_lang::prelude::*;

use crate::Vault;

#[derive(Accounts)]
pub struct SetDepositLimit<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub authority: Signer<'info>,
}

impl<'info> SetDepositLimit<'info> {
    /// Enables deposits for the given SPL Token Mint.
    fn set_deposit_limit(&mut self, token_mint: Pubkey, deposit_limit: u64) -> Result<()> {
        let token_info = self.vault.get_token_info_mut(token_mint).unwrap();
        token_info.deposit_limit = deposit_limit;
        Ok(())
    }
}

pub fn handler(
    ctx: Context<SetDepositLimit>,
    token_mint: Pubkey,
    deposit_limit: u64,
) -> Result<()> {
    ctx.accounts.set_deposit_limit(token_mint, deposit_limit)?;
    Ok(())
}
