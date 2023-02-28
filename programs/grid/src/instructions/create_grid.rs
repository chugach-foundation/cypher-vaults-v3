use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateGrid<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}

pub fn handler(ctx: Context<CreateGrid>) -> Result<()> {
    Ok(())
}
