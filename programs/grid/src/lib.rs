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

    pub fn create_grid(ctx: Context<CreateGrid>) -> Result<()> {
        instructions::create_grid::handler(ctx)
    }
}
