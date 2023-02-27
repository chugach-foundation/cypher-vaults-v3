mod error;
mod instructions;
mod state;

pub use instructions::*;
pub use state::*;

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod vaults {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>, args: CreateVaultArgs) -> Result<()> {
        instructions::create_vault::handler(ctx, args)
    }

    pub fn close_deposits(ctx: Context<CloseDeposits>, token_mint: Pubkey) -> Result<()> {
        instructions::close_deposits::handler(ctx, token_mint)
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        instructions::close_vault::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn disable_deposits(ctx: Context<DisableDeposits>, token_mint: Pubkey) -> Result<()> {
        instructions::disable_deposits::handler(ctx, token_mint)
    }

    pub fn enable_deposits(ctx: Context<EnableDeposits>, token_mint: Pubkey) -> Result<()> {
        instructions::enable_deposits::handler(ctx, token_mint)
    }

    pub fn open_deposits(ctx: Context<OpenDeposits>, args: OpenDepositsArgs) -> Result<()> {
        instructions::open_deposits::handler(ctx, args)
    }

    pub fn set_deposit_limit(
        ctx: Context<SetDepositLimit>,
        token_mint: Pubkey,
        amount: u64,
    ) -> Result<()> {
        instructions::set_deposit_limit::handler(ctx, token_mint, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }
}
