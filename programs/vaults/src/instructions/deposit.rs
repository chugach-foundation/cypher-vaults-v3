use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use cypher_client::{
    cpi::{accounts::DepositFunds, deposit_funds},
    program::Cypher,
    CacheAccount, Clearing, CypherAccount, CypherSubAccount, Pool, PoolNode,
};

use crate::state::{Vault, VAULT_SEED};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = lp_mint
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        token::mint = lp_mint,
        token::authority = payer,
        payer = payer,
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
    pub source_token_account: Box<Account<'info, TokenAccount>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub cypher_program: Program<'info, Cypher>,
}

impl<'info> Deposit<'info> {
    /// Deposit the input amount to the [`cypher_client::CypherAccount`].
    pub fn invoke_deposit_funds(&self, amount: u64) -> Result<()> {
        let cpi_program = self.cypher_program.to_account_info();
        let cpi_accounts = DepositFunds {
            clearing: self.clearing.to_account_info(),
            cache_account: self.cache_account.to_account_info(),
            master_account: self.cypher_account.to_account_info(),
            sub_account: self.cypher_sub_account.to_account_info(),
            pool: self.pool.to_account_info(),
            pool_node: self.pool_node.to_account_info(),
            token_vault: self.token_vault.to_account_info(),
            source_token_account: self.source_token_account.to_account_info(),
            token_mint: self.token_mint.to_account_info(),
            authority: self.vault.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        deposit_funds(
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

    pub fn invoke_mint_to(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.lp_mint.to_account_info(),
            to: self.lp_token_account.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        mint_to(
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

/// The user wants to deposit a token amount represented by `deposit_amount`,
/// taking this number we need to calculate how many tokens we are going to mint for the user.
pub fn handler(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {
    // perform the deposit into the [`Vault`]'s [`CypherAccount`]
    ctx.accounts.invoke_deposit_funds(deposit_amount)?;

    // todo: let's add this logic later
    let mint_amount = 0;

    // mint the appropriate amount of LP tokens to the end user
    ctx.accounts.invoke_mint_to(mint_amount)?;

    let vault = &mut ctx.accounts.vault;

    vault.deposits += deposit_amount;
    vault.token_supply += mint_amount;

    Ok(())
}
