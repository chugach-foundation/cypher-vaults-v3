use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

/// The seed for the PDA of the [`Vault`].
pub const VAULT_SEED: &[u8] = b"VAULT";

/// The seed for the PDA of a [`Vault`]s LP token.
pub const LP_TOKEN_SEED: &[u8] = b"LP_TOKEN";

#[derive(Debug, Default, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct CreateVaultArgs {
    /// The id of the [`Vault`].
    pub id: u64,
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
    /// The number of SPL Tokens to support deposits for.
    pub token_info_count: usize,
}

#[derive(Debug, Default, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct OpenDepositsArgs {
    /// The address of the SPL Token Mint being allowed for deposits
    pub token_mint: Pubkey,
    /// The deposit limit for this SPL Token.
    pub deposit_limit: u64,
    /// The decimals of the [`Vault`]s LP token for this SPL Token Mint.
    pub decimals: u8,
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum VaultType {
    /// The vault only allows deposits for a single token.
    SingleToken,
    /// The vault allows deposits for multiple tokens.
    MultiToken,
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
#[assert_size(64, aligns)]
#[repr(C)]
pub struct TokenInfo {
    /// Whether deposits of this SPL Token are currently enabled or not.
    pub enabled: bool, // 1
    padding: [u8; 7], // 8

    /// The amount of deposits for this SPL Token.
    pub deposits: u64, // 16
    /// The deposit limit for this SPL Token.
    pub deposit_limit: u64, // 24
    /// The supply of the LP token for this SPL Token.
    pub token_supply: u64, // 32

    /// The address of the SPL Token Mint.
    pub token_mint: Pubkey, // 64
}

impl TokenInfo {
    /// Calculate how many tokens should be minted for an LP position to be issued.
    /// We want to ensure that a LP's position always represents a proportion of the vault
    /// that is determined by the vault at the time of issue.
    /// e.g. If the vault is worth $90 and I want to issue $10, I should own 10% of all minted tokens
    ///
    /// To achieve this: (deposit value / old vault + deposit value) = (issued tokens / existing + issued tokens)
    ///
    /// Implying: issued tokens = (deposit *  existing tokens) / (old vault value)
    ///
    /// e.g. 10_000_000 * 90_000_000 / 90_000_000 = 10_000_000
    pub fn calculate_mint_amount(&self, deposit_amount: u128) -> u128 {
        if self.token_supply == 0 {
            deposit_amount
        } else {
            (deposit_amount * self.token_supply as u128) / (self.deposits as u128)
        }
    }

    /// Calculate how many tokens should be burned for a LP position to be redeemed.
    /// We want to ensure that a redeemer's position always represents a proportion of the position
    /// that is determined by the collateral at the time it is redeemed.
    /// e.g. If the position is worth $100 and I want to redeem 10 tokens, I should get 10% of the collateral
    ///
    /// To achieve this: (deposit value / old vault + deposit value) = (issued tokens / existing + issued tokens)
    ///
    /// Implying: burn amount = (vault value * redeem amount) / existing tokens
    ///
    /// e.g. 100_000_000 * 10_000_000 / 100_000_000 = 10_000_000
    pub fn calculate_burn_amount(&self, redeem_amount: u128) -> u128 {
        (self.deposits as u128 * redeem_amount) / self.token_supply as u128
    }
}

#[account]
#[repr(C)]
pub struct Vault {
    /// The version of the [`Vault`].
    pub version: u8, // 1
    /// The bump of the [`Vault`].
    pub bump: u8, // 2
    /// The number of the [`cypher_client::CypherAccount`].
    pub account_number: u8, // 3
    /// The number of the [`cypher_client::CypherSubAccount`].
    pub sub_account_number: u8, // 4
    /// The vault type.
    pub vault_type: VaultType, // 5
    padding: [u8; 3], // 8
    /// The [`Vault`]'s id.
    ///
    /// This is used as a seed for the [`Vault`]'s PDA.
    pub id: u64, // 16

    /// The authority of the [`Vault`].
    ///
    /// This is used as a seed for the [`Vault`]'s PDA.
    pub authority: Pubkey, // 48
    padding2: [u64; 4], // 80

    /// The tokens accepted in this [`Vault`].
    pub token_infos: Vec<TokenInfo>,
}

impl Vault {
    /// Calculates the size of the [`Vault`] for a given amount of SPL Tokens to be accepted.
    pub fn compute_vault_size(token_info_count: usize) -> usize {
        std::mem::size_of::<Vault>() + token_info_count * std::mem::size_of::<TokenInfo>()
    }

    /// Derives the address of a [`Vault`].
    #[cfg(feature = "client")]
    pub fn derive_address(authority: &Pubkey, id: u64) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[VAULT_SEED, authority.as_ref(), id.to_le_bytes().as_ref()],
            &crate::id(),
        )
    }

    /// Derives the address of the LP token Mint for a given [`Vault`].
    #[cfg(feature = "client")]
    pub fn derive_lp_token_mint(vault: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[LP_TOKEN_SEED, vault.as_ref()], &crate::id())
    }

    /// Initialize the [`Vault`].
    pub fn init(&mut self, authority: Pubkey, vault_bump: u8, args: &CreateVaultArgs) {
        self.authority = authority;
        self.bump = vault_bump;
        self.account_number = args.account_number;
        self.sub_account_number = args.sub_account_number;
        self.token_infos = Vec::with_capacity(args.token_info_count);
    }

    /// Gets the [`TokenInfo`] for a given SPL Token Mint.
    pub fn get_token_info_mut(&mut self, token_mint: Pubkey) -> Option<&mut TokenInfo> {
        self.token_infos
            .iter_mut()
            .find(|ti| ti.token_mint == token_mint)
    }

    /// Gets the [`TokenInfo`] for a given SPL Token Mint.
    pub fn get_token_info(&self, token_mint: Pubkey) -> Option<&TokenInfo> {
        self.token_infos
            .iter()
            .find(|ti| ti.token_mint == token_mint)
    }
}
