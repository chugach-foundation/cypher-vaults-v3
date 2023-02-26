use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use cypher_client::{
    cpi::{accounts::WithdrawFunds, withdraw_funds},
    program::Cypher,
    CacheAccount, Clearing, CypherAccount, CypherSubAccount, Pool, PoolNode,
};

use crate::state::{Vault, VAULT_SEED};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        has_one = lp_mint
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
        token::mint = lp_mint,
        token::authority = payer,
    )]
    pub lp_token_account: Box<Account<'info, TokenAccount>>,

    pub cache_account: AccountLoader<'info, CacheAccount>,

    pub clearing: AccountLoader<'info, Clearing>,

    #[account(mut)]
    pub cypher_account: AccountLoader<'info, CypherAccount>,

    #[account(mut)]
    pub cypher_sub_account: AccountLoader<'info, CypherSubAccount>,

    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    #[account(mut)]
    pub pool_node: AccountLoader<'info, PoolNode>,

    #[account(mut)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub destination_token_account: Box<Account<'info, TokenAccount>>,

    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Checked via CPI to [`Cypher`].
    pub vault_signer: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub cypher_program: Program<'info, Cypher>,
}

impl<'info> Withdraw<'info> {
    /// Withdraw the input amount from the [`cypher_client::CypherAccount`].
    pub fn invoke_withdraw_funds(&self, amount: u64) -> Result<()> {
        let cpi_program = self.cypher_program.to_account_info();
        let cpi_accounts = WithdrawFunds {
            clearing: self.clearing.to_account_info(),
            cache_account: self.cache_account.to_account_info(),
            master_account: self.cypher_account.to_account_info(),
            sub_account: self.cypher_sub_account.to_account_info(),
            pool: self.pool.to_account_info(),
            pool_node: self.pool_node.to_account_info(),
            token_vault: self.token_vault.to_account_info(),
            destination_token_account: self.destination_token_account.to_account_info(),
            token_mint: self.token_mint.to_account_info(),
            vault_signer: self.vault_signer.to_account_info(),
            authority: self.vault.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        withdraw_funds(
            CpiContext::new_with_signer(
                cpi_program,
                cpi_accounts,
                &[&[
                    VAULT_SEED,
                    self.cypher_account.key().as_ref(),
                    self.cypher_sub_account.key().as_ref(),
                    &[self.vault.bump],
                ]],
            ),
            amount,
        )
    }

    /// Burn a corresponding amount of LP tokens.
    pub fn invoke_burn(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Burn {
            mint: self.lp_mint.to_account_info(),
            from: self.lp_token_account.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        burn(
            CpiContext::new_with_signer(
                cpi_program,
                cpi_accounts,
                &[&[
                    VAULT_SEED,
                    self.cypher_account.key().as_ref(),
                    self.cypher_sub_account.key().as_ref(),
                    &[self.vault.bump],
                ]],
            ),
            amount,
        )
    }
}

/// The user wants to withdraw a token amount represented by `withdraw_amount`,
/// taking this number we need to calculate how many tokens we are going to burn for the user.
pub fn handler(ctx: Context<Withdraw>, withdraw_amount: u64) -> Result<()> {
    // todo: let's add this logic later
    let burn_amount = 0;

    // burn the corresponding amount
    ctx.accounts.invoke_burn(burn_amount)?;

    // finally withdraw from the [`Vault`]'s [`CypherAccount`]
    ctx.accounts.invoke_withdraw_funds(withdraw_amount)?;

    let vault = &mut ctx.accounts.vault;

    // update the [`Vault`]'s data
    vault.token_supply = vault.token_supply.checked_sub(burn_amount).unwrap();
    vault.deposits = vault.deposits.checked_sub(withdraw_amount).unwrap();

    Ok(())
}
