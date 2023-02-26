use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

/// The seed for the PDA of the [`Vault`].
pub const VAULT_SEED: &[u8] = b"VAULT";

/// The seed for the PDA of a [`Vault`]s LP token.
pub const LP_TOKEN_SEED: &[u8] = b"LP_TOKEN";

#[derive(Debug, Default, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct CreateVaultArgs {
    /// The decimals of the [`Vault`]s LP token.
    pub decimals: u8,
    /// The number of the [`cypher_client::CypherSubAccount`].
    pub account_number: u8,
    /// The bump of the [`cypher_client::CypherAccount`].
    pub account_bump: u8,
    /// The number of the [`cypher_client::CypherSubAccount`].
    pub sub_account_number: u8,
    /// The bump of the [`cypher_client::CypherSubAccount`].
    pub sub_account_bump: u8,
    /// The alias of the [`cypher_client::CypherSubAccount`].
    pub sub_account_alias: [u8; 32],
    /// The mint of the token this [`Vault`] represents.
    pub token_mint: Pubkey,
}

#[assert_size(128, aligns)]
#[account]
#[repr(C)]
pub struct Vault {
    /// The bump of the [`Vault`].
    pub bump: u8, // 1
    /// The number of the [`cypher_client::CypherAccount`].
    pub account_number: u8, // 2
    /// The number of the [`cypher_client::CypherSubAccount`].
    pub sub_account_number: u8, // 3
    padding: [u8; 5], // 8

    /// The authority of the [`Vault`].
    pub authority: Pubkey, // 40
    /// The token mint that this [`Vault`] is responsible for.
    pub token_mint: Pubkey, // 72
    /// The mint of the LP token issued for deposits through this [`Vault`].
    pub lp_mint: Pubkey, // 104

    /// The amount of deposits in this [`Vault`].
    pub deposits: u64, // 112

    /// The supply of the LP token.
    pub token_supply: u64, // 120
    padding2: [u8; 8],
}

impl Vault {
    /// Initialize the [`Vault`].
    pub fn init(
        &mut self,
        authority: Pubkey,
        lp_mint: Pubkey,
        vault_bump: u8,
        args: &CreateVaultArgs,
    ) {
        self.authority = authority;
        self.lp_mint = lp_mint;
        self.bump = vault_bump;
        self.account_number = args.account_number;
        self.sub_account_number = args.sub_account_number;
        self.token_mint = args.token_mint;
    }
}
