use anchor_lang::prelude::*;

use crate::Vault;

#[derive(Accounts)]
pub struct EnableDeposits<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub authority: Signer<'info>,
}

impl<'info> EnableDeposits<'info> {
    /// Enables deposits for the given SPL Token Mint.
    fn enable_deposits(&mut self, token_mint: Pubkey) -> Result<()> {
        let token_info = self.vault.get_token_info_mut(token_mint).unwrap();
        token_info.enabled = true;
        Ok(())
    }
}

pub fn handler(ctx: Context<EnableDeposits>, token_mint: Pubkey) -> Result<()> {
    ctx.accounts.enable_deposits(token_mint)?;
    Ok(())
}
