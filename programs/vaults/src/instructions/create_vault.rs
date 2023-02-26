use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use cypher_client::{
    cpi::{
        accounts::{CreateAccount, CreateSubAccount},
        create_account, create_sub_account,
    },
    program::Cypher,
    Clearing, CypherAccount, CypherSubAccount,
};

use crate::state::{CreateVaultArgs, Vault, LP_TOKEN_SEED, VAULT_SEED};

#[derive(Accounts)]
#[instruction(args: CreateVaultArgs)]
pub struct CreateVault<'info> {
    #[account(
        init,
        seeds = [
            VAULT_SEED,
            cypher_account.key().as_ref(),
            cypher_sub_account.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = std::mem::size_of::<Vault>() + 8
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

    pub clearing: AccountLoader<'info, Clearing>,

    #[account(mut)]
    pub cypher_account: AccountLoader<'info, CypherAccount>,

    #[account(mut)]
    pub cypher_sub_account: AccountLoader<'info, CypherSubAccount>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub cypher_program: Program<'info, Cypher>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateVault<'info> {
    /// Invokes [`Cypher`]'s [`CreateAccount`] instruction.
    pub fn invoke_create_account(&self, args: CreateVaultArgs) -> Result<()> {
        let cpi_program = self.cypher_program.to_account_info();
        let cpi_accounts = CreateAccount {
            clearing: self.clearing.to_account_info(),
            master_account: self.cypher_account.to_account_info(),
            authority: self.vault.to_account_info(),
            payer: self.payer.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        create_account(
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
            args.account_number,
            args.account_bump,
        )
    }

    /// Invokes [`Cypher`]'s [`CreateSubAccount`] instruction.
    pub fn invoke_create_sub_account(&self, args: CreateVaultArgs) -> Result<()> {
        let cpi_program = self.cypher_program.to_account_info();
        let cpi_accounts = CreateSubAccount {
            master_account: self.cypher_account.to_account_info(),
            sub_account: self.cypher_sub_account.to_account_info(),
            authority: self.vault.to_account_info(),
            payer: self.payer.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        create_sub_account(
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
            args.sub_account_number,
            args.sub_account_bump,
            args.sub_account_alias,
        )
    }
}

pub fn handler(ctx: Context<CreateVault>, args: CreateVaultArgs) -> Result<()> {
    let vault_bump = ctx.bumps.get("vault").unwrap();

    let vault = &mut ctx.accounts.vault;

    vault.init(
        ctx.accounts.authority.key(),
        ctx.accounts.lp_token_mint.key(),
        *vault_bump,
        &args,
    );

    ctx.accounts.invoke_create_account(args)?;

    ctx.accounts.invoke_create_sub_account(args)?;

    Ok(())
}
