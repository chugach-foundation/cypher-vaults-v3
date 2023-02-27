use anchor_lang::prelude::*;

use crate::Vault;

#[derive(Accounts)]
pub struct DisableDeposits<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub authority: Signer<'info>,
}

impl<'info> DisableDeposits<'info> {
    /// Disables deposits for the given SPL Token Mint.
    fn disable_deposits(&mut self, token_mint: Pubkey) -> Result<()> {
        let token_info = self.vault.get_token_info_mut(token_mint).unwrap();
        token_info.enabled = false;
        Ok(())
    }
}

pub fn handler(ctx: Context<DisableDeposits>, token_mint: Pubkey) -> Result<()> {
    ctx.accounts.disable_deposits(token_mint)?;
    Ok(())
}
